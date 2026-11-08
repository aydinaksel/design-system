use leptos::prelude::*;
use send_wrapper::SendWrapper;

use super::display::{FileItem, ItemState};
use super::upload::{FileUploader, UploadRules};
use super::StoredFile;
use crate::element_key::ElementKey;

const MAX_CONCURRENT_UPLOADS: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
enum UploadStatus {
    Waiting,
    Uploading,
    Failed(String),
    Done(StoredFile),
}

#[derive(Clone)]
struct LocalUpload {
    stand_in: StoredFile,
    source: Option<SendWrapper<web_sys::File>>,
    preview_url: Option<String>,
    status: UploadStatus,
}

impl LocalUpload {
    fn key(&self) -> &str {
        &self.stand_in.key
    }

    fn is_unfinished(&self) -> bool {
        matches!(self.status, UploadStatus::Waiting | UploadStatus::Uploading)
    }
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
        Self {
            entries,
            uploader: StoredValue::new(uploader),
            rules: StoredValue::new(rules),
        }
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
                stand_in,
                source: rejection.is_none().then(|| SendWrapper::new(file)),
                preview_url,
                status: rejection.map_or(UploadStatus::Waiting, UploadStatus::Failed),
            });
        }
        self.entries.update(|entries| entries.extend(added));
        self.start_waiting();
    }

    pub(super) fn retry(self, key: &str) {
        self.entries.update(|entries| {
            let retryable = entries
                .iter_mut()
                .find(|entry| entry.key() == key && entry.source.is_some());
            if let Some(entry) = retryable {
                entry.status = UploadStatus::Waiting;
            }
        });
        self.start_waiting();
    }

    pub(super) fn dismiss(self, key: &str) -> bool {
        let mut dismissed = false;
        self.entries.update(|entries| {
            entries.retain(|entry| {
                let keep = entry.key() != key || entry.is_unfinished();
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
            .with_untracked(|entries| entries.iter().any(LocalUpload::is_unfinished))
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
        let mut starting: Vec<(String, web_sys::File)> = Vec::new();
        self.entries.update(|entries| {
            let in_flight = entries
                .iter()
                .filter(|entry| entry.status == UploadStatus::Uploading)
                .count();
            let free_slots = MAX_CONCURRENT_UPLOADS.saturating_sub(in_flight);
            let waiting = entries
                .iter_mut()
                .filter(|entry| entry.status == UploadStatus::Waiting)
                .take(free_slots);
            for entry in waiting {
                if let Some(source) = &entry.source {
                    entry.status = UploadStatus::Uploading;
                    starting.push((entry.key().to_owned(), (**source).clone()));
                }
            }
        });
        for (key, file) in starting {
            self.upload(key, file);
        }
    }

    fn upload(self, key: String, file: web_sys::File) {
        let upload = self.uploader.with_value(|uploader| uploader(file));
        leptos::task::spawn_local(async move {
            let status = match upload.await {
                Ok(stored) => UploadStatus::Done(stored),
                Err(reason) => UploadStatus::Failed(reason),
            };
            let finished = self.entries.try_update(|entries| {
                if let Some(entry) = entries.iter_mut().find(|entry| entry.key() == key) {
                    if matches!(status, UploadStatus::Done(_)) {
                        revoke_preview(entry.preview_url.take().as_deref());
                    }
                    entry.status = status;
                }
            });
            if finished.is_some() {
                self.start_waiting();
            }
        });
    }
}

fn local_item(entry: &LocalUpload, state: ItemState) -> FileItem {
    FileItem {
        state,
        ..FileItem::stored(entry.stand_in.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::byte_count;

    #[test]
    fn a_browser_file_size_becomes_whole_bytes() {
        assert_eq!(byte_count(2_516_582.0), Some(2_516_582));
        assert_eq!(byte_count(0.0), Some(0));
    }
}
