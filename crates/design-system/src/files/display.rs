use leptos::html::Img;
use leptos::prelude::*;

use super::viewer::FileViewer;
use super::FileKind;
use super::{name_parts, stem_head_and_ending, StoredFile};
use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::empty_state::EmptyState;
use crate::icons::{self, LucideGlyph};
use crate::tooltip::{Tooltip, TooltipAlign};

const ROW_CLASS: &str = "relative flex min-w-0 items-center gap-3 rounded-surface border border-gray-200 bg-white p-2 transition-colors hover:border-gray-300";
const TILE_CLASS: &str = "relative flex min-w-0 flex-col overflow-hidden rounded-surface border border-gray-200 bg-white transition-colors hover:border-gray-300";
const OPEN_OVERLAY_CLASS: &str =
    "absolute inset-0 z-0 cursor-pointer rounded-surface-inner focus-ring";
const ROW_MEDIA_CLASS: &str =
    "flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-gray-100";
const TILE_MEDIA_CLASS: &str =
    "flex aspect-[4/3] w-full items-center justify-center overflow-hidden bg-gray-100";
const GRID_CLASS: &str = "grid grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] gap-3";
const NAME_CLASS: &str = "font-medium text-gray-900";
const DEFAULT_EMPTY_MESSAGE: &str = "No files yet.";

#[derive(Clone, Copy)]
enum MediaSize {
    Row,
    Tile,
}

impl MediaSize {
    fn frame_class(self) -> &'static str {
        match self {
            Self::Row => ROW_MEDIA_CLASS,
            Self::Tile => TILE_MEDIA_CLASS,
        }
    }

    fn icon_class(self) -> &'static str {
        match self {
            Self::Row => "h-5 w-5",
            Self::Tile => "h-8 w-8",
        }
    }
}

fn kind_icon(kind: FileKind, size: MediaSize) -> AnyView {
    icons::icon(
        kind.glyph(),
        format!("{} {}", size.icon_class(), kind.icon_color_class()),
    )
}

#[cfg(feature = "hydrate")]
fn failed_before_hydration(image: &web_sys::HtmlImageElement) -> bool {
    image.complete() && image.natural_width() == 0
}

#[cfg(not(feature = "hydrate"))]
fn failed_before_hydration(_image: &web_sys::HtmlImageElement) -> bool {
    false
}

fn media_view(file: &StoredFile, size: MediaSize) -> AnyView {
    let kind = file.kind();
    let Some(image_url) = file.image_url().map(str::to_owned) else {
        return view! {
            <span class=size.frame_class() aria-hidden="true">
                {kind_icon(kind, size)}
            </span>
        }
        .into_any();
    };
    let broken = RwSignal::new(false);
    let image: NodeRef<Img> = NodeRef::new();
    Effect::new(move |_| {
        if image
            .get()
            .is_some_and(|element| failed_before_hydration(&element))
        {
            broken.set(true);
        }
    });
    view! {
        <span class=size.frame_class() aria-hidden="true">
            <Show when=move || !broken.get() fallback=move || kind_icon(kind, size)>
                <img
                    node_ref=image
                    src=image_url.clone()
                    alt=""
                    loading="lazy"
                    class="h-full w-full object-cover"
                    on:error=move |_| broken.set(true)
                />
            </Show>
        </span>
    }
    .into_any()
}

fn name_view(file: &StoredFile, text_class: &'static str) -> AnyView {
    let (stem, extension) = name_parts(&file.name);
    let (head, ending) = stem_head_and_ending(stem);
    let head = head.to_owned();
    let kept_ending = format!(
        "{ending}{}",
        extension
            .map(|extension| format!(".{extension}"))
            .unwrap_or_default(),
    );
    view! {
        <span class=format!("flex min-w-0 {text_class}") title=file.name.clone()>
            <span class="truncate">{head}</span>
            <span class="shrink-0 whitespace-pre">{kept_ending}</span>
        </span>
    }
    .into_any()
}

fn open_overlay(file: &StoredFile, open_key: RwSignal<Option<String>>) -> AnyView {
    if file.opens_in_viewer() {
        let key = file.key.clone();
        let label = format!("Open {}", file.name);
        return view! {
            <button
                type="button"
                class=OPEN_OVERLAY_CLASS
                aria-label=label
                on:click=move |_| open_key.set(Some(key.clone()))
            ></button>
        }
        .into_any();
    }
    let label = format!("Download {}", file.name);
    view! {
        <a
            href=file.url.clone()
            download=file.name.clone()
            class=OPEN_OVERLAY_CLASS
            aria-label=label
        ></a>
    }
    .into_any()
}

