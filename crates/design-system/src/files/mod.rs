use chrono::NaiveDate;

use crate::formatting::format_file_size;
use crate::icons::LucideGlyph;

mod arrange;
mod display;
mod drop_target;
mod photo_shrinking;
mod removal;
mod section;
mod toolbar;
mod upload;
mod upload_queue;
mod viewer;

pub use arrange::{file_arranger, ArrangeFuture, Arrangement, FileArranger, FileCategory};
pub use display::{file_cell, FileGrid, FileLayout, FileList};
pub use section::FileSection;
pub use upload::{
    file_remover, file_uploader, FileRemover, FileUploader, PendingUpload, RemoveFuture,
    UploadFailure, UploadFuture, UploadRules,
};
pub use viewer::FileViewer;

const DATE_FORMAT: &str = "%-d %b %Y";
const DETAIL_SEPARATOR: &str = " · ";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredFile {
    pub key: String,
    pub name: String,
    pub url: String,
    pub preview_url: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub added_by: Option<String>,
    pub added_on: Option<NaiveDate>,
    pub category: Option<String>,
}

impl StoredFile {
    #[must_use]
    pub fn new(key: impl Into<String>, name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            url: url.into(),
            preview_url: None,
            content_type: None,
            size_bytes: None,
            added_by: None,
            added_on: None,
            category: None,
        }
    }

    #[must_use]
    pub fn kind(&self) -> FileKind {
        FileKind::of(&self.name, self.content_type.as_deref())
    }

    #[must_use]
    pub fn opens_in_viewer(&self) -> bool {
        self.kind().opens_in_viewer()
    }

    fn image_url(&self) -> Option<&str> {
        (self.kind() == FileKind::Image).then(|| self.preview_url.as_deref().unwrap_or(&self.url))
    }

    #[must_use]
    pub fn type_label(&self) -> String {
        let kind = self.kind();
        if kind != FileKind::Other {
            return kind.label().to_owned();
        }
        name_parts(&self.name)
            .1
            .map_or_else(|| kind.label().to_owned(), str::to_uppercase)
    }

    fn tile_details(&self) -> String {
        let type_label = (self.kind() != FileKind::Image).then(|| self.type_label());
        [type_label, self.size_bytes.map(format_file_size)]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(DETAIL_SEPARATOR)
    }

    fn details(&self) -> String {
        let type_label = Some(self.type_label());
        let size = self.size_bytes.map(format_file_size);
        let added_on = self
            .added_on
            .map(|date| date.format(DATE_FORMAT).to_string());
        [type_label, size, self.added_by.clone(), added_on]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(DETAIL_SEPARATOR)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Image,
    Pdf,
    Spreadsheet,
    Document,
    Email,
    Archive,
    Video,
    Audio,
    Other,
}

impl FileKind {
    #[must_use]
    pub fn of(name: &str, content_type: Option<&str>) -> Self {
        content_type
            .and_then(Self::from_content_type)
            .unwrap_or_else(|| Self::from_extension(name_parts(name).1))
    }

