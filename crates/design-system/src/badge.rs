use leptos::prelude::*;

use crate::tone::Tone;

#[must_use]
pub fn badge(label: impl Into<String>, tone: Tone) -> AnyView {
    let label = label.into();
    let class = format!(
        "inline-block rounded px-2 py-0.5 text-xs font-medium {}",
        tone.badge_class()
    );
    view! { <span class=class>{label}</span> }.into_any()
}
