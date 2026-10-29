use std::sync::atomic::{AtomicU64, Ordering};

use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos_use::use_timeout_fn;

use crate::icons;
use crate::menu::{
    is_typeahead_key, moved_position, typeahead_position, Movement,
    TYPEAHEAD_RESET_DELAY_MILLISECONDS,
};

static NEXT_CHOICE_LIST_NUMBER: AtomicU64 = AtomicU64::new(0);

const LIST_CLASS: &str = "max-h-72 overflow-y-auto overscroll-contain rounded-lg border border-gray-200 bg-white p-1 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-1";
const OPTION_CLASS: &str =
    "flex cursor-pointer select-none items-center gap-3 rounded-md px-3 py-2 text-gray-700";
const HIGHLIGHTED_OPTION_CLASS: &str = "flex cursor-pointer select-none items-center gap-3 rounded-md bg-gray-100 px-3 py-2 text-gray-900";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub glyph: Option<icons::LucideGlyph>,
}

impl ChoiceOption {
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            glyph: None,
        }
    }

    #[must_use]
    pub fn with_description(self, description: impl Into<String>) -> Self {
        Self {
            description: Some(description.into()),
            ..self
        }
    }

    #[must_use]
    pub fn with_glyph(self, glyph: icons::LucideGlyph) -> Self {
        Self {
            glyph: Some(glyph),
            ..self
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one widget: the list, its options, and their key handling belong together"
)]
#[component]
pub fn ChoiceList(
    #[prop(into)] options: Signal<Vec<ChoiceOption>>,
    #[prop(into)] on_choose: Callback<String>,
    #[prop(optional, into)] current: Signal<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional)] autofocus: bool,
) -> impl IntoView {
    let list_number = NEXT_CHOICE_LIST_NUMBER.fetch_add(1, Ordering::Relaxed);
    let option_id = move |index: usize| format!("choice-list-{list_number}-option-{index}");

    let current_index = move || {
        let current = current.get_untracked();
        options.with_untracked(|all| all.iter().position(|option| option.value == current))
    };
    let highlighted: RwSignal<Option<usize>> = RwSignal::new(
        current_index().or_else(|| options.with_untracked(|all| (!all.is_empty()).then_some(0))),
    );
    let typeahead_query = StoredValue::new(String::new());
    let reset_typeahead = use_timeout_fn(
        move |()| typeahead_query.set_value(String::new()),
        TYPEAHEAD_RESET_DELAY_MILLISECONDS,
    )
    .start;

    let choose = move |index: Option<usize>| {
        let chosen = index.and_then(|position| {
            options.with_untracked(|all| all.get(position).map(|option| option.value.clone()))
        });
        if let Some(value) = chosen {
            on_choose.run(value);
        }
    };
    let move_highlight = move |movement: Movement| {
        let count = options.with_untracked(Vec::len);
        highlighted.update(|current| *current = moved_position(*current, count, movement));
    };
    let extend_typeahead = move |key: &str| {
        typeahead_query.update_value(|query| query.push_str(key));
        let labels: Vec<String> =
            options.with_untracked(|all| all.iter().map(|option| option.label.clone()).collect());
        let query = typeahead_query.get_value();
        if let Some(found) = typeahead_position(&labels, highlighted.get_untracked(), &query) {
            highlighted.set(Some(found));
        }
        reset_typeahead(());
    };

    let on_keydown = move |event: KeyboardEvent| {
        let key = event.key();
        let query_in_progress = typeahead_query.with_value(|query| !query.is_empty());
        let movement = match key.as_str() {
            "ArrowDown" => Some(Movement::Next),
            "ArrowUp" => Some(Movement::Previous),
            "Home" | "PageUp" => Some(Movement::First),
            "End" | "PageDown" => Some(Movement::Last),
            _ => None,
        };
        if let Some(movement) = movement {
            event.prevent_default();
            move_highlight(movement);
            return;
        }
        if is_typeahead_key(&event, query_in_progress) {
            event.prevent_default();
            extend_typeahead(&key);
            return;
        }
        if key == "Enter" || key == " " {
            event.prevent_default();
            choose(highlighted.get_untracked());
        }
    };

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let Some(index) = highlighted.get() else {
            return;
        };
        if let Some(element) = document().get_element_by_id(&option_id(index)) {
            let scrolling = web_sys::ScrollIntoViewOptions::new();
            scrolling.set_block(web_sys::ScrollLogicalPosition::Nearest);
            element.scroll_into_view_with_scroll_into_view_options(&scrolling);
        }
    });

    let rows = move || {
        let current = current.get();
        options
            .get()
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let is_current = option.value == current;
                let tick_class = if is_current {
                    "ml-auto h-4 w-4 shrink-0 text-blue-600"
                } else {
                    "invisible ml-auto h-4 w-4 shrink-0"
                };
                let glyph = option
                    .glyph
                    .map(|glyph| icons::icon(glyph, "h-4 w-4 shrink-0 text-gray-400"));
                let description = option.description.map(|description| {
                    view! { <span class="block text-xs text-gray-500">{description}</span> }
                });
                view! {
                    <div
                        id=option_id(index)
                        role="option"
                        aria-selected=is_current.to_string()
                        class=move || {
                            if highlighted.get() == Some(index) {
                                HIGHLIGHTED_OPTION_CLASS
                            } else {
                                OPTION_CLASS
                            }
                        }
                        on:mousemove=move |_| {
                            if highlighted.get_untracked() != Some(index) {
                                highlighted.set(Some(index));
                            }
                        }
                        on:click=move |_| choose(Some(index))
                    >
                        {glyph}
                        <span class="min-w-0">
                            <span class="block truncate">{option.label}</span>
                            {description}
                        </span>
                        {icons::icon(icons::LucideGlyph::Check, tick_class)}
                    </div>
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div
            role="listbox"
            tabindex="0"
            aria-label=label
            aria-activedescendant=move || highlighted.get().map(option_id)
            data-autofocus=autofocus.then_some("")
            class=LIST_CLASS
            on:keydown=on_keydown
        >
            {rows}
        </div>
    }
    .into_any()
}
