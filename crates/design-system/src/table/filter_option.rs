use leptos::prelude::*;

use crate::checkbox::Checkbox;

/// One value a filter offers, with how many rows carry it.
#[component]
pub fn FilterOption(
    label: String,
    #[prop(optional)] row_count: Option<u64>,
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
) -> impl IntoView {
    let title = label.clone();
    view! {
        <label class="group flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm hover:bg-gray-100">
            <Checkbox checked=checked on_toggle=on_toggle />
            <span
                class="min-w-0 flex-1 truncate text-gray-600 group-hover:text-gray-900 group-has-checked:text-gray-900"
                title=title
            >
                {label}
            </span>
            {row_count
                .map(|row_count| {
                    view! {
                        <span class="shrink-0 text-xs text-gray-400 tabular-nums">{row_count}</span>
                    }
                })}
        </label>
    }
    .into_any()
}
