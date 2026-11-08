use leptos::ev::{FocusEvent, MouseEvent};
use leptos::html::{Img, Span};
use leptos::prelude::*;

use super::viewer::FileViewer;
use super::DETAIL_SEPARATOR;
use super::{name_parts, stem_head_and_ending, FileKind, StoredFile};
use crate::badge::badge;
use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::empty_state::EmptyState;
use crate::icons::{self, LucideGlyph};
use crate::tone::Tone;
use crate::tooltip::{OverflowTooltip, Tooltip, TooltipAlign};

const ROW_CLASS: &str = "relative flex min-w-0 items-center gap-3 rounded-surface border bg-white p-2 transition-colors";
const TILE_CLASS: &str =
    "relative flex min-w-0 flex-col overflow-hidden rounded-surface border bg-white transition-colors";
const SETTLED_BORDER_CLASS: &str = "border-gray-200 hover:border-gray-300";
const FAILED_BORDER_CLASS: &str = "border-red-200 bg-red-50/40";
const OPEN_OVERLAY_CLASS: &str =
    "absolute inset-0 z-0 cursor-pointer rounded-surface-inner focus-ring";
const ROW_MEDIA_CLASS: &str = "pointer-events-none relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-gray-100";
const TILE_MEDIA_CLASS: &str =
    "pointer-events-none relative flex aspect-[4/3] w-full items-center justify-center overflow-hidden bg-gray-100";
const BUSY_OVERLAY_CLASS: &str =
    "absolute inset-0 flex items-center justify-center bg-white/60 text-gray-500";
const GRID_CLASS: &str =
    "grid grid-cols-2 gap-3 sm:grid-cols-[repeat(auto-fill,minmax(10rem,1fr))]";
const LIST_CLASS: &str = "flex flex-col gap-2";
pub(super) const NAME_CLASS: &str = "font-medium text-gray-900";
const DETAIL_CLASS: &str = "block truncate text-xs text-gray-500";
const FAILED_DETAIL_CLASS: &str = "block truncate text-xs text-red-700";
const DEFAULT_EMPTY_MESSAGE: &str = "No files yet.";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileLayout {
    #[default]
    List,
    Grid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ItemState {
    Stored,
    Waiting,
    Uploading,
    Removing,
    Failed(String),
}

impl ItemState {
    fn tag(&self) -> &'static str {
        match self {
            Self::Stored => "stored",
            Self::Waiting => "waiting",
            Self::Uploading => "uploading",
            Self::Removing => "removing",
            Self::Failed(_) => "failed",
        }
    }

    fn busy_message(&self) -> Option<&'static str> {
        match self {
            Self::Waiting => Some("Waiting to upload\u{2026}"),
            Self::Uploading => Some("Uploading\u{2026}"),
            Self::Removing => Some("Removing\u{2026}"),
            Self::Stored | Self::Failed(_) => None,
        }
    }

    fn border_class(&self) -> &'static str {
        match self {
            Self::Failed(_) => FAILED_BORDER_CLASS,
            Self::Stored | Self::Waiting | Self::Uploading | Self::Removing => SETTLED_BORDER_CLASS,
        }
    }
}

#[derive(Clone)]
pub(super) struct FileItem {
    pub(super) file: StoredFile,
    pub(super) state: ItemState,
    pub(super) can_retry: bool,
    pub(super) can_remove: bool,
    pub(super) primary: bool,
    pub(super) category_label: Option<String>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct ItemHandlers {
    pub(super) retry: Option<Callback<String>>,
    pub(super) remove: Option<Callback<String>>,
}

impl FileItem {
    pub(super) fn stored(file: StoredFile) -> Self {
        Self {
            file,
            state: ItemState::Stored,
            can_retry: false,
            can_remove: false,
            primary: false,
            category_label: None,
        }
    }

