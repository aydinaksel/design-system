use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tone {
    #[default]
    Neutral,
    Informative,
    Positive,
    Caution,
    Critical,
}

impl Tone {
    #[must_use]
    pub const fn text_class(self) -> &'static str {
        match self {
            Tone::Neutral => "text-gray-900",
            Tone::Informative => "text-blue-700",
            Tone::Positive => "text-green-700",
            Tone::Caution => "text-amber-700",
            Tone::Critical => "text-red-700",
        }
    }

    #[must_use]
    pub const fn badge_class(self) -> &'static str {
        match self {
            Tone::Neutral => "bg-gray-100 text-gray-700",
            Tone::Informative => "bg-blue-100 text-blue-800",
            Tone::Positive => "bg-green-100 text-green-800",
            Tone::Caution => "bg-amber-100 text-amber-800",
            Tone::Critical => "bg-red-100 text-red-800",
        }
    }

    #[must_use]
    pub const fn surface_class(self) -> &'static str {
        match self {
            Tone::Neutral => "border-gray-200 bg-gray-50 text-gray-900",
            Tone::Informative => "border-blue-200 bg-blue-50 text-blue-900",
            Tone::Positive => "border-green-200 bg-green-50 text-green-900",
            Tone::Caution => "border-amber-200 bg-amber-50 text-amber-900",
            Tone::Critical => "border-red-200 bg-red-50 text-red-900",
        }
    }

    #[must_use]
    pub const fn icon_class(self) -> &'static str {
        match self {
            Tone::Neutral => "text-gray-500",
            Tone::Informative => "text-blue-600",
            Tone::Positive => "text-green-600",
            Tone::Caution => "text-amber-600",
            Tone::Critical => "text-red-600",
        }
    }

    #[must_use]
    pub const fn fill_class(self) -> &'static str {
        match self {
            Tone::Neutral => "bg-gray-400",
            Tone::Informative => "bg-blue-500",
            Tone::Positive => "bg-green-500",
            Tone::Caution => "bg-amber-500",
            Tone::Critical => "bg-red-500",
        }
    }

    #[must_use]
    pub fn of_signed(amount: f64) -> Tone {
        if amount < 0.0 {
            Tone::Critical
        } else {
            Tone::Neutral
        }
    }
}

#[must_use]
pub fn toned_text(text: impl Into<String>, tone: Tone) -> AnyView {
    let text = text.into();
    view! { <span class=tone.text_class()>{text}</span> }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_loss_reads_as_critical() {
        assert_eq!(Tone::of_signed(-0.01), Tone::Critical);
        assert_eq!(Tone::of_signed(0.0), Tone::Neutral);
        assert_eq!(Tone::of_signed(12.5), Tone::Neutral);
    }
}
