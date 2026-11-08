use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use leptos::prelude::ServerFnError;

use super::{FileKind, StoredFile};
use crate::formatting::{format_file_size, format_file_size_with_tenths};

const SHRINKABLE_CONTENT_TYPES: [&str; 4] =
    ["image/jpeg", "image/webp", "image/heic", "image/heif"];

#[derive(Clone, Debug)]
pub struct PendingUpload {
    pub file: web_sys::File,
    pub upload_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UploadFailure {
    Rejected(String),
    Interrupted(String),
}

impl From<String> for UploadFailure {
    fn from(reason: String) -> Self {
        Self::Rejected(reason)
    }
}

impl From<ServerFnError> for UploadFailure {
    fn from(error: ServerFnError) -> Self {
        match error {
            ServerFnError::Request(reason) | ServerFnError::Response(reason) => {
                Self::Interrupted(reason)
            }
            ServerFnError::ServerError(reason) => Self::Rejected(reason),
            other => Self::Rejected(other.to_string()),
        }
    }
}

pub type UploadFuture = Pin<Box<dyn Future<Output = Result<StoredFile, UploadFailure>>>>;
pub type RemoveFuture = Pin<Box<dyn Future<Output = Result<(), String>>>>;

pub type FileUploader = Arc<dyn Fn(PendingUpload) -> UploadFuture + Send + Sync>;
pub type FileRemover = Arc<dyn Fn(StoredFile) -> RemoveFuture + Send + Sync>;

pub fn file_uploader<UploadFn, Upload, UploadError>(upload: UploadFn) -> FileUploader
where
    UploadFn: Fn(PendingUpload) -> Upload + Send + Sync + 'static,
    Upload: Future<Output = Result<StoredFile, UploadError>> + 'static,
    UploadError: Into<UploadFailure>,
{
    Arc::new(move |pending| {
        let upload = upload(pending);
        Box::pin(async move { upload.await.map_err(Into::into) })
    })
}

pub fn file_remover<RemoveFn, Remove, RemoveError>(remove: RemoveFn) -> FileRemover
where
    RemoveFn: Fn(StoredFile) -> Remove + Send + Sync + 'static,
    Remove: Future<Output = Result<(), RemoveError>> + 'static,
    RemoveError: ToString,
{
    Arc::new(move |file| {
        let removal = remove(file);
        Box::pin(async move { removal.await.map_err(|error| error.to_string()) })
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadRules {
    accepted: Vec<FileKind>,
    max_size_bytes: Option<u64>,
    multiple: bool,
    shrink_edge_pixels: Option<u32>,
}

impl Default for UploadRules {
    fn default() -> Self {
        Self::any_file()
    }
}

impl UploadRules {
    #[must_use]
    pub fn any_file() -> Self {
        Self {
            accepted: Vec::new(),
            max_size_bytes: None,
            multiple: true,
            shrink_edge_pixels: None,
        }
    }

    #[must_use]
    pub fn accepting(kinds: impl IntoIterator<Item = FileKind>) -> Self {
        Self {
            accepted: kinds.into_iter().collect(),
            ..Self::any_file()
        }
    }

    #[must_use]
    pub fn up_to(self, max_size_bytes: u64) -> Self {
        Self {
            max_size_bytes: Some(max_size_bytes),
            ..self
        }
    }

    #[must_use]
    pub fn one_file(self) -> Self {
        Self {
            multiple: false,
            ..self
        }
    }

    #[must_use]
    pub fn shrinking_photos_to(self, longest_edge_pixels: u32) -> Self {
        Self {
            shrink_edge_pixels: Some(longest_edge_pixels),
            ..self
        }
    }

    pub(super) fn shrink_edge_pixels(&self, content_type: Option<&str>) -> Option<u32> {
        let shrinkable = content_type.is_some_and(|content_type| {
            SHRINKABLE_CONTENT_TYPES
                .iter()
                .any(|shrinkable| content_type.eq_ignore_ascii_case(shrinkable))
        });
        self.shrink_edge_pixels.filter(|_| shrinkable)
    }

    pub(super) fn accepts_photos(&self) -> bool {
        self.accepts_any_kind() || self.accepted.contains(&FileKind::Image)
    }

    pub(super) fn multiple(&self) -> bool {
        self.multiple
    }

    fn accepts_any_kind(&self) -> bool {
        self.accepted.is_empty() || self.accepted.contains(&FileKind::Other)
    }

    pub(super) fn rejection(
        &self,
        name: &str,
        content_type: Option<&str>,
        size_bytes: u64,
    ) -> Option<String> {
        let kind = FileKind::of(name, content_type);
        if !self.accepts_any_kind() && !self.accepted.contains(&kind) {
            return Some(format!(
                "Only {} can be attached here.",
                self.kinds_phrase()
            ));
        }
        if self.shrink_edge_pixels(content_type).is_some() {
            return None;
        }
        self.size_rejection(size_bytes)
    }

    pub(super) fn size_rejection(&self, size_bytes: u64) -> Option<String> {
        self.max_size_bytes
            .filter(|max_size_bytes| size_bytes > *max_size_bytes)
            .map(|max_size_bytes| {
                let limit = format_file_size(max_size_bytes);
                let mut size = format_file_size(size_bytes);
                if size == limit {
                    size = format_file_size_with_tenths(size_bytes);
                }
                format!("This file is {size}, over the {limit} limit.")
            })
    }

    pub(super) fn too_many_message() -> &'static str {
        "Only one file can be attached at a time."
    }

    fn kinds_phrase(&self) -> String {
        let labels: Vec<&str> = self
            .accepted
            .iter()
            .map(|kind| kind.plural_label())
            .collect();
        match labels.split_last() {
            None => "files".to_owned(),
            Some((last, [])) => (*last).to_owned(),
            Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
        }
    }

    pub(super) fn hint(&self) -> String {
        let kinds = if self.accepts_any_kind() {
            "Any file".to_owned()
        } else {
            let phrase = self.kinds_phrase();
            let mut letters = phrase.chars();
            let capitalized: String = letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect())
                .unwrap_or_default();
            if self.accepted.len() == 1 {
                format!("{capitalized} only")
            } else {
                capitalized
            }
        };
        let each = if self.multiple { " each" } else { "" };
        match self.max_size_bytes {
            Some(max_size_bytes) => {
                format!("{kinds}, up to {}{each}.", format_file_size(max_size_bytes))
            }
            None => format!("{kinds}."),
        }
    }

    pub(super) fn accept_attribute(&self) -> Option<String> {
        if self.accepts_any_kind() {
            return None;
        }
        let patterns: Vec<String> = self
            .accepted
            .iter()
            .flat_map(|kind| {
                kind.content_type_pattern()
                    .map(str::to_owned)
                    .into_iter()
                    .chain(
                        kind.extensions()
                            .iter()
                            .map(|extension| format!(".{extension}")),
                    )
            })
            .collect();
        Some(patterns.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEN_MEGABYTES: u64 = 10_485_760;

    #[test]
    fn the_hint_reads_from_the_same_rules_that_are_enforced() {
        assert_eq!(UploadRules::any_file().hint(), "Any file.");
        assert_eq!(
            UploadRules::accepting([FileKind::Pdf])
                .up_to(TEN_MEGABYTES)
                .hint(),
            "PDFs only, up to 10 MB each."
        );
        assert_eq!(
            UploadRules::accepting([FileKind::Pdf, FileKind::Image])
                .up_to(TEN_MEGABYTES)
                .one_file()
                .hint(),
            "PDFs or images, up to 10 MB."
        );
    }

    #[test]
    fn a_file_of_the_wrong_kind_is_rejected_with_what_is_accepted() {
        let rules = UploadRules::accepting([FileKind::Pdf, FileKind::Image, FileKind::Spreadsheet]);
        assert_eq!(
            rules.rejection("notes.docx", None, 10),
            Some("Only PDFs, images or spreadsheets can be attached here.".to_owned())
        );
        assert_eq!(rules.rejection("scan.pdf", None, 10), None);
    }

    #[test]
    fn a_file_over_the_limit_is_rejected_with_both_sizes() {
        let rules = UploadRules::any_file().up_to(TEN_MEGABYTES);
        assert_eq!(
            rules.rejection("photo.jpg", Some("image/jpeg"), 13_002_342),
            Some("This file is 12 MB, over the 10 MB limit.".to_owned())
        );
        assert_eq!(
            rules.rejection("photo.jpg", None, 10_905_190),
            Some("This file is 10.4 MB, over the 10 MB limit.".to_owned())
        );
        assert_eq!(rules.rejection("photo.jpg", None, TEN_MEGABYTES), None);
    }

    #[test]
    fn a_photo_that_will_be_shrunk_is_measured_after_shrinking() {
        let rules = UploadRules::accepting([FileKind::Image])
            .up_to(TEN_MEGABYTES)
            .shrinking_photos_to(2048);
        assert_eq!(
            rules.rejection("camera.jpg", Some("image/jpeg"), 13_002_342),
            None
        );
        assert_eq!(rules.shrink_edge_pixels(Some("image/JPEG")), Some(2048));
        assert_eq!(rules.shrink_edge_pixels(Some("image/png")), None);
        assert!(rules
            .rejection("diagram.png", Some("image/png"), 13_002_342)
            .is_some());
    }

    #[test]
    fn only_a_lost_connection_is_worth_retrying() {
        assert_eq!(
            UploadFailure::from(ServerFnError::Request("offline".to_owned())),
            UploadFailure::Interrupted("offline".to_owned())
        );
        assert_eq!(
            UploadFailure::from(ServerFnError::new("Too big.")),
            UploadFailure::Rejected("Too big.".to_owned())
        );
    }

    #[test]
    fn the_picker_filter_names_every_accepted_kind() {
        assert_eq!(UploadRules::any_file().accept_attribute(), None);
        assert_eq!(
            UploadRules::accepting([FileKind::Pdf]).accept_attribute(),
            Some("application/pdf,.pdf".to_owned())
        );
    }
}
