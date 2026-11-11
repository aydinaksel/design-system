use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html::Button;
use leptos::prelude::*;
use leptos_use_timeout_fn::use_timeout_fn;

use crate::controls::{field_control_id, field_description_id, with_field_error, CONTROL_CLASS};
use crate::element_key::ElementKey;
use crate::icons;
use crate::menu::{
    is_typeahead_key, moved_position, typeahead_position, Movement,
    TYPEAHEAD_RESET_DELAY_MILLISECONDS,
};
use crate::placement::{Alignment, AnchoredPanel, Anchoring, PreferredSide};

pub(crate) const LISTBOX_PANEL_MAXIMUM_HEIGHT_PIXELS: f64 = 240.0;
pub(crate) const LISTBOX_PANEL_CLASS: &str = "fixed z-50 max-h-60 max-w-[calc(100vw-1rem)] overflow-y-auto rounded-surface border border-gray-200 bg-surface p-1 text-sm/6 shadow-lg focus:outline-none motion-safe:animate-fade-in";
pub(crate) const LISTBOX_ANCHORING: Anchoring = Anchoring {
    alignment: Alignment::Start,
    side: PreferredSide::Below,
    gap_pixels: 4.0,
    matches_trigger_width: true,
    reserved_height_pixels: 0.0,
};
const DEFAULT_PLACEHOLDER: &str = "Select\u{2026}";
const OPTION_CLASS: &str =
    "flex cursor-pointer select-none items-center gap-2 rounded-item px-3 py-1.5 text-gray-700";
const HIGHLIGHTED_OPTION_CLASS: &str =
    "flex cursor-pointer select-none items-center gap-2 rounded-item bg-gray-100 px-3 py-1.5 text-gray-900";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListboxOption {
    pub value: String,
    pub label: String,
}

impl ListboxOption {
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one widget: the button, the options, and their key handling belong together"
)]
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Listbox(
    #[prop(into)] options: Signal<Vec<ListboxOption>>,
    selected: RwSignal<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] invalid: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let invalid = with_field_error(invalid);
    let listbox_number = ElementKey::next();
    let panel_id = format!("listbox-{listbox_number}-options");
    let option_id = move |index: usize| format!("listbox-{listbox_number}-option-{index}");
    let placeholder = placeholder.unwrap_or_else(|| DEFAULT_PLACEHOLDER.to_owned());

    let open = RwSignal::new(false);
    let highlighted: RwSignal<Option<usize>> = RwSignal::new(None);
    let panel = AnchoredPanel::new();
    let trigger: NodeRef<Button> = NodeRef::new();
    let typeahead_query = StoredValue::new(String::new());
    let reset_typeahead = use_timeout_fn(
        move |()| typeahead_query.set_value(String::new()),
        TYPEAHEAD_RESET_DELAY_MILLISECONDS,
    )
    .start;
    panel.follow(
        open.into(),
        move || trigger.get_untracked().map(Into::into),
        LISTBOX_ANCHORING,
    );

    let selected_index = move || {
        let current = selected.get_untracked();
        options.with_untracked(|all| all.iter().position(|option| option.value == current))
    };
    let selected_label = move || {
        let current = selected.get();
        options.with(|all| {
            all.iter()
                .find(|option| option.value == current)
                .map(|option| option.label.clone())
        })
    };

    let open_with = move |fallback: Movement| {
        let count = options.with_untracked(Vec::len);
        highlighted.set(selected_index().or_else(|| moved_position(None, count, fallback)));
        open.set(true);
    };
    let close = move || {
        open.set(false);
        highlighted.set(None);
        panel.unplace();
    };
    let choose = move |index: Option<usize>| {
        let chosen = index.and_then(|position| {
            options.with_untracked(|all| all.get(position).map(|option| option.value.clone()))
        });
        if let Some(value) = chosen {
            selected.set(value);
        }
        close();
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
        if !open.get_untracked() {
            let fallback = match key.as_str() {
                "ArrowDown" | " " => Movement::First,
                "ArrowUp" => Movement::Last,
                _ if is_typeahead_key(&event, false) => {
                    event.prevent_default();
                    open_with(Movement::First);
                    extend_typeahead(&key);
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            open_with(fallback);
            return;
        }
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
        match key.as_str() {
            "Enter" | " " => {
                event.prevent_default();
                event.stop_propagation();
                choose(highlighted.get_untracked());
            }
            "Escape" => {
                event.prevent_default();
                event.stop_propagation();
                close();
            }
            "Tab" => choose(highlighted.get_untracked()),
            _ => {}
        }
    };

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let Some(index) = highlighted.get() else {
            return;
        };
        if !open.get() {
            return;
        }
        if let Some(element) = document().get_element_by_id(&option_id(index)) {
            let scrolling = web_sys::ScrollIntoViewOptions::new();
            scrolling.set_block(web_sys::ScrollLogicalPosition::Nearest);
            element.scroll_into_view_with_scroll_into_view_options(&scrolling);
        }
    });

    let panel_id_for_button = panel_id.clone();
    let panel_id = StoredValue::new(panel_id);

    view! {
        <div class="relative">
            <button
                type="button"
                node_ref=trigger
                id=field_control_id()
                aria-label=label
                aria-describedby=field_description_id()
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded=move || open.get().to_string()
                aria-controls=move || open.get().then(|| panel_id_for_button.clone())
                aria-activedescendant=move || {
                    open.get().then(|| highlighted.get().map(option_id)).flatten()
                }
                aria-invalid=move || invalid.get().to_string()
                class=format!("{CONTROL_CLASS} relative cursor-pointer pr-9 text-left")
                prop:disabled=move || disabled.get()
                on:click=move |_| {
                    if open.get_untracked() {
                        close();
                    } else {
                        open_with(Movement::First);
                    }
                }
                on:keydown=on_keydown
                on:blur=move |_| close()
            >
                {move || match selected_label() {
                    Some(label) => view! { <span class="block truncate">{label}</span> }.into_any(),
                    None => {
                        view! {
                            <span class="block truncate text-gray-400">{placeholder.clone()}</span>
                        }
                            .into_any()
                    }
                }}
                <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2.5">
                    {icons::icon(icons::LucideGlyph::ChevronsUpDown, "h-4 w-4 text-gray-400")}
                </span>
            </button>
            {move || {
                open.get()
                    .then(|| {
                        let rows = options
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(index, option)| {
                                let is_selected = option.value == selected.get_untracked();
                                let tick_class = if is_selected {
                                    "h-4 w-4 shrink-0 text-blue-600"
                                } else {
                                    "invisible h-4 w-4 shrink-0"
                                };
                                view! {
                                    <div
                                        id=option_id(index)
                                        role="option"
                                        aria-selected=is_selected.to_string()
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
                                        {icons::icon(icons::LucideGlyph::Check, tick_class)}
                                        <span class="truncate">{option.label}</span>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>();
                        view! {
                            <div
                                id=panel_id.get_value()
                                role="listbox"
                                node_ref=panel.node
                                class=LISTBOX_PANEL_CLASS
                                style=move || panel.style()
                                on:mousedown=|event: MouseEvent| event.prevent_default()
                            >
                                {rows}
                            </div>
                        }
                    })
            }}
        </div>
    }
    .into_any()
}
