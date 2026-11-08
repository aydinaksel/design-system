use leptos::ev::{FocusEvent, MouseEvent};
use leptos::prelude::*;

use crate::checkbox::Checkbox;
use crate::tooltip::OverflowTooltip;

/// One value a filter offers, with how many rows carry it.
#[component]
pub fn FilterOption(
    label: String,
    #[prop(optional)] row_count: Option<u64>,
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
) -> impl IntoView {
    let full_label = OverflowTooltip::new();
    let bubble = full_label.bubble(&label);
    view! {
        <label
            class="group flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm hover:bg-gray-100"
            on:mouseover=move |event: MouseEvent| full_label.pointer_over(&event)
            on:mouseleave=move |_| full_label.hide()
            on:focusin=move |event: FocusEvent| full_label.focused(&event)
            on:focusout=move |_| full_label.hide()
        >
            <Checkbox checked=checked on_toggle=on_toggle />
            <span
                node_ref=full_label.anchor
                class="min-w-0 flex-1 truncate text-gray-600 group-hover:text-gray-900 group-has-checked:text-gray-900"
            >
                {label}
            </span>
            {row_count
                .map(|row_count| {
                    view! {
                        <span class="shrink-0 text-xs text-gray-400 tabular-nums">{row_count}</span>
                    }
                })}
            {bubble}
        </label>
    }
    .into_any()
}