    fn from_content_type(content_type: &str) -> Option<Self> {
        let essence = content_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let (family, subtype) = essence.split_once('/')?;
        match (family, subtype) {
            ("image", _) => Some(Self::Image),
            ("video", _) => Some(Self::Video),
            ("audio", _) => Some(Self::Audio),
            ("application", "pdf") => Some(Self::Pdf),
            ("message", "rfc822") | ("application", "vnd.ms-outlook") => Some(Self::Email),
            ("text", "csv")
            | (
                "application",
                "vnd.ms-excel" | "vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ) => Some(Self::Spreadsheet),
            ("text", "plain")
            | (
                "application",
                "msword" | "rtf" | "vnd.openxmlformats-officedocument.wordprocessingml.document",
            ) => Some(Self::Document),
            ("application", "zip" | "gzip" | "x-7z-compressed" | "x-tar") => Some(Self::Archive),
            _ => None,
        }
    }

    const RECOGNIZED: [Self; 8] = [
        Self::Image,
        Self::Pdf,
        Self::Spreadsheet,
        Self::Document,
        Self::Email,
        Self::Archive,
        Self::Video,
        Self::Audio,
    ];

    fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Image => &["jpg", "jpeg", "png", "gif", "webp", "avif", "heic", "svg"],
            Self::Pdf => &["pdf"],
            Self::Spreadsheet => &["csv", "xls", "xlsx", "ods"],
            Self::Document => &["txt", "doc", "docx", "rtf", "odt"],
            Self::Email => &["eml", "msg"],
            Self::Archive => &["zip", "gz", "7z", "tar"],
            Self::Video => &["mp4", "mov", "webm"],
            Self::Audio => &["mp3", "wav", "m4a"],
            Self::Other => &[],
        }
    }

    fn content_type_pattern(self) -> Option<&'static str> {
        match self {
            Self::Image => Some("image/*"),
            Self::Pdf => Some("application/pdf"),
            Self::Video => Some("video/*"),
            Self::Audio => Some("audio/*"),
            Self::Spreadsheet | Self::Document | Self::Email | Self::Archive | Self::Other => None,
        }
    }

    fn from_extension(extension: Option<&str>) -> Self {
        let extension = extension.unwrap_or_default().to_ascii_lowercase();
        Self::RECOGNIZED
            .into_iter()
            .find(|kind| kind.extensions().contains(&extension.as_str()))
            .unwrap_or(Self::Other)
    }

    #[must_use]
    pub fn opens_in_viewer(self) -> bool {
        matches!(self, Self::Image | Self::Pdf)
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Pdf => "PDF",
            Self::Spreadsheet => "Spreadsheet",
            Self::Document => "Document",
            Self::Email => "Email",
            Self::Archive => "Archive",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Other => "File",
        }
    }

    fn plural_label(self) -> &'static str {
        match self {
            Self::Image => "images",
            Self::Pdf => "PDFs",
            Self::Spreadsheet => "spreadsheets",
            Self::Document => "documents",
            Self::Email => "emails",
            Self::Archive => "archives",
            Self::Video => "videos",
            Self::Audio => "audio files",
            Self::Other => "files",
        }
    }

    fn glyph(self) -> LucideGlyph {
        match self {
            Self::Image => LucideGlyph::FileImage,
            Self::Pdf | Self::Document => LucideGlyph::FileText,
            Self::Spreadsheet => LucideGlyph::FileSpreadsheet,
            Self::Email => LucideGlyph::Mail,
            Self::Archive => LucideGlyph::FileArchive,
            Self::Video => LucideGlyph::FileVideoCamera,
            Self::Audio => LucideGlyph::FileMusic,
            Self::Other => LucideGlyph::File,
        }
    }

    fn icon_color_class(self) -> &'static str {
        match self {
            Self::Pdf => "text-red-600",
            Self::Spreadsheet => "text-green-700",
            Self::Document => "text-blue-600",
            Self::Image | Self::Email | Self::Archive | Self::Video | Self::Audio | Self::Other => {
                "text-gray-500"
            }
        }
    }
}

const KEPT_STEM_ENDING_CHARACTERS: usize = 6;

fn stem_head_and_ending(stem: &str) -> (&str, &str) {
    let ending_start = stem
        .char_indices()
        .rev()
        .take(KEPT_STEM_ENDING_CHARACTERS)
        .last()
        .map_or(0, |(index, _)| index);
    stem.split_at(ending_start)
}

fn name_parts(name: &str) -> (&str, Option<&str>) {
    match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() && !extension.is_empty() => {
            (stem, Some(extension))
        }
        _ => (name, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_content_type_decides_the_kind_before_the_extension() {
        assert_eq!(FileKind::of("scan", Some("application/pdf")), FileKind::Pdf);
        assert_eq!(
            FileKind::of("photo.pdf", Some("image/jpeg; charset=binary")),
            FileKind::Image
        );
    }

    #[test]
    fn the_extension_decides_the_kind_without_a_known_content_type() {
        assert_eq!(
            FileKind::of("Packing List.XLSX", None),
            FileKind::Spreadsheet
        );
        assert_eq!(
            FileKind::of("drawing.dwg", Some("application/octet-stream")),
            FileKind::Other
        );
        assert_eq!(FileKind::of("README", None), FileKind::Other);
    }

    #[test]
    fn only_images_and_pdfs_open_in_the_viewer() {
        assert!(FileKind::Image.opens_in_viewer());
        assert!(FileKind::Pdf.opens_in_viewer());
        assert!(!FileKind::Spreadsheet.opens_in_viewer());
    }

    #[test]
    fn a_name_splits_into_stem_and_extension_at_the_last_dot() {
        assert_eq!(
            name_parts("bill.of.lading.pdf"),
            ("bill.of.lading", Some("pdf"))
        );
        assert_eq!(name_parts(".env"), (".env", None));
        assert_eq!(name_parts("notes."), ("notes.", None));
    }

    #[test]
    fn a_stem_keeps_its_ending_so_similar_names_stay_distinct() {
        assert_eq!(stem_head_and_ending("IMG_4471"), ("IM", "G_4471"));
        assert_eq!(stem_head_and_ending("scan"), ("", "scan"));
        assert_eq!(stem_head_and_ending("café-crème"), ("café", "-crème"));
    }

    #[test]
    fn details_list_what_is_known_in_order() {
        let mut file = StoredFile::new("1", "drawing.dwg", "/drawing.dwg");
        assert_eq!(file.details(), "DWG");
        file.size_bytes = Some(2_516_582);
        file.added_by = Some("Aydin".to_owned());
        file.added_on = NaiveDate::from_ymd_opt(2026, 11, 3);
        assert_eq!(file.details(), "DWG · 2.4 MB · Aydin · 3 Nov 2026");
    }
}