    fn render_key(&self) -> String {
        format!("{}:{}", self.file.key, self.state.tag())
    }
}

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

fn image_or_icon(file: &StoredFile, size: MediaSize) -> AnyView {
    let kind = file.kind();
    let Some(image_url) = file.image_url().map(str::to_owned) else {
        return kind_icon(kind, size);
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
    }
    .into_any()
}

const ACTIONS_SELECTOR: &str = "[data-file-actions]";

fn primary_badge(item: &FileItem) -> Option<AnyView> {
    item.primary.then(|| badge("Primary", Tone::Informative))
}

fn with_category(item: &FileItem, details: Option<String>) -> Option<String> {
    let parts: Vec<String> = [item.category_label.clone(), details]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join(DETAIL_SEPARATOR))
}

fn media_view(item: &FileItem, size: MediaSize) -> AnyView {
    let tile_badge = matches!(size, MediaSize::Tile)
        .then(|| primary_badge(item))
        .flatten()
        .map(|badge| view! { <span class="absolute left-2 top-2">{badge}</span> });
    let busy_overlay = item.state.busy_message().map(|_| {
        view! { <span class=BUSY_OVERLAY_CLASS>{icons::spinner(size.icon_class())}</span> }
    });
    view! {
        <span class=size.frame_class() aria-hidden="true">
            {image_or_icon(&item.file, size)}
            {tile_badge}
            {busy_overlay}
        </span>
    }
    .into_any()
}

pub(super) fn name_view(
    file: &StoredFile,
    text_class: &'static str,
    anchor: NodeRef<Span>,
) -> AnyView {
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
        <span node_ref=anchor class=format!("flex min-w-0 {text_class}")>
            <span class="truncate">{head}</span>
            <span class="shrink-0 whitespace-pre">{kept_ending}</span>
        </span>
    }
    .into_any()
}

fn detail_view(item: &FileItem, settled_details: Option<String>) -> Option<AnyView> {
    if let ItemState::Failed(reason) = &item.state {
        return Some(
            view! {
                <span class=FAILED_DETAIL_CLASS>
                    <span class="sr-only">"Failed: "</span>
                    {reason.clone()}
                </span>
            }
            .into_any(),
        );
    }
    let text = item
        .state
        .busy_message()
        .map(str::to_owned)
        .or(settled_details)?;
    Some(view! { <span class=DETAIL_CLASS>{text}</span> }.into_any())
}

fn open_overlay(item: &FileItem, open_key: RwSignal<Option<String>>) -> Option<AnyView> {
    if item.state != ItemState::Stored {
        return None;
    }
    let file = &item.file;
    if file.opens_in_viewer() {
        let key = file.key.clone();
        let label = format!("Open {}", file.name);
        return Some(
            view! {
                <button
                    type="button"
                    class=OPEN_OVERLAY_CLASS
                    aria-label=label
                    on:click=move |_| open_key.set(Some(key.clone()))
                ></button>
            }
            .into_any(),
        );
    }
    let label = format!("Download {}", file.name);
    Some(
        view! {
            <a
                href=file.url.clone()
                download=file.name.clone()
                class=OPEN_OVERLAY_CLASS
                aria-label=label
            ></a>
        }
        .into_any(),
    )
}

fn action_button(
    tooltip: &'static str,
    label: String,
    glyph: LucideGlyph,
    on_click: Callback<String>,
    key: String,
) -> AnyView {
    view! {
        <Tooltip label=tooltip align=TooltipAlign::End>
            <button
                type="button"
                class=icon_button_class(ButtonTone::Ghost, ButtonSize::Compact)
                aria-label=label
                on:click=move |_| on_click.run(key.clone())
            >
                {icons::icon(glyph, "h-4 w-4")}
            </button>
        </Tooltip>
    }
    .into_any()
}

fn download_link(file: &StoredFile) -> AnyView {
    let label = format!("Download {}", file.name);
    let url = file.url.clone();
    let name = file.name.clone();
    view! {
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
    }
    .into_any()
}

