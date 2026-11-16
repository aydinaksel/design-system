use leptos::prelude::*;

use crate::element_key::ElementKey;
use crate::icons;

const BUTTON_CLASS: &str =
    "group flex w-full cursor-pointer items-start justify-between gap-2 rounded-md text-left focus-ring";
const CHEVRON_CLASS: &str =
    "mt-0.5 h-4 w-4 shrink-0 text-gray-400 transition-transform group-hover:text-gray-600";
const OPEN_CHEVRON_CLASS: &str =
    "mt-0.5 h-4 w-4 shrink-0 rotate-180 text-gray-400 transition-transform group-hover:text-gray-600";

#[component]
pub fn Disclosure(
    #[prop(into)] summary: ViewFn,
    #[prop(optional)] open_at_start: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let open = RwSignal::new(open_at_start);
    let panel_id = format!("disclosure-{}-panel", ElementKey::next());
    let chevron = move || {
        let class = if open.get() {
            OPEN_CHEVRON_CLASS
        } else {
            CHEVRON_CLASS
        };
        icons::icon(icons::LucideGlyph::ChevronDown, class)
    };
    view! {
        <div>
            <button
                type="button"
                class=BUTTON_CLASS
                aria-expanded=move || open.get().to_string()
                aria-controls=panel_id.clone()
                on:click=move |_| open.update(|is_open| *is_open = !*is_open)
            >
                <span class="min-w-0 flex-1">{move || summary.run()}</span>
                {chevron}
            </button>
            <div id=panel_id class="mt-2" class:hidden=move || !open.get()>
                {move || children()}
            </div>
        </div>
    }
    .into_any()
}
