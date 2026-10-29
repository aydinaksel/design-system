use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeTone {
    #[default]
    Neutral,
    Positive,
    Informative,
    Caution,
}

impl BadgeTone {
    fn class(self) -> &'static str {
        match self {
            BadgeTone::Neutral => "bg-gray-100 text-gray-700",
            BadgeTone::Positive => "bg-green-100 text-green-800",
            BadgeTone::Informative => "bg-blue-100 text-blue-800",
            BadgeTone::Caution => "bg-amber-100 text-amber-800",
        }
    }
}

#[must_use]
pub fn badge(label: impl Into<String>, tone: BadgeTone) -> AnyView {
    let label = label.into();
    let class = format!(
        "inline-block rounded px-2 py-0.5 text-xs font-medium {}",
        tone.class()
    );
    view! { <span class=class>{label}</span> }.into_any()
}