fn actions_view(
    item: &FileItem,
    handlers: ItemHandlers,
    include_download: bool,
) -> Option<AnyView> {
    let name = &item.file.name;
    let key = &item.file.key;
    let mut actions: Vec<AnyView> = Vec::new();
    if let Some(retry) = handlers.retry.filter(|_| item.can_retry) {
        actions.push(action_button(
            "Retry",
            format!("Retry {name}"),
            LucideGlyph::RotateCw,
            retry,
            key.clone(),
        ));
    }
    if include_download && item.state == ItemState::Stored {
        actions.push(download_link(&item.file));
    }
    if let Some(remove) = handlers.remove.filter(|_| item.can_remove) {
        let (tooltip, verb, glyph) = if matches!(item.state, ItemState::Failed(_)) {
            ("Dismiss", "Dismiss", LucideGlyph::X)
        } else {
            ("Remove", "Remove", LucideGlyph::Trash2)
        };
        actions.push(action_button(
            tooltip,
            format!("{verb} {name}"),
            glyph,
            remove,
            key.clone(),
        ));
    }
    (!actions.is_empty()).then(|| {
        view! {
            <span data-file-actions="" class="relative z-10 flex shrink-0 items-center gap-1">
                {actions}
            </span>
        }
        .into_any()
    })
}

fn file_row(
    item: &FileItem,
    open_key: RwSignal<Option<String>>,
    handlers: ItemHandlers,
) -> AnyView {
    let name_tooltip = OverflowTooltip::ignoring(ACTIONS_SELECTOR);
    let details = detail_view(item, with_category(item, Some(item.file.details())));
    let row_badge = primary_badge(item);
    view! {
        <li
            class=format!("{ROW_CLASS} {}", item.state.border_class())
            aria-busy=item.state.busy_message().is_some().to_string()
            on:mouseover=move |event: MouseEvent| name_tooltip.pointer_over(&event)
            on:mouseleave=move |_| name_tooltip.hide()
            on:focusin=move |event: FocusEvent| name_tooltip.focused(&event)
            on:focusout=move |_| name_tooltip.hide()
        >
            {open_overlay(item, open_key)}
            {media_view(item, MediaSize::Row)}
            <span class="pointer-events-none relative min-w-0 flex-1 text-sm">
                <span class="flex min-w-0 items-center gap-2">
                    {name_view(&item.file, NAME_CLASS, name_tooltip.anchor)} {row_badge}
                </span>
                {details}
            </span>
            {actions_view(item, handlers, true)}
            {name_tooltip.bubble(&item.file.name)}
        </li>
    }
    .into_any()
}

fn tile_details(file: &StoredFile) -> Option<String> {
    let details = file.tile_details();
    (!details.is_empty()).then_some(details)
}

fn file_tile(
    item: &FileItem,
    open_key: RwSignal<Option<String>>,
    handlers: ItemHandlers,
) -> AnyView {
    let name_tooltip = OverflowTooltip::ignoring(ACTIONS_SELECTOR);
    let details = detail_view(item, with_category(item, tile_details(&item.file)));
    view! {
        <li
            class=format!("{TILE_CLASS} {}", item.state.border_class())
            aria-busy=item.state.busy_message().is_some().to_string()
            on:mouseover=move |event: MouseEvent| name_tooltip.pointer_over(&event)
            on:mouseleave=move |_| name_tooltip.hide()
            on:focusin=move |event: FocusEvent| name_tooltip.focused(&event)
            on:focusout=move |_| name_tooltip.hide()
        >
            {open_overlay(item, open_key)}
            {media_view(item, MediaSize::Tile)}
            <span class="flex min-w-0 items-center gap-1 p-2">
                <span class="pointer-events-none relative min-w-0 flex-1 text-sm">
                    {name_view(&item.file, NAME_CLASS, name_tooltip.anchor)} {details}
                </span>
                {actions_view(item, handlers, false)}
            </span>
            {name_tooltip.bubble(&item.file.name)}
        </li>
    }
    .into_any()
}