fn download_button(file: &StoredFile) -> AnyView {
    let label = format!("Download {}", file.name);
    let url = file.url.clone();
    let name = file.name.clone();
    view! {
        <span class="relative z-10 shrink-0">
            <Tooltip label="Download" align=TooltipAlign::End>
                <a
                    href=url
                    download=name
                    class=icon_button_class(ButtonTone::Ghost, ButtonSize::Compact)
                    aria-label=label
                >
                    {icons::icon(LucideGlyph::Download, "h-4 w-4")}
                </a>
            </Tooltip>
        </span>
    }
    .into_any()
}

fn file_row(file: &StoredFile, open_key: RwSignal<Option<String>>) -> AnyView {
    view! {
        <li class=ROW_CLASS>
            {open_overlay(file, open_key)} {media_view(file, MediaSize::Row)}
            <span class="pointer-events-none relative min-w-0 flex-1 text-sm">
                {name_view(file, NAME_CLASS)}
                <span class="block truncate text-xs text-gray-500">{file.details()}</span>
            </span> {download_button(file)}
        </li>
    }
    .into_any()
}

fn tile_details(file: &StoredFile) -> Option<String> {
    let details = if file.kind() == FileKind::Image {
        file.details_without_type()
    } else {
        file.details()
    };
    (!details.is_empty()).then_some(details)
}

fn file_tile(file: &StoredFile, open_key: RwSignal<Option<String>>) -> AnyView {
    let details = tile_details(file).map(
        |details| view! { <span class="block truncate text-xs text-gray-500">{details}</span> },
    );
    view! {
        <li class=TILE_CLASS>
            {open_overlay(file, open_key)} {media_view(file, MediaSize::Tile)}
            <span class="pointer-events-none relative min-w-0 p-2 text-sm">
                {name_view(file, NAME_CLASS)} {details}
            </span>
        </li>
    }
    .into_any()
}

fn file_collection(
    files: Signal<Vec<StoredFile>>,
    label: String,
    empty_message: String,
    list_class: &'static str,
    item: fn(&StoredFile, RwSignal<Option<String>>) -> AnyView,
) -> AnyView {
    let open_key: RwSignal<Option<String>> = RwSignal::new(None);
    let items = move || {
        if files.with(Vec::is_empty) {
            return view! { <EmptyState title=empty_message.clone() /> }.into_any();
        }
        view! {
            <ul aria-label=label.clone() class=list_class>
                <For
                    each=move || files.get()
                    key=|file| file.key.clone()
                    children=move |file| item(&file, open_key)
                />
            </ul>
        }
        .into_any()
    };
    view! {
        {items}
        <FileViewer files open_key />
    }
    .into_any()
}

#[component]
pub fn FileList(
    #[prop(into)] files: Signal<Vec<StoredFile>>,
    #[prop(into)] label: String,
    #[prop(optional, into)] empty_message: Option<String>,
) -> impl IntoView {
    let empty_message = empty_message.unwrap_or_else(|| DEFAULT_EMPTY_MESSAGE.to_owned());
    file_collection(files, label, empty_message, "flex flex-col gap-2", file_row)
}

#[component]
pub fn FileGrid(
    #[prop(into)] files: Signal<Vec<StoredFile>>,
    #[prop(into)] label: String,
    #[prop(optional, into)] empty_message: Option<String>,
) -> impl IntoView {
    let empty_message = empty_message.unwrap_or_else(|| DEFAULT_EMPTY_MESSAGE.to_owned());
    file_collection(files, label, empty_message, GRID_CLASS, file_tile)
}

#[must_use]
pub fn file_cell(file: &StoredFile) -> AnyView {
    let kind = file.kind();
    let icon = icons::icon(
        kind.glyph(),
        format!("h-4 w-4 shrink-0 {}", kind.icon_color_class()),
    );
    let link = if file.opens_in_viewer() {
        view! {
            <a
                href=file.url.clone()
                target="_blank"
                rel="noopener"
                class="min-w-0 text-blue-600 hover:text-blue-800 hover:underline"
            >
                {name_view(file, "")}
            </a>
        }
        .into_any()
    } else {
        view! {
            <a
                href=file.url.clone()
                download=file.name.clone()
                class="min-w-0 text-blue-600 hover:text-blue-800 hover:underline"
            >
                {name_view(file, "")}
            </a>
        }
        .into_any()
    };
    view! { <span class="flex max-w-xs min-w-0 items-center gap-2">{icon} {link}</span> }.into_any()
}
