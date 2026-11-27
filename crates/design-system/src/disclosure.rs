use leptos::prelude::*;

use crate::icons;

const SUMMARY_CLASS: &str = "flex w-full cursor-pointer list-none items-start justify-between gap-2 rounded-md text-left focus-ring [&::-webkit-details-marker]:hidden";
const CHEVRON_CLASS: &str = "mt-0.5 h-4 w-4 shrink-0 text-gray-400 transition-transform group-hover/disclosure:text-gray-600 group-open/disclosure:rotate-180";

#[component]
pub fn Disclosure(
    #[prop(into)] summary: ViewFn,
    #[prop(optional)] open_at_start: bool,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <details class="group/disclosure" open=open_at_start>
            <summary class=SUMMARY_CLASS>
                <span class="min-w-0 flex-1">{move || summary.run()}</span>
                {icons::icon(icons::LucideGlyph::ChevronDown, CHEVRON_CLASS)}
            </summary>
            <div class="mt-2">{move || children()}</div>
        </details>
    }
    .into_any()
}
