use leptos::prelude::*;

mod bar;
mod card;
mod column;
mod curve;
mod data_table;
mod line;
mod scale;
mod stat;
mod time_buckets;

pub use bar::{BarEntry, HorizontalBarChart};
pub use card::{ChartCard, ChartMetric};
pub use column::ColumnChart;
pub use line::LineChart;
pub use scale::ValueFormat;
pub use stat::{StatChange, StatGrid, StatTile};
pub use time_buckets::{bucket_by_date, BucketedSeries, TimeGrain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartColor {
    Blue,
    Orange,
    Aqua,
    Yellow,
}

impl ChartColor {
    #[must_use]
    pub const fn stroke_class(self) -> &'static str {
        match self {
            Self::Blue => "stroke-chart-1",
            Self::Orange => "stroke-chart-2",
            Self::Aqua => "stroke-chart-3",
            Self::Yellow => "stroke-chart-4",
        }
    }

    #[must_use]
    pub const fn background_class(self) -> &'static str {
        match self {
            Self::Blue => "bg-chart-1",
            Self::Orange => "bg-chart-2",
            Self::Aqua => "bg-chart-3",
            Self::Yellow => "bg-chart-4",
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
    pub values: Vec<f64>,
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
        <ul class="mt-4 flex flex-wrap justify-center gap-x-6 gap-y-2 text-xs text-gray-500">
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
