use leptos::prelude::*;

use crate::icons::{self, LucideGlyph};

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(default = LucideGlyph::Inbox)] glyph: LucideGlyph,
    #[prop(optional)] action: Option<AnyView>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center gap-3 rounded-surface border border-dashed border-gray-300 bg-surface px-4 py-16 text-center">
            {icons::icon(glyph, "h-8 w-8 text-gray-400")} <div class="space-y-1">
                <p class="text-sm font-medium text-gray-900">{title}</p>
                {description
                    .map(|description| {
                        view! { <p class="text-sm text-gray-500">{description}</p> }
                    })}
            </div> {action}
        </div>
    }
    .into_any()
}
