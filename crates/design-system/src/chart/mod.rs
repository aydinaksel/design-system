use leptos::prelude::*;

mod bar;
mod card;
mod column;
mod curve;
mod data_table;
mod frame;
mod heat_map;
mod hover_card;
mod label_layout;
mod line;
mod reference;
mod scale;
mod slope;
mod small_multiples;
mod stacked;
mod stat;
mod time_buckets;

pub use bar::{BarEntry, HorizontalBarChart};
pub use card::{ChartCard, ChartMetric};
pub use column::ColumnChart;
pub use heat_map::CalendarHeatMap;
pub use line::{ChartHeight, LineChart};
pub use reference::ReferenceLine;
pub use scale::ValueFormat;
pub use slope::{SlopeChart, SlopeEntry};
pub use small_multiples::SmallMultiples;
pub use stacked::{StackMode, StackedBarChart};
pub use stat::{StatChange, StatGrid, StatTile};
pub use time_buckets::{bucket_by_date, BucketedSeries, TimeGrain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartColor {
    Blue,
    Orange,
    Aqua,
    Yellow,
    Muted,
}

impl ChartColor {
    #[must_use]
    pub const fn is_muted(self) -> bool {
        matches!(self, Self::Muted)
    }

    #[must_use]
    pub const fn stroke_class(self) -> &'static str {
        match self {
            Self::Blue => "stroke-chart-1",
            Self::Orange => "stroke-chart-2",
            Self::Aqua => "stroke-chart-3",
            Self::Yellow => "stroke-chart-4",
            Self::Muted => "stroke-chart-muted",
        }
    }

    #[must_use]
    pub const fn background_class(self) -> &'static str {
        match self {
            Self::Blue => "bg-chart-1",
            Self::Orange => "bg-chart-2",
            Self::Aqua => "bg-chart-3",
            Self::Yellow => "bg-chart-4",
            Self::Muted => "bg-chart-muted",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AxisLabel {
    pub short: String,
    pub long: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    pub label: String,
    pub color: ChartColor,
    pub values: Vec<Option<f64>>,
}

pub(crate) const MISSING_VALUE_TEXT: &str = "No data";

impl ChartSeries {
    pub(crate) fn value_at(&self, index: usize) -> Option<f64> {
        self.values.get(index).copied().flatten()
    }

    pub(crate) fn present_values(&self) -> impl Iterator<Item = f64> + '_ {
        self.values.iter().copied().flatten()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct KeyedValue {
    pub color: ChartColor,
    pub label: String,
    pub value: f64,
}

pub(crate) fn keyed_value_row(entry: &KeyedValue, format: ValueFormat) -> AnyView {
    view! {
        <div class="flex items-center gap-2">
            {line_key(entry.color)} <span class="flex-1 text-gray-500">{entry.label.clone()}</span>
            <span class="font-semibold tabular-nums text-gray-900">{format.full(entry.value)}</span>
        </div>
    }
    .into_any()
}

pub(crate) fn line_key(color: ChartColor) -> impl IntoView {
    view! {
        <span
            aria-hidden="true"
            class=format!(
                "inline-block h-0.5 w-3 shrink-0 rounded-full {}",
                color.background_class(),
            )
        ></span>
    }
}

fn swatch(color: ChartColor) -> impl IntoView {
    view! {
        <span
            aria-hidden="true"
            class=format!(
                "inline-block size-2.5 shrink-0 rounded-[2px] {}",
                color.background_class(),
            )
        ></span>
    }
}

pub(crate) fn legend(series: &[ChartSeries]) -> AnyView {
    if series.len() < 2 {
        return ().into_any();
    }
    view! {
        <ul class="flex flex-wrap justify-center gap-x-6 gap-y-2 text-xs text-gray-500">
            {series
                .iter()
                .map(|entry| {
                    view! {
                        <li class="flex items-center gap-2">
                            {swatch(entry.color)} {entry.label.clone()}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
    .into_any()
}
