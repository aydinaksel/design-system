use leptos::prelude::*;

use super::scale::ValueFormat;
use super::{AxisLabel, ChartSeries};

fn value_row(
    index: usize,
    label: &AxisLabel,
    series: &[ChartSeries],
    format: ValueFormat,
) -> AnyView {
    let cells = series
        .iter()
        .map(|line| view! { <td>{line.values.get(index).map(|value| format.full(*value))}</td> })
        .collect_view();
    view! {
        <tr>
            <th scope="row">{label.long.clone()}</th>
            {cells}
        </tr>
    }
    .into_any()
}

pub(crate) fn series_table(
    caption: &str,
    labels: &[AxisLabel],
    series: &[ChartSeries],
    format: ValueFormat,
) -> AnyView {
    if labels.is_empty() {
        return ().into_any();
    }
    let headings = series
        .iter()
        .map(|line| view! { <th scope="col">{line.label.clone()}</th> })
        .collect_view();
    let rows = labels
        .iter()
        .enumerate()
        .map(|(index, label)| value_row(index, label, series, format))
        .collect_view();
    view! {
        <div class="sr-only">
            <table>
                <caption>{caption.to_owned()}</caption>
                <thead>
                    <tr>
                        <td></td>
                        {headings}
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
    .into_any()
}
