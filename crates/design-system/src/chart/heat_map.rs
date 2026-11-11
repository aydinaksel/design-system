use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use chrono::{Datelike, NaiveDate};
use leptos::prelude::*;

use super::data_table::series_table;
use super::frame::{chart_name, section_heading_level};
use super::scale::{ValueFormat, ValueScale};
use super::{AxisLabel, ChartColor, ChartSeries};
use crate::calendar::{month_cells, weekday_labels, Month};
use crate::typography::{heading_element, HeadingLevel};

const STEP_UPPER_FRACTIONS: [f64; 5] = [0.2, 0.4, 0.6, 0.8, 1.0];
const STEP_CLASSES: [&str; 5] = [
    "bg-heat-1 text-gray-900",
    "bg-heat-2 text-gray-900",
    "bg-heat-3 text-gray-900",
    "bg-heat-4 text-gray-900",
    "bg-heat-5 text-white",
];
const CELL_SHAPE_CLASS: &str =
    "flex aspect-square min-w-0 flex-col justify-between rounded-sm p-1 leading-none";
const DAY_NUMBER_CLASS: &str = "text-[10px] opacity-70";
const VALUE_CLASS: &str = "truncate text-[11px] font-semibold tabular-nums";
const WEEKDAY_CLASS: &str = "pb-1 text-center text-[10px] font-medium text-gray-500";
const SPOKEN_DATE_FORMAT: &str = "%A, %B %-d, %Y";
const MONTH_TITLE_CLASS: &str = "mb-2 text-sm font-medium text-gray-900";

type DailyValues = BTreeMap<NaiveDate, f64>;

fn daily_values(days: &[(NaiveDate, f64)]) -> DailyValues {
    let mut totals = DailyValues::new();
    for (day, value) in days {
        *totals.entry(*day).or_default() += value;
    }
    totals
}

fn step_index(value: f64, top: f64) -> usize {
    if top <= 0.0 {
        return 0;
    }
    let fraction = value / top;
    STEP_UPPER_FRACTIONS
        .iter()
        .position(|upper| fraction <= *upper)
        .unwrap_or(STEP_UPPER_FRACTIONS.len().saturating_sub(1))
}

fn months_covering(range: &RangeInclusive<NaiveDate>) -> Vec<Month> {
    std::iter::successors(Some(Month::containing(*range.start())), |month| {
        let next = month.next();
        (next != *month).then_some(next)
    })
    .take_while(|month| month.first_day <= *range.end())
    .collect()
}

fn blank_cell() -> AnyView {
    view! { <div></div> }.into_any()
}

fn day_cell(
    day: NaiveDate,
    value: Option<f64>,
    in_range: bool,
    top: f64,
    format: ValueFormat,
) -> AnyView {
    let number = view! { <span class=DAY_NUMBER_CLASS>{day.day()}</span> };
    let Some(amount) = value.filter(|_| in_range) else {
        let tone = if in_range {
            "bg-gray-50 text-gray-400"
        } else {
            "text-gray-300"
        };
        return view! { <div class=format!("{CELL_SHAPE_CLASS} {tone}")>{number}</div> }.into_any();
    };
    let step_class = STEP_CLASSES
        .get(step_index(amount, top))
        .copied()
        .unwrap_or_default();
    view! {
        <div class=format!(
            "{CELL_SHAPE_CLASS} {step_class}",
        )>{number} <span class=VALUE_CLASS>{format.compact(amount)}</span></div>
    }
    .into_any()
}

fn month_grid(
    month: Month,
    range: &RangeInclusive<NaiveDate>,
    values: &DailyValues,
    top: f64,
    title_level: HeadingLevel,
    format: ValueFormat,
) -> AnyView {
    let cells = month_cells(month, blank_cell, |day| {
        day_cell(
            day,
            values.get(&day).copied(),
            range.contains(&day),
            top,
            format,
        )
    });
    view! {
        <section class="min-w-0">
            {heading_element(title_level, None, MONTH_TITLE_CLASS, month.label())}
            <div class="grid grid-cols-7 gap-1">{weekday_labels(WEEKDAY_CLASS)} {cells}</div>
        </section>
    }
    .into_any()
}

