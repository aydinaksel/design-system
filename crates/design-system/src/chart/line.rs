use leptos::ev::{KeyboardEvent, PointerEvent};
use leptos::html::Div;
use leptos::prelude::*;

use super::curve::{monotone_path, PlotPoint};
use super::data_table::series_table;
use super::frame::chart_name;
use super::label_layout::spread_label_positions;
use super::reference::{covered_maximum, reference_overlay, ReferenceLine};
use super::scale::{ValueFormat, ValueScale};
use super::{line_key, AxisLabel, ChartSeries};

const MAXIMUM_AXIS_LABELS: usize = 6;
const TOOLTIP_GAP: &str = "12px";
const END_LABEL_SPACING_PERCENT: f64 = 8.0;
const LABELLED_AXIS_CLASS: &str = "relative mt-2 mr-31 ml-15 h-4 text-xs text-gray-500";
const UNLABELLED_AXIS_CLASS: &str = "relative mt-2 ml-15 h-4 text-xs text-gray-500";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartHeight {
    #[default]
    Standard,
    Compact,
}

impl ChartHeight {
    const fn plot_row_class(self) -> &'static str {
        match self {
            Self::Standard => "flex h-64 gap-3",
            Self::Compact => "flex h-32 gap-3",
        }
    }
}

fn position_of(index: usize) -> f64 {
    f64::from(u32::try_from(index).unwrap_or(u32::MAX))
}

fn horizontal_percent(index: usize, point_count: usize) -> f64 {
    let last_index = point_count.saturating_sub(1);
    if last_index == 0 {
        50.0
    } else {
        position_of(index) / position_of(last_index) * 100.0
    }
}

fn nearest_index(fraction: f64, point_count: usize) -> Option<usize> {
    let target_percent = fraction.clamp(0.0, 1.0) * 100.0;
    (0..point_count).min_by(|left, right| {
        let left_distance = (horizontal_percent(*left, point_count) - target_percent).abs();
        let right_distance = (horizontal_percent(*right, point_count) - target_percent).abs();
        left_distance.total_cmp(&right_distance)
    })
}

fn axis_label_indices(point_count: usize) -> Vec<usize> {
    let intervals = MAXIMUM_AXIS_LABELS.saturating_sub(1);
    let step = point_count.saturating_sub(1).div_ceil(intervals).max(1);
    let mut indices: Vec<usize> = (0..point_count).step_by(step).collect();
    let last_index = point_count.saturating_sub(1);
    let last_chosen = indices.last().copied().unwrap_or(last_index);
    if last_index.saturating_sub(last_chosen) >= step / 2 {
        if last_chosen != last_index {
            indices.push(last_index);
        }
    } else if let Some(final_label) = indices.last_mut() {
        *final_label = last_index;
    }
    indices
}

fn highest_value(series: &[ChartSeries]) -> f64 {
    series
        .iter()
        .flat_map(|line| line.values.iter().copied())
        .fold(0.0, f64::max)
}

fn line_path(values: &[f64], scale: &ValueScale) -> String {
    let point_count = values.len();
    let points: Vec<PlotPoint> = values
        .iter()
        .enumerate()
        .map(|(index, value)| PlotPoint {
            horizontal: horizontal_percent(index, point_count),
            vertical: scale.percent_from_top(*value),
        })
        .collect();
    monotone_path(&points)
}

fn axis_label_alignment(index: usize, point_count: usize) -> &'static str {
    if index == 0 {
        "translateX(0)"
    } else if index == point_count.saturating_sub(1) {
        "translateX(-100%)"
    } else {
        "translateX(-50%)"
    }
}

