use leptos::prelude::*;

use crate::icons::{icon, LucideGlyph};

#[derive(Clone, Debug, PartialEq)]
pub struct StatChange {
    pub percent: f64,
    pub comparison: String,
    pub higher_is_better: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Flat,
}

impl StatChange {
    fn direction(&self) -> Direction {
        if self.percent.abs() < 0.05 {
            Direction::Flat
        } else if self.percent > 0.0 {
            Direction::Up
        } else {
            Direction::Down
        }
    }

    fn badge_tone(&self) -> &'static str {
        let improved = match self.direction() {
            Direction::Flat => return "border-gray-200 text-gray-700",
            Direction::Up => self.higher_is_better,
            Direction::Down => !self.higher_is_better,
        };
        if improved {
            "border-green-200 text-green-700"
        } else {
            "border-red-200 text-red-700"
        }
    }

    fn signed_percent(&self) -> String {
        match self.direction() {
            Direction::Flat => "0%".to_owned(),
            Direction::Up | Direction::Down => format!("{:+.1}%", self.percent),
        }
    }

    fn summary(&self) -> String {
        let magnitude = format!("{:.1}%", self.percent.abs());
        match self.direction() {
            Direction::Up => format!("Up {magnitude} on {}", self.comparison),
            Direction::Down => format!("Down {magnitude} on {}", self.comparison),
            Direction::Flat => format!("Level with {}", self.comparison),
        }
    }

    fn glyph(&self) -> Option<LucideGlyph> {
        match self.direction() {
            Direction::Up => Some(LucideGlyph::TrendingUp),
            Direction::Down => Some(LucideGlyph::TrendingDown),
            Direction::Flat => None,
        }
    }
}

fn change_badge(change: &StatChange) -> AnyView {
    view! {
        <span class=format!(
            "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium tabular-nums {}",
            change.badge_tone(),
        )>{change.glyph().map(|glyph| icon(glyph, "size-3.5"))} {change.signed_percent()}</span>
    }
    .into_any()
}

fn change_summary(change: &StatChange) -> AnyView {
    view! {
        <div class="flex items-center gap-2 font-medium text-gray-900">
            {change.summary()} {change.glyph().map(|glyph| icon(glyph, "size-4"))}
        </div>
    }
    .into_any()
}

#[component]
pub fn StatTile(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional_no_strip)] change: Option<StatChange>,
    #[prop(optional, into)] footnote: Option<String>,
) -> impl IntoView {
    let (badge, summary) = change
        .map(|change| (change_badge(&change), change_summary(&change)))
        .unzip();
    view! {
        <section class="flex flex-col justify-between gap-6 rounded-surface border border-gray-200 bg-surface p-6 shadow-xs">
            <header class="space-y-1.5">
                <div class="flex items-center justify-between gap-3">
                    <h3 class="text-sm text-gray-500">{label}</h3>
                    {badge}
                </div>
                <p class="text-2xl font-semibold text-gray-900 sm:text-3xl">{value}</p>
            </header>
            <footer class="space-y-1.5 text-sm">
                {summary} {footnote.map(|text| view! { <p class="text-gray-500">{text}</p> })}
            </footer>
        </section>
    }
    .into_any()
}

#[component]
pub fn StatGrid(children: Children) -> impl IntoView {
    view! { <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">{children()}</div> }
}

#[cfg(test)]
mod tests {
    use super::StatChange;

    fn change(percent: f64, higher_is_better: bool) -> StatChange {
        StatChange {
            percent,
            comparison: "Q3 2025".to_owned(),
            higher_is_better,
        }
    }

    #[test]
    fn a_rise_reads_as_good_only_when_higher_is_better() {
        assert_eq!(
            change(8.4, true).badge_tone(),
            "border-green-200 text-green-700"
        );
        assert_eq!(
            change(8.4, false).badge_tone(),
            "border-red-200 text-red-700"
        );
    }

    #[test]
    fn the_summary_names_the_comparison_period() {
        assert_eq!(change(-4.21, true).summary(), "Down 4.2% on Q3 2025");
        assert_eq!(change(0.01, true).summary(), "Level with Q3 2025");
        assert_eq!(change(12.5, true).signed_percent(), "+12.5%");
    }
}
