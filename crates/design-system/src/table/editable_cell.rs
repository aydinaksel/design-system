use leptos::prelude::*;

use crate::icons;

const EDITABLE_BUTTON_CLASS: &str = "group inline-flex cursor-pointer items-center gap-2 rounded-sm text-left focus:outline-none focus:ring-2 focus:ring-blue-500";

const EDITABLE_TEXT_CLASS: &str = "border-b border-dotted border-gray-400 \
     group-hover:border-solid group-hover:border-blue-600 group-hover:text-blue-700";

const EMPTY_LABEL: &str = "+ Add";

pub fn editable_cell(
    text: String,
    tooltip: String,
    text_class: &'static str,
    begin_edit: impl Fn() + Send + Sync + 'static,
) -> AnyView {
    let label = if text.trim().is_empty() {
        view! { <span class="text-gray-400 group-hover:text-blue-700">{EMPTY_LABEL}</span> }
            .into_any()
    } else {
        view! { <span class=format!("{text_class} {EDITABLE_TEXT_CLASS}")>{text}</span> }.into_any()
    };
    view! {
        <button
            type="button"
            class=EDITABLE_BUTTON_CLASS
            title=tooltip
            on:click=move |event: leptos::ev::MouseEvent| {
                event.stop_propagation();
                begin_edit();
            }
        >
            {label}
            {icons::icon(
                icons::LucideGlyph::Pencil,
                "h-3 w-3 shrink-0 text-gray-400 group-hover:text-blue-600",
            )}
        </button>
    }
    .into_any()
}
