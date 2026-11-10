use leptos::prelude::*;

use super::frame::chart_name;
use super::line::{ChartHeight, LineChart};
use super::scale::ValueFormat;
use super::{AxisLabel, ChartSeries};

fn shared_maximum(series: &[ChartSeries]) -> f64 {
    series
        .iter()
        .flat_map(|panel| panel.values.iter().copied())
        .fold(0.0, f64::max)
}

fn panel(
    labels: Signal<Vec<AxisLabel>>,
    line: ChartSeries,
    ceiling: f64,
    format: ValueFormat,
) -> AnyView {
    let title = line.label.clone();
    view! {
        <section class="min-w-0 space-y-4">
            <h4 class="text-sm font-medium text-gray-900">{title.clone()}</h4>
            <LineChart
                labels=labels
                series=vec![line]
                format=format
                description=title
                value_ceiling=ceiling
                height=ChartHeight::Compact
            />
        </section>
    }
    .into_any()
}

#[component]
pub fn SmallMultiples(
    #[prop(into)] labels: Signal<Vec<AxisLabel>>,
    #[prop(into)] series: Signal<Vec<ChartSeries>>,
    format: ValueFormat,
    #[prop(optional, into)] description: Option<String>,
) -> impl IntoView {
    let name = chart_name(description);
    let panels = move || {
        let lines = series.get();
        let ceiling = shared_maximum(&lines);
        lines
            .into_iter()
            .map(|line| panel(labels, line, ceiling, format))
            .collect_view()
    };
    view! {
        <div
            role="group"
            aria-label=name.label
            aria-labelledby=name.labelled_by
            class="grid grid-cols-1 gap-x-8 gap-y-6 sm:grid-cols-2 xl:grid-cols-3"
        >
            {panels}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::shared_maximum;
    use crate::chart::{ChartColor, ChartSeries};

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < f64::EPSILON
    }

    #[test]
    fn every_panel_shares_the_highest_value() {
        let series = [4.0, 9.0].map(|peak| ChartSeries {
            label: String::new(),
            color: ChartColor::Blue,
            values: vec![1.0, peak],
        });
        assert!(close(shared_maximum(&series), 9.0));
    }
}
