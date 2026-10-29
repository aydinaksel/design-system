use leptos::prelude::*;

use crate::button::{button_icon_class, ButtonSize, SHARED_CLASS};
use crate::icons;

const UNSELECTED_CLASS: &str =
    "border border-gray-300 bg-white text-gray-700 shadow-sm hover:bg-gray-50 focus-visible:ring-gray-400";
const SELECTED_CLASS: &str =
    "border border-blue-300 bg-blue-50 text-blue-700 shadow-sm hover:bg-blue-100 focus-visible:ring-blue-500";

fn chip_class(selected: bool, size: ButtonSize) -> String {
    let tone = if selected {
        SELECTED_CLASS
    } else {
        UNSELECTED_CLASS
    };
    format!("{SHARED_CLASS} {} {tone}", size.class())
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FilterChip(
    #[prop(into)] label: String,
    #[prop(into)] selected: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
    #[prop(default = ButtonSize::Compact)] size: ButtonSize,
) -> impl IntoView {
    view! {
        <button
            type="button"
            aria-pressed=move || selected.get().to_string()
            class=move || chip_class(selected.get(), size)
            on:click=move |_| on_toggle.run(())
        >
            {move || {
                selected
                    .get()
                    .then(|| icons::icon(icons::LucideGlyph::Check, button_icon_class(size)))
            }}
            {label}
        </button>
    }
    .into_any()
}
