use std::collections::BTreeSet;

use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::prelude::*;
use leptos_use_timeout_fn::use_timeout_fn;

use crate::checkbox::checkbox_mark;
use crate::element_key::ElementKey;
use crate::icons;
use crate::listbox::{HIGHLIGHTED_OPTION_CLASS, OPTION_CLASS};
use crate::menu::{
    focus_on_hover, is_typeahead_key, moved_position, typeahead_position, Movement, ICON_CLASS,
    ITEM_CLASS, TYPEAHEAD_RESET_DELAY_MILLISECONDS,
};
use crate::search_input::{SearchInput, SearchSize};

const SEARCH_VISIBILITY_THRESHOLD: usize = 10;
const DEFAULT_SEARCH_PLACEHOLDER: &str = "Search Values";
pub(crate) const SCROLLBAR_GUTTER_CLASS: &str = "[scrollbar-gutter:stable] [scrollbar-width:thin]";
pub(crate) const OPTION_END_INSET_CLASS: &str = "pr-4.5";
const LIST_CLASS: &str = "max-h-56 overflow-auto p-1.5 text-sm focus:outline-none";
const ROW_COUNT_CLASS: &str = "shrink-0 text-xs text-gray-400 tabular-nums";
const UNAVAILABLE_ROW_COUNT_CLASS: &str = "shrink-0 text-xs text-gray-300 tabular-nums";
const LABEL_CLASS: &str = "min-w-0 flex-1 truncate";
const UNAVAILABLE_LABEL_CLASS: &str = "min-w-0 flex-1 truncate text-gray-400";
const NO_MATCHES_CLASS: &str = "px-3 py-1.5 text-sm text-gray-400";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiSelectOption {
    pub value: String,
    pub label: String,
    pub row_count: Option<u64>,
}

fn key_movement(key: &str) -> Option<Movement> {
    match key {
        "ArrowDown" => Some(Movement::Next),
        "ArrowUp" => Some(Movement::Previous),
        "Home" | "PageUp" => Some(Movement::First),
        "End" | "PageDown" => Some(Movement::Last),
        _ => None,
    }
}

fn first_position(count: usize) -> Option<usize> {
    moved_position(None, count, Movement::First)
}

fn is_unavailable(option: &MultiSelectOption) -> bool {
    option.row_count == Some(0)
}

fn option_row(
    index: usize,
    option: MultiSelectOption,
    id: String,
    highlighted: RwSignal<Option<usize>>,
    selected: Signal<BTreeSet<String>>,
    toggle: Callback<Option<usize>>,
) -> AnyView {
    let unavailable = is_unavailable(&option);
    let MultiSelectOption {
        value,
        label,
        row_count,
    } = option;
    let checked = Signal::derive(move || selected.with(|all| all.contains(&value)));
    let (label_class, count_class) = if unavailable {
        (UNAVAILABLE_LABEL_CLASS, UNAVAILABLE_ROW_COUNT_CLASS)
    } else {
        (LABEL_CLASS, ROW_COUNT_CLASS)
    };
    let count = row_count.map(|row_count| view! { <span class=count_class>{row_count}</span> });
    view! {
        <div
            id=id
            role="option"
            aria-selected=move || checked.get().to_string()
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
            on:click=move |_| toggle.run(Some(index))
        >
            {checkbox_mark(checked)}
            <span class=label_class>{label}</span>
            {count}
        </div>
    }
    .into_any()
}

fn clear_selection_item(on_clear: Callback<()>) -> AnyView {
    view! {
        <div class="border-t border-divider p-1">
            <button
                type="button"
                class=ITEM_CLASS
                on:mousemove=|event: MouseEvent| focus_on_hover(&event)
                on:click=move |_| on_clear.run(())
            >
                {icons::icon(icons::LucideGlyph::X, ICON_CLASS)}
                "Clear Selection"
            </button>
        </div>
    }
    .into_any()
}

