use std::sync::atomic::{AtomicU64, Ordering};

use chrono::NaiveDate;
use design_system::table::{CellValue, Column, DataTable};
use design_system::{
    file_cell, file_remover, file_uploader, FileGrid, FileKind, FileLayout, FileList, FileRemover,
    FileSection, FileUploader, PageHeader, StoredFile, UploadRules,
};
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
            {uploading_sections()}
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

#[cfg(feature = "ssr")]
const SIMULATED_UPLOAD_MILLISECONDS: u64 = 1200;
#[cfg(feature = "ssr")]
const SIMULATED_REMOVAL_MILLISECONDS: u64 = 600;
const TEN_MEGABYTES: u64 = 10_485_760;

static NEXT_UPLOADED_NUMBER: AtomicU64 = AtomicU64::new(0);

#[server]
pub async fn simulate_upload(name: String) -> Result<(), ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_millis(
        SIMULATED_UPLOAD_MILLISECONDS,
    ))
    .await;
    if name.contains("fail") {
        return Err(ServerFnError::new("The server turned this file away."));
    }
    Ok(())
}

#[server]
pub async fn simulate_removal(name: String) -> Result<(), ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_millis(
        SIMULATED_REMOVAL_MILLISECONDS,
    ))
    .await;
    if name.contains("locked") {
        return Err(ServerFnError::new("It is attached to an open invoice."));
    }
    Ok(())
}

fn server_message(error: ServerFnError) -> String {
    match error {
        ServerFnError::ServerError(message) => message,
        other => other.to_string(),
    }
}

fn uploaded_copy(file: &web_sys::File) -> StoredFile {
    let number = NEXT_UPLOADED_NUMBER.fetch_add(1, Ordering::Relaxed);
    let url = web_sys::Url::create_object_url_with_blob(file).unwrap_or_default();
    StoredFile {
        content_type: Some(file.type_()).filter(|content_type| !content_type.is_empty()),
        size_bytes: format!("{:.0}", file.size()).parse().ok(),
        added_by: Some("You".to_owned()),
        added_on: Some(chrono::Local::now().date_naive()),
        ..StoredFile::new(format!("uploaded-{number}"), file.name(), url)
    }
}

fn gallery_uploader() -> FileUploader {
    file_uploader(|file: web_sys::File| async move {
        simulate_upload(file.name()).await.map_err(server_message)?;
        Ok::<_, String>(uploaded_copy(&file))
    })
}

fn gallery_remover() -> FileRemover {
    file_remover(|file: StoredFile| async move {
        simulate_removal(file.name).await.map_err(server_message)
    })
}

fn attached_documents() -> Vec<StoredFile> {
    vec![
        added(
            StoredFile::new(
                "attached-delivery-note",
                "delivery-note-DN-10482.pdf",
                SAMPLE_PDF_PATH,
            ),
            1094,
            "Aydin",
            3,
        ),
        added(
            StoredFile::new(
                "attached-locked",
                "locked-supplier-invoice.pdf",
                SAMPLE_PDF_PATH,
            ),
            48_230,
            "Stephanie",
            2,
        ),
    ]
}

fn uploading_sections() -> AnyView {
    let documents = Signal::stored(attached_documents());
    let no_photos = Signal::stored(Vec::<StoredFile>::new());
    let no_statement = Signal::stored(Vec::<StoredFile>::new());
    let locked = Signal::stored(true);
    view! {
        {section(
            "Upload Documents",
            view! {
                <div class="max-w-xl">
                    <FileSection
                        files=documents
                        label="Delivery Documents"
                        rules=UploadRules::accepting([FileKind::Pdf, FileKind::Image])
                            .up_to(TEN_MEGABYTES)
                        upload=gallery_uploader()
                        remove=gallery_remover()
                    />
                </div>
            }
                .into_any(),
        )}
        {section(
            "Upload Photos",
            view! {
                <FileSection
                    files=no_photos
                    label="Delivery Photos"
                    layout=FileLayout::Grid
                    rules=UploadRules::accepting([FileKind::Image]).up_to(TEN_MEGABYTES)
                    upload=gallery_uploader()
                    remove=gallery_remover()
                />
            }
                .into_any(),
        )}
        {section(
            "Upload One File",
            view! {
                <div class="max-w-xl">
                    <FileSection
                        files=no_statement
                        label="Bank Statement"
                        rules=UploadRules::accepting([FileKind::Spreadsheet]).one_file()
                        upload=gallery_uploader()
                    />
                </div>
            }
                .into_any(),
        )}
        {section(
            "Uploads Disabled",
            view! {
                <div class="max-w-xl">
                    <FileSection
                        files=no_statement
                        label="Locked Documents"
                        upload=gallery_uploader()
                        disabled=locked
                    />
                </div>
            }
                .into_any(),
        )}
    }
    .into_any()
}
