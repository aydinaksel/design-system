use leptos::prelude::*;

use super::frame::{chart_name, section_heading_level};
use super::line::{ChartHeight, LineChart};
use super::scale::ValueFormat;
use super::{AxisLabel, ChartSeries};
use crate::typography::{heading_element, HeadingLevel};

const PANEL_TITLE_CLASS: &str = "text-sm font-medium text-gray-900";

fn shared_maximum(series: &[ChartSeries]) -> f64 {
    series
        .iter()
        .flat_map(ChartSeries::present_values)
        .fold(0.0, f64::max)
}

fn panel(
    labels: Signal<Vec<AxisLabel>>,
    line: ChartSeries,
    ceiling: f64,
    title_level: HeadingLevel,
    format: ValueFormat,
) -> AnyView {
    let title = line.label.clone();
    view! {
        <section class="min-w-0 space-y-4">
            {heading_element(title_level, None, PANEL_TITLE_CLASS, title.clone())}
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
    #[prop(optional)] heading_level: Option<HeadingLevel>,
) -> impl IntoView {
    let name = chart_name(description);
    let title_level = section_heading_level(heading_level);
    let panels = move || {
        let lines = series.get();
        let ceiling = shared_maximum(&lines);
        lines
            .into_iter()
            .map(|line| panel(labels, line, ceiling, title_level, format))
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
            values: vec![Some(1.0), Some(peak)],
        });
        assert!(close(shared_maximum(&series), 9.0));
    }
}
