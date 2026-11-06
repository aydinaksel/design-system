use chrono::NaiveDate;
use design_system::table::{CellValue, Column, DataTable};
use design_system::{file_cell, FileGrid, FileList, PageHeader, StoredFile};
use leptos::prelude::*;

use super::section;
use crate::fake_data::swatch;

pub const SAMPLE_PDF_PATH: &str = "/gallery-files/sample-delivery-note.pdf";
const MISSING_IMAGE_PATH: &str = "/gallery-files/missing-photo.jpg";
const SAMPLE_CSV_URL: &str = "data:text/csv,crate,sku,pieces%0A1,TRVT-024,48%0A2,GRNT-076,32";

fn added(file: StoredFile, size_bytes: u64, added_by: &str, day: u32) -> StoredFile {
    StoredFile {
        size_bytes: Some(size_bytes),
        added_by: Some(added_by.to_owned()),
        added_on: NaiveDate::from_ymd_opt(2026, 11, day),
        ..file
    }
}

fn swatch_photo(key: &str, name: &str, color_hex: &str) -> StoredFile {
    StoredFile {
        content_type: Some("image/jpeg".to_owned()),
        ..StoredFile::new(key, name, swatch(color_hex).display_url)
    }
}

fn sample_documents() -> Vec<StoredFile> {
    vec![
        added(
            StoredFile::new(
                "delivery-note",
                "delivery-note-DN-10482.pdf",
                SAMPLE_PDF_PATH,
            ),
            1094,
            "Aydin",
            3,
        ),
        added(
            StoredFile::new(
                "bill-of-lading",
                "bill-of-lading-MSKU-4471029-ashworth-interiors-final-signed-copy.pdf",
                SAMPLE_PDF_PATH,
            ),
            2_516_582,
            "Stephanie",
            2,
        ),
        added(
            StoredFile::new("packing-list", "packing-list.csv", SAMPLE_CSV_URL),
            48_230,
            "Aydin",
            2,
        ),
        added(
            StoredFile::new("supplier-email", "Re: revised ETA for MSKU4471029.eml", "#"),
            182_004,
            "Mailbox",
            1,
        ),
        StoredFile::new("site-drawing", "kitchen-elevation.dwg", "#"),
        added(
            swatch_photo("crate-photo", "crate-3-loaded.jpg", "8b9a7a"),
            3_840_112,
            "Warehouse",
            3,
        ),
    ]
}

fn sample_photos() -> Vec<StoredFile> {
    vec![
        swatch_photo("travertine", "ivory-honed-travertine.jpg", "e8e0d0"),
        swatch_photo("granite", "black-polished-granite.jpg", "3a3a3a"),
        swatch_photo("slate", "green-brushed-slate.jpg", "8b9a7a"),
        StoredFile {
            content_type: Some("image/jpeg".to_owned()),
            ..StoredFile::new(
                "missing",
                "photo-that-failed-to-load.jpg",
                MISSING_IMAGE_PATH,
            )
        },
        swatch_photo("limestone", "beige-tumbled-limestone.jpg", "d8c4a4"),
        StoredFile::new("specification", "specification-sheet.pdf", SAMPLE_PDF_PATH),
    ]
}

fn file_columns() -> Vec<Column<StoredFile>> {
    vec![
        Column::custom("name", "File", file_cell)
            .with_value(|file: &StoredFile| CellValue::Text(file.name.clone()))
            .sortable()
            .searchable(),
        Column::text("kind", "Type", |file: &StoredFile| file.type_label()).filterable(),
        Column::text("added_by", "Added By", |file: &StoredFile| {
            file.added_by.clone().unwrap_or_default()
        })
        .filterable(),
        Column::text("added_on", "Added On", |file: &StoredFile| {
            file.added_on
                .map(|date| date.to_string())
                .unwrap_or_default()
        })
        .sortable(),
    ]
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FilesPage() -> impl IntoView {
    let documents = Signal::stored(sample_documents());
    let photos = Signal::stored(sample_photos());
    let no_files = Signal::stored(Vec::<StoredFile>::new());
    let table_rows = Signal::stored(Some(Ok::<_, String>(sample_documents())));

    view! {
        <main class="mx-auto max-w-screen-2xl space-y-10 px-6 py-8">
            <PageHeader title="Files" />
            {section(
                "Documents on a Record",
                view! {
                    <div class="max-w-xl">
                        <FileList files=documents label="Delivery Documents" />
                    </div>
                }
                    .into_any(),
            )}
            {section(
                "Images",
                view! { <FileGrid files=photos label="Product Photos" /> }.into_any(),
            )}
            {section(
                "No Files Yet",
                view! {
                    <div class="max-w-xl">
                        <FileList files=no_files label="Delivery Documents" />
                    </div>
                }
                    .into_any(),
            )}
            {section(
                "Files in a Table",
                view! {
                    <DataTable
                        rows=table_rows
                        columns=file_columns()
                        empty_message="No files found."
                    />
                }
                    .into_any(),
            )}
        </main>
    }
    .into_any()
}
