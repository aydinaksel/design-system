use leptos::prelude::*;

use super::data_table::series_table;
use super::frame::chart_name;
use super::label_layout::spread_label_positions;
use super::scale::ValueFormat;
use super::{AxisLabel, ChartColor, ChartSeries};

const LABEL_SPACING_PERCENT: f64 = 7.0;
const LABEL_COLUMN_CLASS: &str = "relative w-48 shrink-0 text-xs text-gray-700";

#[derive(Clone, Debug, PartialEq)]
pub struct SlopeEntry {
    pub label: String,
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    From,
    To,
}

impl SlopeEntry {
    fn value(&self, side: Side) -> f64 {
        match side {
            Side::From => self.from,
            Side::To => self.to,
        }
    }

    fn color(&self) -> ChartColor {
        if self.to < self.from {
            ChartColor::Orange
        } else {
            ChartColor::Blue
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ValueRange {
    lowest: f64,
    highest: f64,
}

impl ValueRange {
    fn covering(entries: &[SlopeEntry]) -> Self {
        let values = entries.iter().flat_map(|entry| [entry.from, entry.to]);
        let (lowest, highest) = values
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                (low.min(value), high.max(value))
            });
        Self { lowest, highest }
    }

    fn percent_from_top(self, value: f64) -> f64 {
        let span = self.highest - self.lowest;
        if span.is_finite() && span > 0.0 {
            (self.highest - value) / span * 100.0
        } else {
            50.0
        }
    }
}

fn side_labels(
    entries: &[SlopeEntry],
    range: ValueRange,
    side: Side,
    format: ValueFormat,
) -> AnyView {
    let mut placed: Vec<(f64, &SlopeEntry)> = entries
        .iter()
        .map(|entry| (range.percent_from_top(entry.value(side)), entry))
        .collect();
    placed.sort_by(|left, right| left.0.total_cmp(&right.0));
    let desired: Vec<f64> = placed.iter().map(|(top_percent, _)| *top_percent).collect();
    let alignment = match side {
        Side::From => "justify-end text-right",
        Side::To => "flex-row-reverse justify-end text-left",
    };
    placed
        .iter()
        .zip(spread_label_positions(&desired, LABEL_SPACING_PERCENT))
        .map(|((_, entry), top_percent)| {
            view! {
                <span
                    class=format!("absolute inset-x-0 flex -translate-y-1/2 gap-2 {alignment}")
                    style=format!("top: {top_percent:.3}%")
                >
                    <span class="truncate" title=entry.label.clone()>
                        {entry.label.clone()}
                    </span>
                    <span class="font-semibold tabular-nums text-gray-900">
                        {format.compact(entry.value(side))}
                    </span>
                </span>
            }
        })
        .collect_view()
        .into_any()
}

fn slopes(entries: &[SlopeEntry], range: ValueRange) -> AnyView {
    let lines = entries
        .iter()
        .map(|entry| {
            view! {
                <line
                    x1="0"
                    x2="100"
                    y1=range.percent_from_top(entry.from)
                    y2=range.percent_from_top(entry.to)
                    class=entry.color().stroke_class()
                    stroke-width="2"
                    stroke-linecap="round"
                    vector-effect="non-scaling-stroke"
                />
            }
        })
        .collect_view();
    let ends = entries
        .iter()
        .flat_map(|entry| {
            [
                (0, entry.from, entry.color()),
                (100, entry.to, entry.color()),
            ]
        })
        .map(|(left_percent, value, color)| {
            view! {
                <span
                    class=format!(
                        "absolute size-2 -translate-x-1/2 -translate-y-1/2 rounded-full {}",
                        color.background_class(),
                    )
                    style=format!(
                        "left: {left_percent}%; top: {:.3}%",
                        range.percent_from_top(value),
                    )
                ></span>
            }
        })
        .collect_view();
    view! {
        <svg
            class="absolute inset-0 h-full w-full overflow-visible"
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
        >
            {lines}
        </svg>
        {ends}
    }
    .into_any()
}

fn table_shape(
    entries: &[SlopeEntry],
    from_label: &str,
    to_label: &str,
) -> (Vec<AxisLabel>, Vec<ChartSeries>) {
    let labels = entries
        .iter()
        .map(|entry| AxisLabel {
            short: entry.label.clone(),
            long: entry.label.clone(),
        })
        .collect();
    let column = |label: &str, side: Side| ChartSeries {
        label: label.to_owned(),
        color: ChartColor::Blue,
        values: entries
            .iter()
            .map(|entry| Some(entry.value(side)))
            .collect(),
    };
    (
        labels,
        vec![column(from_label, Side::From), column(to_label, Side::To)],
    )
}

#[component]
pub fn SlopeChart(
    #[prop(into)] entries: Signal<Vec<SlopeEntry>>,
    #[prop(into)] from_label: String,
    #[prop(into)] to_label: String,
    format: ValueFormat,
    #[prop(optional, into)] description: Option<String>,
) -> impl IntoView {
    let name = chart_name(description);
    let caption = name.caption.clone();
    let table_from = from_label.clone();
    let table_to = to_label.clone();
    let body = move || {
        entries.with(|rows| {
            let range = ValueRange::covering(rows);
            let (labels, series) = table_shape(rows, &table_from, &table_to);
            view! {
                <div aria-hidden="true" class="flex h-72 gap-4">
                    <div class=LABEL_COLUMN_CLASS>
                        {side_labels(rows, range, Side::From, format)}
                    </div>
                    <div class="relative flex-1">{slopes(rows, range)}</div>
                    <div class=LABEL_COLUMN_CLASS>{side_labels(rows, range, Side::To, format)}</div>
                </div>
                {series_table(&caption, &labels, &series, format)}
            }
            .into_any()
        })
    };
    view! {
        <figure aria-label=name.label aria-labelledby=name.labelled_by>
            <div
                aria-hidden="true"
                class="mb-4 flex justify-between gap-4 text-xs font-semibold text-gray-900"
            >
                <span class="w-48 text-right">{from_label}</span>
                <span class="w-48">{to_label}</span>
            </div>
            {body}
        </figure>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::{SlopeEntry, ValueRange};

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < f64::EPSILON
    }

    fn entry(from: f64, to: f64) -> SlopeEntry {
        SlopeEntry {
            label: String::new(),
            from,
            to,
        }
    }

    #[test]
    fn the_range_spans_both_periods() {
        let range = ValueRange::covering(&[entry(10.0, 30.0), entry(20.0, 5.0)]);
        assert!(close(range.percent_from_top(30.0), 0.0));
        assert!(close(range.percent_from_top(5.0), 100.0));
    }

    #[test]
    fn a_single_value_sits_in_the_middle() {
        let range = ValueRange::covering(&[entry(7.0, 7.0)]);
        assert!(close(range.percent_from_top(7.0), 50.0));
        assert!(close(ValueRange::covering(&[]).percent_from_top(1.0), 50.0));
    }
}