#[expect(
    clippy::too_many_lines,
    reason = "one widget: the search, the options, and their key handling belong together"
)]
#[component]
pub fn MultiSelectList(
    #[prop(into)] options: Signal<Vec<MultiSelectOption>>,
    #[prop(into)] selected: Signal<BTreeSet<String>>,
    #[prop(into)] on_toggle: Callback<String>,
    #[prop(into)] label: String,
    #[prop(optional, into)] on_clear: Option<Callback<()>>,
    #[prop(default = DEFAULT_SEARCH_PLACEHOLDER)] search_placeholder: &'static str,
) -> impl IntoView {
    let list_number = ElementKey::next();
    let list_id = format!("multi-select-{list_number}-options");
    let option_id = move |index: usize| format!("multi-select-{list_number}-option-{index}");
    let searchable = options.with_untracked(|all| all.len() > SEARCH_VISIBILITY_THRESHOLD);
    let value_search = RwSignal::new(String::new());
    let pinned_first = if searchable {
        selected.get_untracked()
    } else {
        BTreeSet::new()
    };
    let visible = Memo::new(move |_| {
        let search_lower = value_search.get().to_lowercase();
        let mut matching: Vec<MultiSelectOption> = options.with(|all| {
            all.iter()
                .filter(|option| {
                    search_lower.is_empty() || option.label.to_lowercase().contains(&search_lower)
                })
                .cloned()
                .collect()
        });
        matching.sort_by_key(|option| !pinned_first.contains(&option.value));
        matching
    });
    let highlighted: RwSignal<Option<usize>> =
        RwSignal::new(first_position(visible.with_untracked(Vec::len)));
    Effect::watch(
        move || value_search.get(),
        move |_, _, _| highlighted.set(first_position(visible.with_untracked(Vec::len))),
        false,
    );
    let active_descendant = Signal::derive(move || highlighted.get().map(option_id));

    let typeahead_query = StoredValue::new(String::new());
    let reset_typeahead = use_timeout_fn(
        move |()| typeahead_query.set_value(String::new()),
        TYPEAHEAD_RESET_DELAY_MILLISECONDS,
    )
    .start;
    let extend_typeahead = move |key: &str| {
        typeahead_query.update_value(|query| query.push_str(key));
        let labels: Vec<String> =
            visible.with_untracked(|all| all.iter().map(|option| option.label.clone()).collect());
        let query = typeahead_query.get_value();
        if let Some(found) = typeahead_position(&labels, highlighted.get_untracked(), &query) {
            highlighted.set(Some(found));
        }
        reset_typeahead(());
    };

    let toggle = Callback::new(move |index: Option<usize>| {
        let value = index.and_then(|position| {
            visible.with_untracked(|all| all.get(position).map(|option| option.value.clone()))
        });
        if let Some(value) = value {
            on_toggle.run(value);
        }
    });

    let on_keydown = move |event: KeyboardEvent| {
        let key = event.key();
        if let Some(movement) = key_movement(&key) {
            event.prevent_default();
            let count = visible.with_untracked(Vec::len);
            highlighted.update(|current| *current = moved_position(*current, count, movement));
            return;
        }
        let query_in_progress = typeahead_query.with_value(|query| !query.is_empty());
        let toggles = key == "Enter" || (key == " " && !searchable && !query_in_progress);
        if toggles {
            event.prevent_default();
            toggle.run(highlighted.get_untracked());
            return;
        }
        if !searchable && is_typeahead_key(&event, query_in_progress) {
            event.prevent_default();
            extend_typeahead(&key);
        }
    };

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(index) = highlighted.get() {
            crate::listbox::scroll_option_into_view(&option_id(index));
        }
    });

    let search_box = searchable.then(|| {
        view! {
            <div class="border-b border-divider p-2.5">
                <SearchInput
                    value=value_search
                    placeholder=search_placeholder
                    size=SearchSize::Compact
                    controls=list_id.clone()
                    active_descendant=active_descendant
                    autofocus=true
                />
            </div>
        }
    });

    let rows = move || {
        let all = visible.get();
        if all.is_empty() {
            return view! { <div class=NO_MATCHES_CLASS>"No matches."</div> }.into_any();
        }
        all.into_iter()
            .enumerate()
            .map(|(index, option)| {
                option_row(
                    index,
                    option,
                    option_id(index),
                    highlighted,
                    selected,
                    toggle,
                )
            })
            .collect_view()
            .into_any()
    };

    let clear_item = move || {
        let something_selected = selected.with(|all| !all.is_empty());
        on_clear
            .filter(|_| something_selected)
            .map(clear_selection_item)
    };

    view! {
        <div on:keydown=on_keydown>
            {search_box}
            <div
                id=list_id
                role="listbox"
                aria-multiselectable="true"
                aria-label=label
                tabindex=if searchable { "-1" } else { "0" }
                data-autofocus=(!searchable).then_some("")
                aria-activedescendant=move || {
                    (!searchable).then(|| active_descendant.get()).flatten()
                }
                class=format!("{LIST_CLASS} {SCROLLBAR_GUTTER_CLASS}")
                on:mousedown=|event: MouseEvent| event.prevent_default()
                on:mouseleave=move |_| highlighted.set(None)
            >
                {rows}
            </div> {clear_item}
        </div>
    }
    .into_any()
}
