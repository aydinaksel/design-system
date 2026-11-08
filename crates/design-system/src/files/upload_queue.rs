use std::fmt::Write;
use std::time::Duration;

use leptos::prelude::*;
use leptos_use_event_listener::use_event_listener;
use leptos_use_window::use_window;
use send_wrapper::SendWrapper;

use super::display::{FileItem, ItemState};
use super::photo_shrinking::shrink_photo;
use super::upload::{FileUploader, PendingUpload, UploadFailure, UploadRules};
use super::StoredFile;
use crate::element_key::ElementKey;

const MAX_CONCURRENT_UPLOADS: usize = 3;
const FIRST_RECONNECT_DELAY: Duration = Duration::from_secs(2);
const LONGEST_RECONNECT_DELAY: Duration = Duration::from_secs(30);
const MOST_AUTOMATIC_RECONNECTS: u32 = 10;
const CONNECTION_LOST_MESSAGE: &str = "The connection kept dropping. Retry once you have a signal.";
const UPLOAD_KEY_BYTES: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
enum UploadStatus {
    Waiting,
    Uploading,
    Reconnecting,
    Failed(String),
    Done(StoredFile),
}

#[derive(Clone)]
struct LocalUpload {
    stand_in: StoredFile,
    source: Option<SendWrapper<web_sys::File>>,
    upload_key: String,
    shrink_edge_pixels: Option<u32>,
    preview_url: Option<String>,
    status: UploadStatus,
    reconnects: u32,
}

impl LocalUpload {
    fn key(&self) -> &str {
        &self.stand_in.key
    }

    fn is_unfinished(&self) -> bool {
        matches!(
            self.status,
            UploadStatus::Waiting | UploadStatus::Uploading | UploadStatus::Reconnecting
        )
    }

    fn settle(&mut self, prepared: web_sys::File, outcome: Result<StoredFile, UploadFailure>) {
        self.shrink_edge_pixels = None;
        self.status = match outcome {
            Ok(stored) => {
                revoke_preview(self.preview_url.take().as_deref());
                self.source = None;
                UploadStatus::Done(stored)
            }
            Err(UploadFailure::Rejected(reason)) => UploadStatus::Failed(reason),
            Err(UploadFailure::Interrupted(_)) if self.reconnects >= MOST_AUTOMATIC_RECONNECTS => {
                UploadStatus::Failed(CONNECTION_LOST_MESSAGE.to_owned())
            }
            Err(UploadFailure::Interrupted(_)) => {
                self.reconnects = self.reconnects.saturating_add(1);
                UploadStatus::Reconnecting
            }
        };
        if self.source.is_some() {
            self.source = Some(SendWrapper::new(prepared));
        }
    }
}

struct UploadJob {
    key: String,
    file: web_sys::File,
    upload_key: String,
    shrink_edge_pixels: Option<u32>,
}

fn revoke_preview(preview_url: Option<&str>) {
    if let Some(url) = preview_url {
        web_sys::Url::revoke_object_url(url).unwrap_or_default();
    }
}

fn stand_in_for(file: &web_sys::File) -> (StoredFile, Option<String>) {
    let content_type = Some(file.type_()).filter(|content_type| !content_type.is_empty());
    let mut stand_in = StoredFile::new(format!("upload-{}", ElementKey::next()), file.name(), "");
    stand_in.content_type = content_type;
    stand_in.size_bytes = byte_count(file.size());
    let preview_url = stand_in
        .image_url()
        .and_then(|_| web_sys::Url::create_object_url_with_blob(file).ok());
    if let Some(url) = &preview_url {
        stand_in.url.clone_from(url);
    }
    (stand_in, preview_url)
}

fn byte_count(size: f64) -> Option<u64> {
    format!("{size:.0}").parse().ok()
}

fn reconnect_delay(reconnects: u32) -> Duration {
    let doublings = reconnects
        .saturating_sub(1)
        .min(u32::BITS.saturating_sub(1));
    FIRST_RECONNECT_DELAY
        .saturating_mul(2_u32.saturating_pow(doublings))
        .min(LONGEST_RECONNECT_DELAY)
}

fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        write!(text, "{byte:02x}").unwrap_or_default();
        text
    })
}

#[cfg(feature = "hydrate")]
fn new_upload_key() -> String {
    let mut bytes = [0_u8; UPLOAD_KEY_BYTES];
    let filled = web_sys::window()
        .and_then(|window| window.crypto().ok())
        .and_then(|crypto| crypto.get_random_values_with_u8_array(&mut bytes).ok());
    match filled {
        Some(_) => hexadecimal(&bytes),
        None => format!("{}-{}", js_sys::Date::now(), ElementKey::next()),
    }
}

