use leptos::ev::{Event, FocusEvent};
use leptos::html::Textarea;
use leptos::prelude::*;

use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::controls::{
    field_control_id, field_description_id, field_interaction, user_validity_handler,
    with_field_error,
};
use crate::focus_loss::while_mounted;
use crate::icons;
use crate::markdown::MarkdownText;
use crate::tooltip::Tooltip;

const EDITOR_CLASS: &str = "rounded-lg border border-control-edge bg-surface shadow-sm focus-within:border-focus focus-within:ring-1 focus-within:ring-focus";
const TOOLBAR_CLASS: &str =
    "flex items-center justify-between gap-2 border-b border-divider px-1.5 py-1";
const TEXTAREA_CLASS: &str = "block min-h-40 w-full resize-y rounded-b-lg border-0 bg-transparent px-3 py-2 text-sm text-gray-900 placeholder:text-gray-400 focus:outline-none focus:ring-0";
const PREVIEW_CLASS: &str = "min-h-40 px-3 py-2";
const EMPTY_PREVIEW_CLASS: &str = "text-sm text-gray-400";
const EMPTY_PREVIEW: &str = "Nothing to preview yet.";
const ACTIVE_MODE_CLASS: &str = "bg-gray-100 text-gray-900";
const MODE_SIZE: ButtonSize = ButtonSize::Dense;
const FORMAT_ICON_CLASS: &str = "h-4 w-4";
#[cfg(any(feature = "hydrate", test))]
const LINK_PLACEHOLDER_TEXT: &str = "link";
#[cfg(any(feature = "hydrate", test))]
const LINK_PLACEHOLDER_ADDRESS: &str = "https://";
#[cfg(any(feature = "hydrate", test))]
const LIST_MARKER: &str = "- ";

#[derive(Clone, Copy)]
enum Formatting {
    Bold,
    Italic,
    List,
    Link,
}

impl Formatting {
    const ALL: [Self; 4] = [Self::Bold, Self::Italic, Self::List, Self::Link];

    fn label(self) -> &'static str {
        match self {
            Self::Bold => "Bold",
            Self::Italic => "Italic",
            Self::List => "Bulleted List",
            Self::Link => "Link",
        }
    }

    fn glyph(self) -> icons::LucideGlyph {
        match self {
            Self::Bold => icons::LucideGlyph::Bold,
            Self::Italic => icons::LucideGlyph::Italic,
            Self::List => icons::LucideGlyph::List,
            Self::Link => icons::LucideGlyph::Link,
        }
    }

    #[cfg(any(feature = "hydrate", test))]
    fn wrap(self, selected: &str) -> String {
        match self {
            Self::Bold => format!("**{selected}**"),
            Self::Italic => format!("_{selected}_"),
            Self::List => selected
                .lines()
                .map(|line| format!("{LIST_MARKER}{line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Link => {
                let text = if selected.is_empty() {
                    LINK_PLACEHOLDER_TEXT
                } else {
                    selected
                };
                format!("[{text}]({LINK_PLACEHOLDER_ADDRESS})")
            }
        }
    }
}

#[cfg(any(feature = "hydrate", test))]
fn byte_offset(text: &str, utf16_offset: usize) -> usize {
    let mut units = 0_usize;
    for (byte_index, character) in text.char_indices() {
        if units >= utf16_offset {
            return byte_index;
        }
        units = units.saturating_add(character.len_utf16());
    }
    text.len()
}

#[cfg(any(feature = "hydrate", test))]
fn with_formatting(
    text: &str,
    start: usize,
    end: usize,
    formatting: Formatting,
) -> (String, usize) {
    let start_byte = byte_offset(text, start);
    let end_byte = byte_offset(text, end).max(start_byte);
    let before = text.get(..start_byte).unwrap_or_default();
    let selected = text.get(start_byte..end_byte).unwrap_or_default();
    let after = text.get(end_byte..).unwrap_or_default();
    let wrapped = formatting.wrap(selected);
    let cursor = start.saturating_add(wrapped.encode_utf16().count());
    (format!("{before}{wrapped}{after}"), cursor)
}

