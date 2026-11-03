use leptos::prelude::*;

use crate::icons;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn LoadingState(#[prop(optional)] class: &'static str) -> impl IntoView {
    view! {
        <div
            role="status"
            aria-label="Loading"
            class=format!(
                "flex items-center justify-center rounded-surface border border-gray-200 bg-gray-50 {class}",
            )
        >
            {icons::spinner("h-6 w-6 text-gray-400")}
        </div>
    }
}
