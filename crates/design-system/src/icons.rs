use lepticons::Icon;
use leptos::prelude::*;

pub use lepticons::LucideGlyph;

/// A [Lucide](https://lucide.dev) glyph sized and coloured by the caller's
/// classes, so an icon inherits the text colour of whatever it sits next to.
pub fn icon(glyph: LucideGlyph, class: impl Into<String>) -> AnyView {
    let class = class.into();
    view! { <Icon glyph=glyph class=class /> }.into_any()
}

pub fn spinner(class: impl Into<String>) -> AnyView {
    let class = format!("animate-spin {}", class.into());
    view! { <Icon glyph=LucideGlyph::LoaderCircle class=class /> }.into_any()
}

#[must_use]
pub fn loading_indicator() -> AnyView {
    view! {
        <div class="flex justify-center py-8" role="status" aria-label="Loading">
            <Icon glyph=LucideGlyph::LoaderCircle class="h-5 w-5 animate-spin text-gray-400" />
        </div>
    }
    .into_any()
}

/// An icon that carries meaning no neighbouring text repeats, so it needs a
/// title for screen readers and as a hover tooltip.
pub fn labelled_icon(glyph: LucideGlyph, class: impl Into<String>, title: &'static str) -> AnyView {
    let class = class.into();
    view! {
        <span class="inline-flex" title=title role="img" aria-label=title>
            <Icon glyph=glyph class=class />
        </span>
    }
    .into_any()
}
