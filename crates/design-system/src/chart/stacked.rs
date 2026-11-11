use leptos::prelude::*;

use super::data_table::series_table;
use super::frame::chart_name;
use super::hover_card::{HoverCard, HoverSide};
use super::scale::ValueFormat;
use super::{legend, line_key, AxisLabel, ChartSeries, KeyedValue};

const LONGEST_BAR_PERCENT: f64 = 82.0;
const ROW_CLASS: &str = "group/stack relative grid grid-cols-[minmax(0,11rem)_1fr] items-center gap-3 rounded-sm py-1 outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus";
const SEGMENT_CLASS: &str = "h-full min-w-px first:rounded-l-[4px] last:rounded-r-[4px] transition-opacity group-hover/stack:opacity-80 group-focus-visible/stack:opacity-80";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackMode {
    #[default]
    Total,
    Share,
}

fn row_values(series: &[ChartSeries], index: usize) -> Vec<f64> {
    series
        .iter()
        .map(|part| part.value_at(index).unwrap_or_default().max(0.0))
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

fn breakdown(heading: &str, parts: &[KeyedValue], format: ValueFormat) -> AnyView {
    let total: f64 = parts.iter().map(|part| part.value).sum();
    let rows = parts
        .iter()
        .map(|part| {
            view! {
                <div class="flex items-center gap-2">
                    {line_key(part.color)}
                    <span class="flex-1 text-gray-500">{part.label.clone()}</span>
                    <span class="font-semibold tabular-nums text-gray-900">
                        {format.full(part.value)}
                    </span>
                    <span class="w-10 text-right tabular-nums text-gray-500">
                        {format!("{:.0}%", share_percent(part.value, total))}
                    </span>
                </div>
            }
        })
        .collect_view();
    view! {
        <div class="font-medium text-gray-900">{heading.to_owned()}</div>
        {rows}
        <div class="flex justify-between border-t border-gray-200 pt-1.5 font-medium text-gray-900">
            <span>"Total"</span>
            <span class="tabular-nums">{format.full(total)}</span>
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
    let parts: Vec<KeyedValue> = series
        .iter()
        .zip(row_values(series, index))
        .map(|(part, value)| KeyedValue {
            color: part.color,
            label: part.label.clone(),
            value,
        })
        .collect();
    let total = row_total(series, index);
    let segments = parts
        .iter()
        .map(|part| {
            view! {
                <span
                    class=format!("{SEGMENT_CLASS} {}", part.color.background_class())
                    style=format!("flex: {} 1 0", part.value)
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
    let hover = HoverCard::new();
    let heading = label.long.clone();
    let card = hover.card(HoverSide::Above, move || {
        breakdown(&heading, &parts, format)
    });
    view! {
        <div
            class=ROW_CLASS
            tabindex="0"
            on:pointerenter=move |_| hover.show()
            on:pointerleave=move |_| hover.hide()
            on:focus=move |_| hover.show()
            on:blur=move |_| hover.hide()
        >
            <span class="truncate text-sm text-gray-700" title=label.long.clone()>
                {label.short.clone()}
            </span>
            <div node_ref=hover.anchor class="flex h-8 items-center">
                <span
                    class="flex h-full gap-0.5"
                    style=format!("width: {:.3}%", bar_width_percent(mode, total, longest_total))
                >
                    {segments}
                </span>
                {total_label}
            </div>
            {card}
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
