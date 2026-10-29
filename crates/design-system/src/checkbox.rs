use lepticons::{Icon, LucideGlyph};
use leptos::prelude::*;

const BOX_CLASS: &str = "peer size-4 cursor-pointer appearance-none rounded border border-gray-300 bg-white transition-colors hover:border-gray-400 checked:border-blue-600 checked:bg-blue-600 checked:hover:border-blue-700 checked:hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 disabled:cursor-not-allowed disabled:border-gray-200 disabled:bg-gray-100 disabled:checked:border-gray-300 disabled:checked:bg-gray-300";
const TICK_CLASS: &str =
    "pointer-events-none absolute size-3 text-white invisible peer-checked:visible";
const TICK_STROKE_WIDTH: &str = "3.5";

/// A native checkbox drawn to match the design system, so labels, keyboard
/// toggling and form semantics stay the browser's own.
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <span class="relative inline-flex size-4 shrink-0 items-center justify-center">
            <input
                type="checkbox"
                class=BOX_CLASS
                prop:checked=move || checked.get()
                prop:disabled=move || disabled.get()
                on:change=move |_| on_toggle.run(())
            />
            <Icon glyph=LucideGlyph::Check class=TICK_CLASS stroke_width=TICK_STROKE_WIDTH />
        </span>
    }
    .into_any()
}
