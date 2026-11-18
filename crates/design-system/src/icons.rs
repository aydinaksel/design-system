use lepticons::Icon;
use leptos::prelude::*;

use crate::tooltip::Tooltip;

pub use lepticons::LucideGlyph;

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

pub fn labelled_icon(glyph: LucideGlyph, class: impl Into<String>, title: &'static str) -> AnyView {
    let class = class.into();
    view! {
        <Tooltip label=title>
            <span class="inline-flex" role="img" aria-label=title>
                <Icon glyph=glyph class=class />
            </span>
        </Tooltip>
    }
    .into_any()
}
