use leptos::prelude::*;

use crate::icons::{self, LucideGlyph};
use crate::tone::Tone;

fn glyph(tone: Tone) -> LucideGlyph {
    match tone {
        Tone::Neutral | Tone::Informative => LucideGlyph::Info,
        Tone::Positive => LucideGlyph::CircleCheck,
        Tone::Caution => LucideGlyph::TriangleAlert,
        Tone::Critical => LucideGlyph::CircleAlert,
    }
}

fn role(tone: Tone) -> &'static str {
    match tone {
        Tone::Critical => "alert",
        Tone::Neutral | Tone::Informative | Tone::Positive | Tone::Caution => "status",
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Alert(
    #[prop(optional)] tone: Tone,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let title_view = title.map(|title| view! { <p class="font-medium">{title}</p> });
    let body = children.map(|children| view! { <div class="space-y-1">{children()}</div> });
    view! {
        <div
            role=role(tone)
            class=format!("flex gap-3 rounded-md border px-4 py-3 text-sm {}", tone.surface_class())
        >
            {icons::icon(glyph(tone), format!("mt-0.5 h-4 w-4 shrink-0 {}", tone.icon_class()))}
            <div class="min-w-0 flex-1 space-y-1">{title_view} {body}</div>
        </div>
    }
    .into_any()
}
