use leptos::prelude::*;

pub const LABEL_CLASS: &str = "text-xs font-medium uppercase tracking-wide text-gray-500";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeadingLevel {
    #[default]
    Two,
    Three,
    Four,
}

impl HeadingLevel {
    #[must_use]
    pub const fn below(self) -> Self {
        match self {
            Self::Two => Self::Three,
            Self::Three | Self::Four => Self::Four,
        }
    }
}

pub(crate) fn heading_element(
    level: HeadingLevel,
    id: Option<String>,
    class: &'static str,
    text: String,
) -> AnyView {
    match level {
        HeadingLevel::Two => view! {
            <h2 id=id class=class>
                {text}
            </h2>
        }
        .into_any(),
        HeadingLevel::Three => view! {
            <h3 id=id class=class>
                {text}
            </h3>
        }
        .into_any(),
        HeadingLevel::Four => view! {
            <h4 id=id class=class>
                {text}
            </h4>
        }
        .into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::HeadingLevel;

    #[test]
    fn a_heading_below_stops_at_the_deepest_level() {
        assert_eq!(HeadingLevel::Two.below(), HeadingLevel::Three);
        assert_eq!(HeadingLevel::Four.below(), HeadingLevel::Four);
    }
}