#[cfg(feature = "hydrate")]
fn apply_formatting(textarea: NodeRef<Textarea>, value: RwSignal<String>, formatting: Formatting) {
    let Some(element) = textarea.get_untracked() else {
        return;
    };
    let offset = |position: Result<Option<u32>, _>| {
        position
            .ok()
            .flatten()
            .and_then(|position| usize::try_from(position).ok())
            .unwrap_or_default()
    };
    let start = offset(element.selection_start());
    let end = offset(element.selection_end());
    let (formatted, cursor) =
        value.with_untracked(|text| with_formatting(text, start, end, formatting));
    value.set(formatted.clone());
    element.set_value(&formatted);
    let cursor = u32::try_from(cursor).unwrap_or_default();
    element
        .set_selection_range(cursor, cursor)
        .unwrap_or_default();
    element.focus().unwrap_or_default();
}

#[cfg(not(feature = "hydrate"))]
fn apply_formatting(
    _textarea: NodeRef<Textarea>,
    _value: RwSignal<String>,
    _formatting: Formatting,
) {
}

fn mode_button(
    label: &'static str,
    active: Signal<bool>,
    on_select: impl Fn() + 'static,
) -> AnyView {
    let class = move || {
        let base = button_class(ButtonTone::Subtle, MODE_SIZE);
        if active.get() {
            format!("{base} {ACTIVE_MODE_CLASS}")
        } else {
            base
        }
    };
    view! {
        <button
            type="button"
            class=class
            aria-pressed=move || active.get().to_string()
            on:click=move |_| on_select()
        >
            {label}
        </button>
    }
    .into_any()
}

fn formatting_buttons(textarea: NodeRef<Textarea>, value: RwSignal<String>) -> AnyView {
    Formatting::ALL
        .into_iter()
        .map(|formatting| {
            view! {
                <Tooltip label=formatting.label()>
                    <button
                        type="button"
                        class=icon_button_class(ButtonTone::Ghost, ButtonSize::Dense)
                        aria-label=formatting.label()
                        on:mousedown=|event| event.prevent_default()
                        on:click=move |_| apply_formatting(textarea, value, formatting)
                    >
                        {icons::icon(formatting.glyph(), FORMAT_ICON_CLASS)}
                    </button>
                </Tooltip>
            }
        })
        .collect_view()
        .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn MarkdownEditor(
    value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] invalid: Signal<bool>,
) -> impl IntoView {
    let invalid = with_field_error(invalid);
    let on_user_validity = user_validity_handler(field_interaction());
    let on_leave = on_user_validity.clone();
    let previewing = RwSignal::new(false);
    let textarea: NodeRef<Textarea> = NodeRef::new();
    let writing = Signal::derive(move || !previewing.get());
    let preview = move || {
        if value.with(|text| text.trim().is_empty()) {
            view! { <p class=EMPTY_PREVIEW_CLASS>{EMPTY_PREVIEW}</p> }.into_any()
        } else {
            view! { <MarkdownText text=value /> }.into_any()
        }
    };
    view! {
        <div class=EDITOR_CLASS>
            <div class=TOOLBAR_CLASS>
                <div class="flex items-center gap-1">
                    {mode_button("Write", writing, move || previewing.set(false))}
                    {mode_button("Preview", previewing.into(), move || previewing.set(true))}
                </div>
                <div class="flex items-center gap-0.5" class:hidden=move || previewing.get()>
                    {formatting_buttons(textarea, value)}
                </div>
            </div>
            <textarea
                node_ref=textarea
                id=field_control_id()
                aria-describedby=field_description_id()
                aria-invalid=move || invalid.get().to_string()
                class=TEXTAREA_CLASS
                class:hidden=move || previewing.get()
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |event: Event| value.set(event_target_value(&event))
                on:change=move |event: Event| on_user_validity(&event)
                on:blur=while_mounted(move |event: FocusEvent| on_leave(&event))
            ></textarea>
            <div class=PREVIEW_CLASS class:hidden=move || !previewing.get()>
                {preview}
            </div>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_the_selection_in_markdown() {
        assert_eq!(
            with_formatting("confirmed 13 crates", 10, 19, Formatting::Bold),
            ("confirmed **13 crates**".to_owned(), 23)
        );
        assert_eq!(
            with_formatting("see supplier", 4, 12, Formatting::Link),
            ("see [supplier](https://)".to_owned(), 24)
        );
    }

    #[test]
    fn turns_each_selected_line_into_a_list_item() {
        assert_eq!(
            with_formatting("one\ntwo", 0, 7, Formatting::List).0,
            "- one\n- two"
        );
    }

    #[test]
    fn counts_positions_the_way_the_browser_does() {
        assert_eq!(
            with_formatting("£12 é", 4, 5, Formatting::Italic).0,
            "£12 _é_"
        );
    }
}