fn scale_legend(top: f64, format: ValueFormat) -> AnyView {
    let swatches = STEP_CLASSES
        .iter()
        .map(|step_class| view! { <span class=format!("h-2.5 {step_class}")></span> })
        .collect_view();
    let boundaries = std::iter::once(0.0)
        .chain(STEP_UPPER_FRACTIONS)
        .map(|fraction| {
            view! {
                <span
                    class="absolute -translate-x-1/2 tabular-nums"
                    style=format!("left: {:.1}%", fraction * 100.0)
                >
                    {format.compact(fraction * top)}
                </span>
            }
        })
        .collect_view();
    view! {
        <div aria-hidden="true" class="mx-auto mt-6 w-64">
            <div class="grid grid-cols-5 gap-px overflow-hidden rounded-sm">{swatches}</div>
            <div class="relative mt-1 h-4 text-[10px] text-gray-500">{boundaries}</div>
        </div>
    }
    .into_any()
}

fn table_shape(
    range: &RangeInclusive<NaiveDate>,
    values: &DailyValues,
) -> (Vec<AxisLabel>, Vec<ChartSeries>) {
    let shown: Vec<(&NaiveDate, &f64)> = values.range(range.clone()).collect();
    let labels = shown
        .iter()
        .map(|(day, _)| {
            let spoken = day.format(SPOKEN_DATE_FORMAT).to_string();
            AxisLabel {
                short: spoken.clone(),
                long: spoken,
            }
        })
        .collect();
    let series = vec![ChartSeries {
        label: "Value".to_owned(),
        color: ChartColor::Blue,
        values: shown.iter().map(|(_, value)| Some(**value)).collect(),
    }];
    (labels, series)
}

#[component]
pub fn CalendarHeatMap(
    #[prop(into)] days: Signal<Vec<(NaiveDate, f64)>>,
    #[prop(into)] range: Signal<RangeInclusive<NaiveDate>>,
    format: ValueFormat,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] heading_level: Option<HeadingLevel>,
) -> impl IntoView {
    let name = chart_name(description);
    let title_level = section_heading_level(heading_level);
    let caption = name.caption.clone();
    let body = move || {
        let shown_range = range.get();
        let values = days.with(|entries| daily_values(entries));
        let highest = values
            .range(shown_range.clone())
            .map(|(_, value)| *value)
            .fold(0.0, f64::max);
        let top = ValueScale::covering(highest).top;
        let months = months_covering(&shown_range)
            .into_iter()
            .map(|month| month_grid(month, &shown_range, &values, top, title_level, format))
            .collect_view();
        let (labels, series) = table_shape(&shown_range, &values);
        view! {
            <div aria-hidden="true" class="grid gap-6 sm:grid-cols-2 xl:grid-cols-3">
                {months}
            </div>
            {scale_legend(top, format)}
            {series_table(&caption, &labels, &series, format)}
        }
        .into_any()
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
    use chrono::NaiveDate;

    use super::{daily_values, months_covering, step_index};

    fn day(year: i32, month: u32, day_of_month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day_of_month).unwrap()
    }

    #[test]
    fn values_fall_into_five_even_steps() {
        assert_eq!(step_index(0.0, 60.0), 0);
        assert_eq!(step_index(12.0, 60.0), 0);
        assert_eq!(step_index(12.5, 60.0), 1);
        assert_eq!(step_index(60.0, 60.0), 4);
        assert_eq!(step_index(5.0, 0.0), 0);
    }

    #[test]
    fn a_quarter_spans_three_months() {
        let months = months_covering(&(day(2026, 7, 1)..=day(2026, 9, 30)));
        let firsts: Vec<NaiveDate> = months.iter().map(|month| month.first_day).collect();
        assert_eq!(
            firsts,
            vec![day(2026, 7, 1), day(2026, 8, 1), day(2026, 9, 1)]
        );
    }

    #[test]
    fn repeated_days_add_up() {
        let values = daily_values(&[(day(2026, 7, 1), 5.0), (day(2026, 7, 1), 7.0)]);
        assert_eq!(values.get(&day(2026, 7, 1)).copied(), Some(12.0));
    }
}
