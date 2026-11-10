use leptos::prelude::*;

use super::data_table::series_table;
use super::frame::chart_name;
use super::scale::ValueFormat;
use super::{legend, line_key, AxisLabel, ChartSeries};

const LONGEST_BAR_PERCENT: f64 = 82.0;
const ROW_CLASS: &str = "group/stack relative grid grid-cols-[minmax(0,11rem)_1fr] items-center gap-3 rounded-sm py-1 outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus";
const SEGMENT_CLASS: &str = "h-full min-w-px first:rounded-l-[4px] last:rounded-r-[4px] transition-opacity group-hover/stack:opacity-80 group-focus-visible/stack:opacity-80";
const TOOLTIP_CLASS: &str = "pointer-events-none absolute bottom-full left-48 z-10 mb-1 hidden min-w-52 space-y-1.5 rounded-item border border-gray-200 bg-surface px-3 py-2 text-xs shadow-lg group-hover/stack:block group-focus-visible/stack:block";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackMode {
    #[default]
    Total,
    Share,
}

fn row_values(series: &[ChartSeries], index: usize) -> Vec<f64> {
    series
        .iter()
        .map(|part| part.values.get(index).copied().unwrap_or_default().max(0.0))
        .collect()
}

fn row_total(series: &[ChartSeries], index: usize) -> f64 {
    row_values(series, index).iter().sum()
}

fn bar_width_percent(mode: StackMode, total: f64, longest_total: f64) -> f64 {
    match mode {
        StackMode::Share => 100.0,
        StackMode::Total if longest_total > 0.0 => total / longest_total * LONGEST_BAR_PERCENT,
        StackMode::Total => 0.0,
    }
}

fn share_percent(value: f64, total: f64) -> f64 {
    if total > 0.0 {
        value / total * 100.0
    } else {
        0.0
    }
}

fn tooltip(
    label: &AxisLabel,
    series: &[ChartSeries],
    index: usize,
    format: ValueFormat,
) -> AnyView {
    let total = row_total(series, index);
    let rows = series
        .iter()
        .zip(row_values(series, index))
        .map(|(part, value)| {
            view! {
                <div class="flex items-center gap-2">
                    {line_key(part.color)}
                    <span class="flex-1 text-gray-500">{part.label.clone()}</span>
                    <span class="font-semibold tabular-nums text-gray-900">
                        {format.full(value)}
                    </span>
                    <span class="w-10 text-right tabular-nums text-gray-500">
                        {format!("{:.0}%", share_percent(value, total))}
                    </span>
                </div>
            }
        })
        .collect_view();
    view! {
        <div class=TOOLTIP_CLASS>
            <div class="font-medium text-gray-900">{label.long.clone()}</div>
            {rows}
            <div class="flex justify-between border-t border-gray-200 pt-1.5 font-medium text-gray-900">
                <span>"Total"</span>
                <span class="tabular-nums">{format.full(total)}</span>
            </div>
        </div>
    }
    .into_any()
}

fn stacked_row(
    index: usize,
    label: &AxisLabel,
    series: &[ChartSeries],
    mode: StackMode,
    longest_total: f64,
    format: ValueFormat,
) -> AnyView {
    let total = row_total(series, index);
    let segments = series
        .iter()
        .zip(row_values(series, index))
        .map(|(part, value)| {
            view! {
                <span
                    class=format!("{SEGMENT_CLASS} {}", part.color.background_class())
                    style=format!("flex: {value} 1 0")
                ></span>
            }
        })
        .collect_view();
    let total_label = (mode == StackMode::Total).then(|| {
        view! {
            <span class="ml-2 text-sm font-medium tabular-nums text-gray-900">
                {format.compact(total)}
            </span>
        }
    });
    view! {
        <div class=ROW_CLASS tabindex="0">
            <span class="truncate text-sm text-gray-700" title=label.long.clone()>
                {label.short.clone()}
            </span>
            <span class="flex h-8 items-center">
                <span
                    class="flex h-full gap-0.5"
                    style=format!("width: {:.3}%", bar_width_percent(mode, total, longest_total))
                >
                    {segments}
                </span>
                {total_label}
            </span>
            {tooltip(label, series, index, format)}
        </div>
    }
    .into_any()
}

fn chart_body(
    caption: &str,
    labels: &[AxisLabel],
    series: &[ChartSeries],
    mode: StackMode,
    format: ValueFormat,
    empty_message: &'static str,
) -> AnyView {
    if labels.is_empty() {
        return view! { <p class="py-8 text-center text-sm text-gray-500">{empty_message}</p> }
            .into_any();
    }
    let longest_total = (0..labels.len())
        .map(|index| row_total(series, index))
        .fold(0.0, f64::max);
    let rows = labels
        .iter()
        .enumerate()
        .map(|(index, label)| stacked_row(index, label, series, mode, longest_total, format))
        .collect_view();
    view! {
        <div class="mb-3">{legend(series)}</div>
        <div class="space-y-1">{rows}</div>
        {series_table(caption, labels, series, format)}
    }
    .into_any()
}

#[component]
pub fn StackedBarChart(
    #[prop(into)] labels: Signal<Vec<AxisLabel>>,
    #[prop(into)] series: Signal<Vec<ChartSeries>>,
    format: ValueFormat,
    #[prop(optional)] mode: StackMode,
    #[prop(optional, into)] description: Option<String>,
    #[prop(default = "No data in this range.")] empty_message: &'static str,
) -> impl IntoView {
    let name = chart_name(description);
    let caption = name.caption.clone();
    let body = move || {
        labels.with(|rows| {
            series.with(|parts| chart_body(&caption, rows, parts, mode, format, empty_message))
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
    use super::{bar_width_percent, share_percent, StackMode};

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < f64::EPSILON
    }

    #[test]
    fn shares_fill_the_whole_track() {
        assert!(close(
            bar_width_percent(StackMode::Share, 10.0, 40.0),
            100.0
        ));
    }

    #[test]
    fn totals_scale_against_the_longest_row() {
        assert!(close(bar_width_percent(StackMode::Total, 20.0, 40.0), 41.0));
        assert!(close(bar_width_percent(StackMode::Total, 0.0, 0.0), 0.0));
    }

    #[test]
    fn an_empty_row_has_no_share() {
        assert!(close(share_percent(5.0, 0.0), 0.0));
        assert!(close(share_percent(1.0, 4.0), 25.0));
    }
}
