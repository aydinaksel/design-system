use leptos::prelude::*;

use super::data_table::series_table;
use super::frame::chart_name;
use super::hover_card::{HoverCard, HoverSide};
use super::reference::{covered_maximum, reference_overlay, ReferenceLine};
use super::scale::{ValueFormat, ValueScale};
use super::{keyed_value_row, legend, AxisLabel, ChartSeries, KeyedValue, MISSING_VALUE_TEXT};

const SINGLE_SERIES_GAP_TO_BAR_WIDTH: f64 = 0.6;
const CLUSTER_GAP_TO_BAR_WIDTH: f64 = 1.5;
const BAR_CLASS: &str = "flex-1 rounded-t-[4px] transition-opacity group-hover/column:opacity-80 group-focus-visible/column:opacity-80";
const GROUP_CLASS: &str = "group/column relative flex h-full flex-1 justify-center rounded-sm outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus";

fn category_gap_to_bar_width(series_count: usize) -> f64 {
    if series_count > 1 {
        CLUSTER_GAP_TO_BAR_WIDTH
    } else {
        SINGLE_SERIES_GAP_TO_BAR_WIDTH
    }
}

fn bar_row_width_percent(series_count: usize) -> f64 {
    let bars = f64::from(u32::try_from(series_count.max(1)).unwrap_or(u32::MAX));
    bars / (bars + category_gap_to_bar_width(series_count)) * 100.0
}

fn highest_value(series: &[ChartSeries]) -> f64 {
    series
        .iter()
        .flat_map(ChartSeries::present_values)
        .fold(0.0, f64::max)
}

fn gridlines(scale: &ValueScale) -> AnyView {
    scale
        .ticks
        .iter()
        .enumerate()
        .map(|(position, tick)| {
            let tone = if position == 0 {
                "bg-gray-300"
            } else {
                "bg-gray-200"
            };
            view! {
                <span
                    class=format!("absolute inset-x-0 h-px {tone}")
                    style=format!("top: {:.3}%", scale.percent_from_top(*tick))
                ></span>
            }
        })
        .collect_view()
        .into_any()
}

fn value_axis(scale: &ValueScale, format: ValueFormat) -> AnyView {
    scale
        .ticks
        .iter()
        .map(|tick| {
            let style = format!(
                "top: {:.3}%; transform: translateY(-50%)",
                scale.percent_from_top(*tick)
            );
            view! {
                <span class="absolute right-0" style=style>
                    {format.compact(*tick)}
                </span>
            }
        })
        .collect_view()
        .into_any()
}

fn column_group(
    index: usize,
    label: &AxisLabel,
    series: &[ChartSeries],
    scale: &ValueScale,
    format: ValueFormat,
) -> AnyView {
    let values: Vec<KeyedValue> = series
        .iter()
        .filter_map(|column| {
            column.value_at(index).map(|value| KeyedValue {
                color: column.color,
                label: column.label.clone(),
                value,
            })
        })
        .collect();
    let bars = values
        .iter()
        .map(|entry| {
            let height_percent = (entry.value / scale.top * 100.0).clamp(0.0, 100.0);
            view! {
                <span
                    class=format!("{BAR_CLASS} {}", entry.color.background_class())
                    style=format!("height: {height_percent:.3}%")
                ></span>
            }
        })
        .collect_view();
    let bar_row_style = format!("width: {:.3}%", bar_row_width_percent(series.len()));
    let hover = HoverCard::new();
    let heading = label.long.clone();
    let card = hover.card(HoverSide::Beside, move || {
        view! {
            <div class="font-medium text-gray-900">{heading.clone()}</div>
            {values.iter().map(|entry| keyed_value_row(entry, format)).collect_view()}
            {values
                .is_empty()
                .then(|| view! { <div class="text-gray-400">{MISSING_VALUE_TEXT}</div> })}
        }
        .into_any()
    });
    view! {
        <div
            node_ref=hover.anchor
            class=GROUP_CLASS
            tabindex="0"
            aria-label=label.long.clone()
            on:pointerenter=move |_| hover.show()
            on:pointerleave=move |_| hover.hide()
            on:focus=move |_| hover.show()
            on:blur=move |_| hover.hide()
        >
            <div class="flex h-full items-end gap-0.5" style=bar_row_style>
                {bars}
            </div>
            {card}
        </div>
    }
    .into_any()
}

fn plot(
    labels: &[AxisLabel],
    series: &[ChartSeries],
    reference: Option<&ReferenceLine>,
    format: ValueFormat,
) -> AnyView {
    let scale = ValueScale::covering(covered_maximum(highest_value(series), reference));
    let groups = labels
        .iter()
        .enumerate()
        .map(|(index, label)| column_group(index, label, series, &scale, format))
        .collect_view();
    let category_labels = labels
        .iter()
        .map(|label| {
            view! { <span class="flex-1 truncate text-center">{label.short.clone()}</span> }
        })
        .collect_view();
    view! {
        <div class="flex h-64 gap-3">
            <div class="relative w-12 shrink-0 text-right text-xs tabular-nums text-gray-500">
                {value_axis(&scale, format)}
            </div>
            <div class="relative flex-1">
                {gridlines(&scale)} <div class="absolute inset-0 flex">{groups}</div>
                {reference_overlay(reference, &scale, format)}
            </div>
        </div>
        <div class="mt-2 ml-15 flex text-xs text-gray-500">{category_labels}</div>
    }
    .into_any()
}

fn chart_body(
    caption: &str,
    labels: &[AxisLabel],
    series: &[ChartSeries],
    reference: Option<&ReferenceLine>,
    format: ValueFormat,
    empty_message: &'static str,
) -> AnyView {
    if labels.is_empty() {
        return view! { <p class="flex h-64 items-center justify-center text-sm text-gray-500">{empty_message}</p> }
        .into_any();
    }
    view! {
        {plot(labels, series, reference, format)}
        <div class="mt-4">{legend(series)}</div>
        {series_table(caption, labels, series, format)}
    }
    .into_any()
}

#[component]
pub fn ColumnChart(
    #[prop(into)] labels: Signal<Vec<AxisLabel>>,
    #[prop(into)] series: Signal<Vec<ChartSeries>>,
    format: ValueFormat,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] reference: Option<ReferenceLine>,
    #[prop(default = "No data in this range.")] empty_message: &'static str,
) -> impl IntoView {
    let name = chart_name(description);
    let caption = name.caption.clone();
    let body = move || {
        labels.with(|axis| {
            series.with(|columns| {
                chart_body(
                    &caption,
                    axis,
                    columns,
                    reference.as_ref(),
                    format,
                    empty_message,
                )
            })
        })
    };
    view! {
        <figure aria-label=name.label aria-labelledby=name.labelled_by>
            {body}
        </figure>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::bar_row_width_percent;

    fn bar_and_gap_percent(series_count: usize) -> (f64, f64) {
        let row_percent = bar_row_width_percent(series_count);
        let bars = f64::from(u32::try_from(series_count).unwrap_or(u32::MAX));
        (row_percent / bars, 100.0 - row_percent)
    }

    #[test]
    fn single_bars_sit_closer_than_their_own_width() {
        let (bar_percent, gap_percent) = bar_and_gap_percent(1);
        assert!(gap_percent < bar_percent);
    }

    #[test]
    fn clusters_sit_further_apart_than_one_bar() {
        for series_count in 2..=4 {
            let (bar_percent, gap_percent) = bar_and_gap_percent(series_count);
            assert!(gap_percent > bar_percent, "{series_count} series");
        }
    }
}