fn plot_layer(series: &[ChartSeries], scale: &ValueScale) -> AnyView {
    let gridlines = scale
        .ticks
        .iter()
        .enumerate()
        .map(|(position, tick)| {
            let height = scale.percent_from_top(*tick);
            let tone = if position == 0 {
                "stroke-gray-300"
            } else {
                "stroke-gray-200"
            };
            view! {
                <line
                    x1="0"
                    x2="100"
                    y1=height
                    y2=height
                    class=tone
                    stroke-width="1"
                    vector-effect="non-scaling-stroke"
                />
            }
        })
        .collect_view();
    let mut drawing_order: Vec<&ChartSeries> = series.iter().collect();
    drawing_order.sort_by_key(|line| !line.color.is_muted());
    let lines = drawing_order
        .into_iter()
        .map(|line| {
            view! {
                <path
                    d=line_path(&line.values, scale)
                    fill="none"
                    class=line.color.stroke_class()
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    vector-effect="non-scaling-stroke"
                />
            }
        })
        .collect_view();
    view! {
        <svg
            aria-hidden="true"
            class="absolute inset-0 h-full w-full overflow-visible"
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
        >
            {gridlines}
            {lines}
        </svg>
    }
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

fn category_axis(labels: &[AxisLabel]) -> AnyView {
    let point_count = labels.len();
    axis_label_indices(point_count)
        .into_iter()
        .filter_map(|index| labels.get(index).map(|label| (index, label.short.clone())))
        .map(|(index, text)| {
            let style = format!(
                "left: {:.3}%; transform: {}",
                horizontal_percent(index, point_count),
                axis_label_alignment(index, point_count)
            );
            view! {
                <span class="absolute top-0 whitespace-nowrap" style=style>
                    {text}
                </span>
            }
        })
        .collect_view()
        .into_any()
}

fn tooltip_rows(series: &[ChartSeries], index: usize, format: ValueFormat) -> AnyView {
    series
        .iter()
        .filter_map(|line| line.values.get(index).map(|value| (line, *value)))
        .map(|(line, value)| {
            view! {
                <div class="flex items-center gap-2">
                    {line_key(line.color)}
                    <span class="flex-1 text-gray-500">{line.label.clone()}</span>
                    <span class="font-semibold tabular-nums text-gray-900">
                        {format.full(value)}
                    </span>
                </div>
            }
        })
        .collect_view()
        .into_any()
}

fn hover_layer(
    labels: &[AxisLabel],
    series: &[ChartSeries],
    scale: &ValueScale,
    index: usize,
    format: ValueFormat,
) -> AnyView {
    let Some(label) = labels.get(index) else {
        return ().into_any();
    };
    let left_percent = horizontal_percent(index, labels.len());
    let tooltip_offset = if left_percent > 50.0 {
        format!("calc(-100% - {TOOLTIP_GAP})")
    } else {
        TOOLTIP_GAP.to_owned()
    };
    let markers = series
        .iter()
        .filter_map(|line| line.values.get(index).map(|value| (line.color, *value)))
        .map(|(color, value)| {
            let style = format!(
                "left: {left_percent:.3}%; top: {:.3}%",
                scale.percent_from_top(value)
            );
            view! {
                <span
                    class=format!(
                        "absolute size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full ring-2 ring-white {}",
                        color.background_class(),
                    )
                    style=style
                ></span>
            }
        })
        .collect_view();
    view! {
        <span
            class="absolute inset-y-0 w-px bg-gray-300"
            style=format!("left: {left_percent:.3}%")
        ></span>
        {markers}
        <div
            class="absolute top-0 z-10 min-w-44 space-y-1.5 rounded-item border border-gray-200 bg-surface px-3 py-2 text-xs shadow-lg"
            style=format!("left: {left_percent:.3}%; transform: translateX({tooltip_offset})")
        >
            <div class="font-medium text-gray-900">{label.long.clone()}</div>
            {tooltip_rows(series, index, format)}
        </div>
    }
    .into_any()
}

fn labels_each_line(series: &[ChartSeries]) -> bool {
    series.len() > 1
}

fn end_labels(series: &[ChartSeries], scale: &ValueScale) -> AnyView {
    if !labels_each_line(series) {
        return ().into_any();
    }
    let mut endings: Vec<(f64, &ChartSeries)> = series
        .iter()
        .filter(|line| !line.color.is_muted())
        .filter_map(|line| {
            line.values
                .last()
                .map(|value| (scale.percent_from_top(*value), line))
        })
        .collect();
    endings.sort_by(|left, right| left.0.total_cmp(&right.0));
    let desired: Vec<f64> = endings
        .iter()
        .map(|(top_percent, _)| *top_percent)
        .collect();
    let labels = endings
        .iter()
        .zip(spread_label_positions(&desired, END_LABEL_SPACING_PERCENT))
        .map(|((_, line), top_percent)| {
            view! {
                <span
                    class="absolute inset-x-0 flex -translate-y-1/2 items-center gap-1.5"
                    style=format!("top: {top_percent:.3}%")
                >
                    {line_key(line.color)}
                    <span class="truncate" title=line.label.clone()>
                        {line.label.clone()}
                    </span>
                </span>
            }
        })
        .collect_view();
    view! {
        <div aria-hidden="true" class="relative w-28 shrink-0 text-xs font-medium text-gray-700">
            {labels}
        </div>
    }
    .into_any()
}

fn stepped_index(key: &str, current: Option<usize>, point_count: usize) -> Option<usize> {
    let last_index = point_count.checked_sub(1)?;
    let from = current.unwrap_or(last_index);
    match key {
        "ArrowLeft" => Some(from.saturating_sub(1)),
        "ArrowRight" => Some(from.saturating_add(1).min(last_index)),
        "Home" => Some(0),
        "End" => Some(last_index),
        _ => None,
    }
}

fn value_scale(
    series: Signal<Vec<ChartSeries>>,
    value_ceiling: Option<f64>,
    reference: Option<ReferenceLine>,
) -> Memo<ValueScale> {
    Memo::new(move |_| {
        let data_maximum = series.with(|lines| highest_value(lines));
        let ceiling =
            value_ceiling.map_or(data_maximum, |minimum_top| data_maximum.max(minimum_top));
        ValueScale::covering(covered_maximum(ceiling, reference.as_ref()))
    })
}

fn category_axis_class(series: Signal<Vec<ChartSeries>>) -> &'static str {
    if series.with(|lines| labels_each_line(lines)) {
        LABELLED_AXIS_CLASS
    } else {
        UNLABELLED_AXIS_CLASS
    }
}