#[cfg(not(feature = "hydrate"))]
fn new_upload_key() -> String {
    format!(
        "{}{}",
        hexadecimal(&[0; UPLOAD_KEY_BYTES]),
        ElementKey::next()
    )
}

#[cfg(feature = "hydrate")]
fn browser_is_offline() -> bool {
    web_sys::window().is_some_and(|window| !window.navigator().on_line())
}

#[cfg(not(feature = "hydrate"))]
fn browser_is_offline() -> bool {
    false
}

fn blamed_on_the_connection(failure: UploadFailure) -> UploadFailure {
    match failure {
        UploadFailure::Rejected(reason) if browser_is_offline() => {
            UploadFailure::Interrupted(reason)
        }
        other => other,
    }
}

#[derive(Clone, Copy)]
pub(super) struct UploadQueue {
    entries: RwSignal<Vec<LocalUpload>>,
    uploader: StoredValue<FileUploader>,
    rules: StoredValue<UploadRules>,
}

impl UploadQueue {
    pub(super) fn new(uploader: FileUploader, rules: UploadRules) -> Self {
        let entries: RwSignal<Vec<LocalUpload>> = RwSignal::new(Vec::new());
        on_cleanup(move || {
            entries.try_with_untracked(|entries| {
                for entry in entries {
                    revoke_preview(entry.preview_url.as_deref());
                }
            });
        });
        let queue = Self {
            entries,
            uploader: StoredValue::new(uploader),
            rules: StoredValue::new(rules),
        };
        drop(use_event_listener(
            use_window(),
            leptos::ev::online,
            move |_| {
                queue.reconnect(None);
            },
        ));
        queue
    }

    pub(super) fn add(self, files: Vec<web_sys::File>) {
        let rules = self.rules.get_value();
        let mut accepted_any = false;
        let mut added: Vec<LocalUpload> = Vec::new();
        for file in files {
            let (stand_in, preview_url) = stand_in_for(&file);
            let rejection = if accepted_any && !rules.multiple() {
                Some(UploadRules::too_many_message().to_owned())
            } else {
                rules.rejection(
                    &file.name(),
                    stand_in.content_type.as_deref(),
                    stand_in.size_bytes.unwrap_or_default(),
                )
            };
            accepted_any = accepted_any || rejection.is_none();
            added.push(LocalUpload {
                shrink_edge_pixels: rules.shrink_edge_pixels(stand_in.content_type.as_deref()),
                stand_in,
                source: rejection.is_none().then(|| SendWrapper::new(file)),
                upload_key: new_upload_key(),
                preview_url,
                status: rejection.map_or(UploadStatus::Waiting, UploadStatus::Failed),
                reconnects: 0,
            });
        }
        self.entries.update(|entries| entries.extend(added));
        self.start_waiting();
    }

    pub(super) fn retry(self, key: &str) {
        self.entries.update(|entries| {
            let retryable = entries.iter_mut().find(|entry| {
                entry.key() == key
                    && entry.source.is_some()
                    && entry.status != UploadStatus::Uploading
            });
            if let Some(entry) = retryable {
                entry.status = UploadStatus::Waiting;
                entry.reconnects = 0;
            }
        });
        self.start_waiting();
    }

    fn reconnect(self, only_key: Option<&str>) {
        let resumed = self.entries.try_update(|entries| {
            entries
                .iter_mut()
                .filter(|entry| entry.status == UploadStatus::Reconnecting)
                .filter(|entry| only_key.is_none_or(|key| entry.key() == key))
                .for_each(|entry| entry.status = UploadStatus::Waiting);
        });
        if resumed.is_some() {
            self.start_waiting();
        }
    }

    fn reconnect_later(self, key: String, reconnects: u32) {
        set_timeout(
            move || self.reconnect(Some(&key)),
            reconnect_delay(reconnects),
        );
    }

    pub(super) fn dismiss(self, key: &str) -> bool {
        let mut dismissed = false;
        self.entries.update(|entries| {
            entries.retain(|entry| {
                let keep = entry.key() != key || entry.status == UploadStatus::Uploading;
                if !keep {
                    revoke_preview(entry.preview_url.as_deref());
                    dismissed = true;
                }
                keep
            });
        });
        dismissed
    }

    pub(super) fn forget_uploaded(self, stored_key: &str) {
        self.entries.try_update(|entries| {
            entries.retain(|entry| !matches!(&entry.status, UploadStatus::Done(file) if file.key == stored_key));
        });
    }

    pub(super) fn has_unfinished(self) -> bool {
        self.entries
            .try_with_untracked(|entries| entries.iter().any(LocalUpload::is_unfinished))
            .unwrap_or(false)
    }

