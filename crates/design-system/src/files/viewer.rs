use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

use super::{FileKind, StoredFile, DETAIL_SEPARATOR};
use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::dialog::{Dialog, DialogSize};
use crate::icons::{self, LucideGlyph};
use crate::tooltip::Tooltip;

const STAGE_CLASS: &str =
    "flex min-h-0 flex-1 items-center justify-center overflow-hidden rounded-md bg-gray-100";
const ACTION_LABEL_CLASS: &str = "max-sm:sr-only";
const UNAVAILABLE_CLASS: &str =
    "flex flex-col items-center gap-3 p-6 text-center text-sm text-gray-600";

fn download_link(file: &StoredFile) -> AnyView {
    view! {
        <a
            href=file.url.clone()
            download=file.name.clone()
            class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
        >
            {icons::icon(LucideGlyph::Download, "h-4 w-4 shrink-0 opacity-70")}
            <span class=ACTION_LABEL_CLASS>"Download"</span>
        </a>
    }
    .into_any()
}

fn file_actions(file: &StoredFile) -> AnyView {
    view! {
        {download_link(file)}
        <a
            href=file.url.clone()
            target="_blank"
            rel="noopener"
            class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
        >
            {icons::icon(LucideGlyph::ExternalLink, "h-4 w-4 shrink-0 opacity-70")}
            <span class=ACTION_LABEL_CLASS>"Open in New Tab"</span>
        </a>
    }
    .into_any()
}

fn unavailable_view(file: &StoredFile, message: &'static str) -> AnyView {
    let kind = file.kind();
    view! {
        <div class=UNAVAILABLE_CLASS>
            {icons::icon(kind.glyph(), format!("h-10 w-10 {}", kind.icon_color_class()))}
            <p>{message}</p> {download_link(file)}
        </div>
    }
    .into_any()
}

fn image_view(file: &StoredFile) -> AnyView {
    let broken = RwSignal::new(false);
    let url = file.url.clone();
    let name = file.name.clone();
    let unavailable = file.clone();
    view! {
        <Show
            when=move || !broken.get()
            fallback=move || unavailable_view(&unavailable, "This image could not be loaded.")
        >
            <img
                src=url.clone()
                alt=name.clone()
                class="max-h-full max-w-full object-contain"
                on:error=move |_| broken.set(true)
            />
        </Show>
    }
    .into_any()
}

fn preview_view(file: &StoredFile) -> AnyView {
    match file.kind() {
        FileKind::Image => image_view(file),
        FileKind::Pdf => view! {
            <iframe
                src=file.url.clone()
                title=file.name.clone()
                class="h-full w-full bg-white"
            ></iframe>
        }
        .into_any(),
        FileKind::Spreadsheet
        | FileKind::Document
        | FileKind::Email
        | FileKind::Archive
        | FileKind::Video
        | FileKind::Audio
        | FileKind::Other => unavailable_view(file, "There is no preview for this kind of file."),
    }
}

fn step_button(
    label: &'static str,
    glyph: LucideGlyph,
    available: Signal<bool>,
    step: impl Fn() + Send + Sync + 'static,
) -> AnyView {
    view! {
        <Tooltip label=label>
            <button
                type="button"
                aria-label=label
                class=format!(
                    "{} disabled:cursor-not-allowed disabled:opacity-40",
                    icon_button_class(ButtonTone::Secondary, ButtonSize::Compact),
                )
                prop:disabled=move || !available.get()
                on:click=move |_| step()
            >
                {icons::icon(glyph, "h-4 w-4")}
            </button>
        </Tooltip>
    }
    .into_any()
}

fn viewer_dialog(
    files: Signal<Vec<StoredFile>>,
    open_key: RwSignal<Option<String>>,
    position: Memo<Option<usize>>,
) -> AnyView {
    let count = Signal::derive(move || files.with(Vec::len));
    let current = Signal::derive(move || {
        position
            .get()
            .and_then(|index| files.with(|all| all.get(index).cloned()))
    });
    let go_to = move |index: Option<usize>| {
        let target = index.and_then(|index| files.with_untracked(|all| all.get(index).cloned()));
        if let Some(file) = target {
            open_key.set(Some(file.key));
        }
    };
    let previous = move || {
        go_to(
            position
                .get_untracked()
                .and_then(|index| index.checked_sub(1)),
        );
    };
    let next = move || {
        go_to(
            position
                .get_untracked()
                .and_then(|index| index.checked_add(1)),
        );
    };
    let has_previous = Signal::derive(move || position.get().is_some_and(|index| index > 0));
    let has_next = Signal::derive(move || {
        position
            .get()
            .is_some_and(|index| index.saturating_add(1) < count.get())
    });
    let on_keydown = move |event: KeyboardEvent| match event.key().as_str() {
        "ArrowLeft" => {
            event.prevent_default();
            previous();
        }
        "ArrowRight" => {
            event.prevent_default();
            next();
        }
        _ => {}
    };
    let title = Signal::derive(move || current.get().map(|file| file.name).unwrap_or_default());
    let description = Signal::derive(move || {
        let place = position
            .get()
            .map(|index| format!("{} of {}", index.saturating_add(1), count.get()));
        let details = current.get().map(|file| file.details());
        [place, details]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(DETAIL_SEPARATOR)
    });

    view! {
        <Dialog
            title
            description
            size=DialogSize::FullScreen
            on_close=Callback::new(move |()| open_key.set(None))
        >
            <div
                data-autofocus=""
                tabindex="-1"
                class="flex min-h-0 flex-1 flex-col gap-3 focus:outline-none"
                on:keydown=on_keydown
            >
                <div class="flex flex-wrap items-center gap-2">
                    {step_button("Previous", LucideGlyph::ChevronLeft, has_previous, previous)}
                    {step_button("Next", LucideGlyph::ChevronRight, has_next, next)}
                    <span class="flex-1"></span>
                    {move || current.get().map(|file| file_actions(&file))}
                </div>
                <div class=STAGE_CLASS>{move || current.get().map(|file| preview_view(&file))}</div>
            </div>
        </Dialog>
    }
    .into_any()
}

#[component]
pub fn FileViewer(
    #[prop(into)] files: Signal<Vec<StoredFile>>,
    open_key: RwSignal<Option<String>>,
) -> impl IntoView {
    let position = Memo::new(move |_| {
        let key = open_key.get()?;
        files.with(|all| all.iter().position(|file| file.key == key))
    });
    let is_open = Memo::new(move |_| position.get().is_some());
    view! { {move || is_open.get().then(|| viewer_dialog(files, open_key, position))} }.into_any()
}
