use leptos::prelude::*;

use crate::icons::{icon, LucideGlyph};
use crate::tone::Tone;

const FLAT_CHANGE_PERCENT: f64 = 0.05;
const FLAT_TONE_CLASS: &str = "text-gray-500";

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
        if self.percent.abs() < FLAT_CHANGE_PERCENT {
            Direction::Flat
        } else if self.percent > 0.0 {
            Direction::Up
        } else {
            Direction::Down
        }
    }

    fn tone(&self) -> Option<Tone> {
        let improved = match self.direction() {
            Direction::Flat => return None,
            Direction::Up => self.higher_is_better,
            Direction::Down => !self.higher_is_better,
        };
        Some(if improved {
            Tone::Positive
        } else {
            Tone::Critical
        })
    }

    fn tone_class(&self) -> &'static str {
        self.tone().map_or(FLAT_TONE_CLASS, Tone::text_class)
    }

    fn movement(&self) -> String {
        match self.direction() {
            Direction::Flat => "No change".to_owned(),
            Direction::Up | Direction::Down => format!("{:.1}%", self.percent.abs()),
        }
    }

    fn spoken_direction(&self) -> &'static str {
        match self.direction() {
            Direction::Up => "Up ",
            Direction::Down => "Down ",
            Direction::Flat => "",
        }
    }

    fn glyph(&self) -> Option<LucideGlyph> {
        match self.direction() {
            Direction::Up => Some(LucideGlyph::ArrowUp),
            Direction::Down => Some(LucideGlyph::ArrowDown),
            Direction::Flat => None,
        }
    }
}

fn change_line(change: &StatChange) -> AnyView {
    let glyph = change
        .glyph()
        .map(|glyph| view! { <span aria-hidden="true">{icon(glyph, "size-3.5")}</span> });
    view! {
        <p class="flex flex-wrap items-center gap-x-1.5 text-sm">
            <span class=format!(
                "inline-flex items-center gap-0.5 font-medium tabular-nums {}",
                change.tone_class(),
            )>
                {glyph} <span class="sr-only">{change.spoken_direction()}</span> {change.movement()}
            </span>
            <span class="text-gray-500">{format!("vs {}", change.comparison)}</span>
        </p>
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
    view! {
        <div class="flex flex-col gap-1 rounded-surface border border-gray-200 bg-surface p-5">
            <p class="text-sm text-gray-500">{label}</p>
            <p class="text-3xl font-semibold tracking-tight tabular-nums text-gray-900">{value}</p>
            {change.map(|change| change_line(&change))}
            {footnote.map(|text| view! { <p class="mt-2 text-xs text-gray-500">{text}</p> })}
        </div>
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
    use crate::tone::Tone;

    fn change(percent: f64, higher_is_better: bool) -> StatChange {
        StatChange {
            percent,
            comparison: "Q3 2025".to_owned(),
            higher_is_better,
        }
    }

    #[test]
    fn a_rise_reads_as_good_only_when_higher_is_better() {
        assert_eq!(change(8.4, true).tone(), Some(Tone::Positive));
        assert_eq!(change(8.4, false).tone(), Some(Tone::Critical));
        assert_eq!(change(0.0, true).tone(), None);
    }

    #[test]
    fn the_movement_shows_magnitude_and_speaks_direction() {
        assert_eq!(change(-4.21, true).movement(), "4.2%");
        assert_eq!(change(-4.21, true).spoken_direction(), "Down ");
        assert_eq!(change(0.01, true).movement(), "No change");
    }
}