pub(super) fn list_class(layout: FileLayout) -> &'static str {
    match layout {
        FileLayout::List => LIST_CLASS,
        FileLayout::Grid => GRID_CLASS,
    }
}

pub(super) fn arrange_media(file: &StoredFile, layout: FileLayout) -> AnyView {
    let size = match layout {
        FileLayout::List => MediaSize::Row,
        FileLayout::Grid => MediaSize::Tile,
    };
    view! {
        <span class=size.frame_class() aria-hidden="true">
            {image_or_icon(file, size)}
        </span>
    }
    .into_any()
}

type ItemRenderer = fn(&FileItem, RwSignal<Option<String>>, ItemHandlers) -> AnyView;

pub(super) fn file_items(
    items: Signal<Vec<FileItem>>,
    layout: FileLayout,
    label: String,
    open_key: RwSignal<Option<String>>,
    handlers: ItemHandlers,
) -> AnyView {
    let (list_class, render): (&'static str, ItemRenderer) = match layout {
        FileLayout::List => (LIST_CLASS, file_row),
        FileLayout::Grid => (GRID_CLASS, file_tile),
    };
    view! {
        <ul aria-label=label class=list_class>
            <For
                each=move || items.get()
                key=FileItem::render_key
                children=move |item| render(&item, open_key, handlers)
            />
        </ul>
    }
    .into_any()
}

fn read_only_files(
    files: Signal<Vec<StoredFile>>,
    layout: FileLayout,
    label: String,
    empty_message: Option<String>,
) -> AnyView {
    let empty_message = empty_message.unwrap_or_else(|| DEFAULT_EMPTY_MESSAGE.to_owned());
    let open_key: RwSignal<Option<String>> = RwSignal::new(None);
    let items = Signal::derive(move || files.get().into_iter().map(FileItem::stored).collect());
    let content = move || {
        if files.with(Vec::is_empty) {
            return view! { <EmptyState title=empty_message.clone() /> }.into_any();
        }
        file_items(
            items,
            layout,
            label.clone(),
            open_key,
            ItemHandlers::default(),
        )
    };
    view! {
        {content}
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
    read_only_files(files, FileLayout::List, label, empty_message)
}

#[component]
pub fn FileGrid(
    #[prop(into)] files: Signal<Vec<StoredFile>>,
    #[prop(into)] label: String,
    #[prop(optional, into)] empty_message: Option<String>,
) -> impl IntoView {
    read_only_files(files, FileLayout::Grid, label, empty_message)
}

#[must_use]
pub fn file_cell(file: &StoredFile) -> AnyView {
    let kind = file.kind();
    let name_tooltip = OverflowTooltip::new();
    let icon = icons::icon(
        kind.glyph(),
        format!("h-4 w-4 shrink-0 {}", kind.icon_color_class()),
    );
    let link_class = "min-w-0 text-blue-600 hover:text-blue-800 hover:underline";
    let link = if file.opens_in_viewer() {
        view! {
            <a href=file.url.clone() target="_blank" rel="noopener" class=link_class>
                {name_view(file, "", name_tooltip.anchor)}
            </a>
        }
        .into_any()
    } else {
        view! {
            <a href=file.url.clone() download=file.name.clone() class=link_class>
                {name_view(file, "", name_tooltip.anchor)}
            </a>
        }
        .into_any()
    };
    view! {
        <span
            class="flex max-w-xs min-w-0 items-center gap-2"
            on:mouseover=move |event: MouseEvent| name_tooltip.pointer_over(&event)
            on:mouseleave=move |_| name_tooltip.hide()
            on:focusin=move |event: FocusEvent| name_tooltip.focused(&event)
            on:focusout=move |_| name_tooltip.hide()
        >
            {icon}
            {link}
            {name_tooltip.bubble(&file.name)}
        </span>
    }
    .into_any()
}