    pub(super) fn is_busy(self) -> bool {
        self.entries
            .with(|entries| entries.iter().any(LocalUpload::is_unfinished))
    }

    pub(super) fn is_empty(self) -> bool {
        self.entries.with(Vec::is_empty)
    }

    pub(super) fn items(self, stored_keys: &[String]) -> Vec<FileItem> {
        self.entries.with(|entries| {
            entries
                .iter()
                .filter_map(|entry| match &entry.status {
                    UploadStatus::Done(file) if stored_keys.contains(&file.key) => None,
                    UploadStatus::Done(file) => Some(FileItem::stored(file.clone())),
                    UploadStatus::Waiting => Some(local_item(entry, ItemState::Waiting)),
                    UploadStatus::Uploading => Some(local_item(entry, ItemState::Uploading)),
                    UploadStatus::Reconnecting => Some(FileItem {
                        can_remove: true,
                        ..local_item(entry, ItemState::Reconnecting)
                    }),
                    UploadStatus::Failed(reason) => Some(FileItem {
                        can_retry: entry.source.is_some(),
                        can_remove: true,
                        ..local_item(entry, ItemState::Failed(reason.clone()))
                    }),
                })
                .collect()
        })
    }

    fn start_waiting(self) {
        let starting = self
            .entries
            .try_update(|entries| {
                let in_flight = entries
                    .iter()
                    .filter(|entry| entry.status == UploadStatus::Uploading)
                    .count();
                let free_slots = MAX_CONCURRENT_UPLOADS.saturating_sub(in_flight);
                entries
                    .iter_mut()
                    .filter(|entry| entry.status == UploadStatus::Waiting && entry.source.is_some())
                    .take(free_slots)
                    .filter_map(begin_uploading)
                    .collect::<Vec<UploadJob>>()
            })
            .unwrap_or_default();
        for job in starting {
            self.upload(job);
        }
    }

    fn upload(self, job: UploadJob) {
        let Some(uploader) = self.uploader.try_get_value() else {
            return;
        };
        leptos::task::spawn_local(async move {
            let file = match job.shrink_edge_pixels {
                Some(longest_edge_pixels) => shrink_photo(job.file, longest_edge_pixels).await,
                None => job.file,
            };
            let too_big = self
                .rules
                .try_with_value(|rules| {
                    rules.size_rejection(byte_count(file.size()).unwrap_or_default())
                })
                .flatten();
            let outcome = match too_big {
                Some(rejection) => Err(UploadFailure::Rejected(rejection)),
                None => uploader(PendingUpload {
                    file: file.clone(),
                    upload_key: job.upload_key,
                })
                .await
                .map_err(blamed_on_the_connection),
            };
            self.finish(job.key, file, outcome);
        });
    }

    fn finish(
        self,
        key: String,
        prepared: web_sys::File,
        outcome: Result<StoredFile, UploadFailure>,
    ) {
        let reconnecting = self.entries.try_update(|entries| {
            let entry = entries.iter_mut().find(|entry| entry.key() == key)?;
            entry.settle(prepared, outcome);
            (entry.status == UploadStatus::Reconnecting).then_some(entry.reconnects)
        });
        let Some(reconnecting) = reconnecting else {
            return;
        };
        if let Some(reconnects) = reconnecting {
            self.reconnect_later(key, reconnects);
        }
        self.start_waiting();
    }
}

fn begin_uploading(entry: &mut LocalUpload) -> Option<UploadJob> {
    let file = entry.source.as_deref().cloned()?;
    entry.status = UploadStatus::Uploading;
    Some(UploadJob {
        key: entry.key().to_owned(),
        file,
        upload_key: entry.upload_key.clone(),
        shrink_edge_pixels: entry.shrink_edge_pixels,
    })
}

fn local_item(entry: &LocalUpload, state: ItemState) -> FileItem {
    FileItem {
        state,
        ..FileItem::stored(entry.stand_in.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_browser_file_size_becomes_whole_bytes() {
        assert_eq!(byte_count(2_516_582.0), Some(2_516_582));
        assert_eq!(byte_count(0.0), Some(0));
    }

    #[test]
    fn reconnecting_backs_off_until_it_settles_on_the_longest_wait() {
        let delays: Vec<u64> = (1..=6)
            .map(|reconnects| reconnect_delay(reconnects).as_secs())
            .collect();
        assert_eq!(delays, vec![2, 4, 8, 16, 30, 30]);
        assert_eq!(reconnect_delay(u32::MAX), LONGEST_RECONNECT_DELAY);
    }

    #[test]
    fn an_upload_key_is_written_as_hexadecimal() {
        assert_eq!(hexadecimal(&[0, 15, 255]), "000fff");
    }
}