#[component]
pub fn LineChart(
    #[prop(into)] labels: Signal<Vec<AxisLabel>>,
    #[prop(into)] series: Signal<Vec<ChartSeries>>,
    format: ValueFormat,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] reference: Option<ReferenceLine>,
    #[prop(optional)] value_ceiling: Option<f64>,
    #[prop(optional)] height: ChartHeight,
    #[prop(default = "No data in this range.")] empty_message: &'static str,
) -> impl IntoView {
    let active_index: RwSignal<Option<usize>> = RwSignal::new(None);
    let plot_area: NodeRef<Div> = NodeRef::new();
    let scale = value_scale(series, value_ceiling, reference.clone());
    let point_count = move || labels.with(Vec::len);
    let name = chart_name(description);
    let caption = name.caption.clone();
    let axis_class = move || category_axis_class(series);

    let on_pointer_move = move |event: PointerEvent| {
        let Some(element) = plot_area.get_untracked() else {
            return;
        };
        let bounds = element.get_bounding_client_rect();
        if bounds.width() <= 0.0 {
            return;
        }
        let fraction = (f64::from(event.client_x()) - bounds.left()) / bounds.width();
        active_index.set(nearest_index(fraction, labels.with_untracked(Vec::len)));
    };
    let on_key_down = move |event: KeyboardEvent| {
        let stepped = stepped_index(
            &event.key(),
            active_index.get_untracked(),
            labels.with_untracked(Vec::len),
        );
        if let Some(index) = stepped {
            event.prevent_default();
            active_index.set(Some(index));
        }
    };
    let on_focus = move |_| {
        if active_index.get_untracked().is_none() {
            active_index.set(labels.with_untracked(Vec::len).checked_sub(1));
        }
    };

    view! {
        <figure>
            <div class=height.plot_row_class()>
                <div class="relative w-12 shrink-0 text-right text-xs tabular-nums text-gray-500">
                    {move || value_axis(&scale.get(), format)}
                </div>
                <div class="relative flex-1" node_ref=plot_area>
                    {move || series.with(|lines| plot_layer(lines, &scale.get()))}
                    {move || reference_overlay(reference.as_ref(), &scale.get(), format)}
                    {move || {
                        (point_count() == 0)
                            .then(|| {
                                view! {
                                    <p class="absolute inset-0 flex items-center justify-center text-sm text-gray-500">
                                        {empty_message}
                                    </p>
                                }
                            })
                    }}
                    <div aria-live="polite" class="pointer-events-none absolute inset-0">
                        {move || {
                            active_index
                                .get()
                                .map(|index| {
                                    labels
                                        .with(|axis| {
                                            series
                                                .with(|lines| {
                                                    hover_layer(axis, lines, &scale.get(), index, format)
                                                })
                                        })
                                })
                        }}
                    </div>
                    <div
                        class="absolute inset-0 cursor-crosshair rounded-sm focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-focus"
                        tabindex="0"
                        role="group"
                        aria-label=name.label
                        aria-labelledby=name.labelled_by
                        on:pointermove=on_pointer_move
                        on:pointerleave=move |_| active_index.set(None)
                        on:keydown=on_key_down
                        on:focus=on_focus
                        on:blur=move |_| active_index.set(None)
                    ></div>
                </div>
                {move || series.with(|lines| end_labels(lines, &scale.get()))}
            </div>
            <div class=axis_class>{move || labels.with(|axis| category_axis(axis))}</div>
            {move || {
                labels
                    .with(|axis| {
                        series.with(|lines| series_table(&caption, axis, lines, format))
                    })
            }}
        </figure>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::{axis_label_indices, nearest_index, stepped_index};

    #[test]
    fn a_quarter_of_days_gets_six_evenly_spaced_labels() {
        assert_eq!(axis_label_indices(92), vec![0, 19, 38, 57, 76, 91]);
    }

    #[test]
    fn the_pointer_snaps_to_the_closest_point() {
        assert_eq!(nearest_index(0.49, 3), Some(1));
        assert_eq!(nearest_index(1.4, 3), Some(2));
        assert_eq!(nearest_index(0.5, 0), None);
    }

    #[test]
    fn arrow_keys_stay_inside_the_series() {
        assert_eq!(stepped_index("ArrowRight", Some(2), 3), Some(2));
        assert_eq!(stepped_index("ArrowLeft", Some(0), 3), Some(0));
        assert_eq!(stepped_index("ArrowLeft", None, 0), None);
    }
}
