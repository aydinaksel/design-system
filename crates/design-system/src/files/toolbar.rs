use leptos::prelude::*;

use super::arrange::FileCategory;
use super::section::SectionState;
use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::filter_chip::FilterChip;
use crate::icons::{self, LucideGlyph};

const TOUCH_ONLY_CLASS: &str = "hidden pointer-coarse:contents";
const PHONE_EMPTY_CLASS: &str = "hidden flex-col items-center gap-3 rounded-surface border-2 border-dashed border-gray-300 bg-surface px-6 py-8 text-center pointer-coarse:flex";
const DROP_ZONE_CLASS: &str = "flex w-full cursor-pointer flex-col items-center gap-1 rounded-surface border-2 border-dashed border-gray-300 bg-surface px-6 py-8 text-center text-sm transition-colors hover:border-gray-400 focus-ring disabled:cursor-not-allowed disabled:opacity-60";

fn large_button(
    label: &'static str,
    glyph: LucideGlyph,
    tone: ButtonTone,
    disabled: Signal<bool>,
    on_click: Callback<()>,
) -> AnyView {
    view! {
        <button
            type="button"
            class=button_class(tone, ButtonSize::Standard)
            prop:disabled=move || disabled.get()
            on:click=move |_| on_click.run(())
        >
            {icons::icon(glyph, "h-4 w-4 shrink-0 opacity-70")}
            {label}
        </button>
    }
    .into_any()
}

pub(super) fn drop_zone_view(
    hint: String,
    disabled: Signal<bool>,
    open_picker: Callback<()>,
    take_photo: Option<Callback<()>>,
) -> AnyView {
    let Some(take_photo) = take_photo else {
        return drop_zone(hint, disabled, open_picker);
    };
    view! {
        <div class="pointer-coarse:hidden">{drop_zone(hint.clone(), disabled, open_picker)}</div>
        <div class=PHONE_EMPTY_CLASS>
            {icons::icon(LucideGlyph::Camera, "h-5 w-5 text-gray-400")}
            <span class="text-xs text-gray-500">{hint}</span>
            <div class="flex flex-wrap justify-center gap-2">
                {large_button(
                    "Take Photo",
                    LucideGlyph::Camera,
                    ButtonTone::Primary,
                    disabled,
                    take_photo,
                )}
                {large_button(
                    "Choose Files",
                    LucideGlyph::Upload,
                    ButtonTone::Secondary,
                    disabled,
                    open_picker,
                )}
            </div>
        </div>
    }
    .into_any()
}

fn drop_zone(hint: String, disabled: Signal<bool>, open_picker: Callback<()>) -> AnyView {
    view! {
        <button
            type="button"
            class=DROP_ZONE_CLASS
            prop:disabled=move || disabled.get()
            on:click=move |_| open_picker.run(())
        >
            {icons::icon(LucideGlyph::Upload, "h-5 w-5 text-gray-400")}
            <span class="font-medium text-gray-700">"Drop Files Here, or Browse"</span>
            <span class="text-xs text-gray-500">{hint}</span>
        </button>
    }
    .into_any()
}

fn compact_button(
    label: &'static str,
    glyph: LucideGlyph,
    disabled: Signal<bool>,
    on_click: Callback<()>,
) -> AnyView {
    view! {
        <button
            type="button"
            class=format!(
                "{} disabled:cursor-not-allowed",
                button_class(ButtonTone::Secondary, ButtonSize::Compact),
            )
            prop:disabled=move || disabled.get()
            on:click=move |_| on_click.run(())
        >
            {icons::icon(glyph, "h-4 w-4 shrink-0 opacity-70")}
            {label}
        </button>
    }
    .into_any()
}

pub(super) fn toolbar_view(
    hint: String,
    state: &SectionState,
    disabled: Signal<bool>,
    open_picker: Callback<()>,
    take_photo: Option<Callback<()>>,
) -> AnyView {
    let state = *state;
    let camera = take_photo.map(|take_photo| {
        view! {
            <span class=TOUCH_ONLY_CLASS>
                {compact_button("Take Photo", LucideGlyph::Camera, disabled, take_photo)}
            </span>
        }
    });
    let arranging = state.arranging;
    let arrange = arranging.enabled().then(|| {
        let unavailable = Signal::derive(move || {
            disabled.get() || state.queue.is_busy() || state.stored_in_order().is_empty()
        });
        let begin = Callback::new(move |()| {
            state.presentation.filter.set(None);
            arranging.begin(state.stored_in_order());
        });
        compact_button("Arrange", LucideGlyph::ArrowUpDown, unavailable, begin)
    });
    view! {
        <div class="flex flex-wrap items-center justify-between gap-3">
            <p class="text-xs text-gray-500">{hint}</p>
            <div class="flex items-center gap-2">
                {arrange} {camera}
                {compact_button("Add Files", LucideGlyph::Upload, disabled, open_picker)}
            </div>
        </div>
    }
    .into_any()
}

fn category_chip(
    filter: RwSignal<Option<String>>,
    category: FileCategory,
    count: usize,
) -> AnyView {
    let key = category.key.clone();
    let selected_key = category.key;
    let selected = Signal::derive(move || filter.get().as_deref() == Some(selected_key.as_str()));
    let toggle = Callback::new(move |()| {
        let chosen = Some(key.clone());
        filter.update(|current| *current = if *current == chosen { None } else { chosen });
    });
    view! { <FilterChip label=category.label selected on_toggle=toggle count=count /> }.into_any()
}

pub(super) fn category_chips(state: &SectionState) -> AnyView {
    let state = *state;
    let filter = state.presentation.filter;
    let chips = move || {
        let counts = state.category_counts();
        if counts.is_empty() {
            return ().into_any();
        }
        let total = state.stored_in_order().len();
        let chips = counts
            .into_iter()
            .map(|(category, count)| category_chip(filter, category, count))
            .collect_view();
        view! {
            <div role="group" aria-label="Filter by Type" class="flex flex-wrap gap-2">
                <FilterChip
                    label="All"
                    count=total
                    selected=Signal::derive(move || filter.get().is_none())
                    on_toggle=Callback::new(move |()| filter.set(None))
                />
                {chips}
            </div>
        }
        .into_any()
    };
    view! { {chips} }.into_any()
}
