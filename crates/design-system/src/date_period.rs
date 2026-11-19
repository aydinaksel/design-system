use chrono::{Datelike, NaiveDate};
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};
use leptos_router::params::ParamsMap;
use leptos_router::NavigateOptions;

use crate::date_picker::DatePicker;
use crate::listbox::{Listbox, ListboxOption};

pub const PERIOD_PARAMETER: &str = "period";
pub const FIRST_DATE_PARAMETER: &str = "from";
pub const LAST_DATE_PARAMETER: &str = "to";
const DATE_FORMAT: &str = "%Y-%m-%d";
const CUSTOM_KEY: &str = "custom";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePeriod {
    ThisMonth,
    LastMonth,
    #[default]
    ThisQuarter,
    LastQuarter,
    YearToDate,
    LastYear,
}

impl DatePeriod {
    pub const ALL: [Self; 6] = [
        Self::ThisMonth,
        Self::LastMonth,
        Self::ThisQuarter,
        Self::LastQuarter,
        Self::YearToDate,
        Self::LastYear,
    ];

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::ThisMonth => "this-month",
            Self::LastMonth => "last-month",
            Self::ThisQuarter => "this-quarter",
            Self::LastQuarter => "last-quarter",
            Self::YearToDate => "year-to-date",
            Self::LastYear => "last-year",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ThisMonth => "This Month",
            Self::LastMonth => "Last Month",
            Self::ThisQuarter => "This Quarter",
            Self::LastQuarter => "Last Quarter",
            Self::YearToDate => "Year to Date",
            Self::LastYear => "Last Year",
        }
    }

    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|period| period.key() == key)
    }

    #[must_use]
    pub fn span(self, today: NaiveDate) -> Option<DateSpan> {
        match self {
            Self::ThisMonth => month_containing(today),
            Self::LastMonth => month_containing(start_of_month(today)?.pred_opt()?),
            Self::ThisQuarter => quarter_containing(today),
            Self::LastQuarter => quarter_containing(quarter_containing(today)?.first.pred_opt()?),
            Self::YearToDate => Some(DateSpan {
                first: NaiveDate::from_ymd_opt(today.year(), 1, 1)?,
                last: today,
            }),
            Self::LastYear => {
                let year = today.year().checked_sub(1)?;
                Some(DateSpan {
                    first: NaiveDate::from_ymd_opt(year, 1, 1)?,
                    last: NaiveDate::from_ymd_opt(year, 12, 31)?,
                })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateSpan {
    pub first: NaiveDate,
    pub last: NaiveDate,
}

impl DateSpan {
    #[must_use]
    pub fn parse(first: &str, last: &str) -> Option<Self> {
        let first = NaiveDate::parse_from_str(first, DATE_FORMAT).ok()?;
        let last = NaiveDate::parse_from_str(last, DATE_FORMAT).ok()?;
        (first <= last).then_some(Self { first, last })
    }

    #[must_use]
    pub fn first_text(self) -> String {
        self.first.format(DATE_FORMAT).to_string()
    }

    #[must_use]
    pub fn last_text(self) -> String {
        self.last.format(DATE_FORMAT).to_string()
    }

    #[must_use]
    pub fn through(self, today: NaiveDate) -> Self {
        Self {
            first: self.first,
            last: self.last.min(today).max(self.first),
        }
    }

    #[must_use]
    pub fn a_year_earlier(self) -> Option<Self> {
        let year = chrono::Months::new(12);
        Some(Self {
            first: self.first.checked_sub_months(year)?,
            last: self.last.checked_sub_months(year)?,
        })
    }

    #[must_use]
    pub fn describe(self) -> String {
        let last = self.last.format("%B %-d, %Y");
        if self.first.year() == self.last.year() {
            format!("{} to {last}", self.first.format("%B %-d"))
        } else {
            format!("{} to {last}", self.first.format("%B %-d, %Y"))
        }
    }
}

fn start_of_month(day: NaiveDate) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(day.year(), day.month(), 1)
}

fn month_containing(day: NaiveDate) -> Option<DateSpan> {
    let first = start_of_month(day)?;
    let following = first.checked_add_months(chrono::Months::new(1))?;
    Some(DateSpan {
        first,
        last: following.pred_opt()?,
    })
}

fn quarter_containing(day: NaiveDate) -> Option<DateSpan> {
    let first_month = match day.month() {
        1..=3 => 1,
        4..=6 => 4,
        7..=9 => 7,
        _ => 10,
    };
    let first = NaiveDate::from_ymd_opt(day.year(), first_month, 1)?;
    let following = first.checked_add_months(chrono::Months::new(3))?;
    Some(DateSpan {
        first,
        last: following.pred_opt()?,
    })
}

#[must_use]
pub fn resolve_date_span(
    period: Option<&str>,
    first: Option<&str>,
    last: Option<&str>,
    today: NaiveDate,
) -> Option<DateSpan> {
    let custom = first
        .zip(last)
        .and_then(|(first, last)| DateSpan::parse(first, last));
    custom.or_else(|| {
        period
            .and_then(DatePeriod::from_key)
            .unwrap_or_default()
            .span(today)
    })
}

fn selected_key(parameters: &ParamsMap) -> String {
    let has_custom_dates = parameters.get(FIRST_DATE_PARAMETER).is_some()
        && parameters.get(LAST_DATE_PARAMETER).is_some();
    if has_custom_dates {
        return CUSTOM_KEY.to_owned();
    }
    parameters
        .get(PERIOD_PARAMETER)
        .and_then(|key| DatePeriod::from_key(&key))
        .unwrap_or_default()
        .key()
        .to_owned()
}

fn period_options() -> Vec<ListboxOption> {
    DatePeriod::ALL
        .into_iter()
        .map(|period| ListboxOption::new(period.key(), period.label()))
        .chain(std::iter::once(ListboxOption::new(
            CUSTOM_KEY,
            "Custom Range",
        )))
        .collect()
}

fn with_period(mut parameters: ParamsMap, key: &str, current: Option<DateSpan>) -> ParamsMap {
    parameters.remove(PERIOD_PARAMETER);
    parameters.remove(FIRST_DATE_PARAMETER);
    parameters.remove(LAST_DATE_PARAMETER);
    if key == CUSTOM_KEY {
        if let Some(span) = current {
            parameters.insert(FIRST_DATE_PARAMETER, span.first_text());
            parameters.insert(LAST_DATE_PARAMETER, span.last_text());
        }
    } else if key != DatePeriod::default().key() {
        parameters.insert(PERIOD_PARAMETER, key.to_owned());
    }
    parameters
}

fn with_custom_dates(mut parameters: ParamsMap, span: DateSpan) -> ParamsMap {
    parameters.remove(PERIOD_PARAMETER);
    parameters.replace(FIRST_DATE_PARAMETER, span.first_text());
    parameters.replace(LAST_DATE_PARAMETER, span.last_text());
    parameters
}

fn moved_first_date(current: DateSpan, chosen: NaiveDate) -> DateSpan {
    DateSpan {
        first: chosen,
        last: current.last.max(chosen),
    }
}

fn moved_last_date(current: DateSpan, chosen: NaiveDate) -> DateSpan {
    DateSpan {
        first: current.first.min(chosen),
        last: chosen,
    }
}

fn date_picker(shown: String, on_choose: impl Fn(String) + 'static) -> AnyView {
    let value = RwSignal::new(shown.clone());
    Effect::new(move |_| {
        let chosen = value.get();
        if chosen != shown {
            on_choose(chosen);
        }
    });
    view! { <DatePicker value=value /> }.into_any()
}

/// Picks the dates a report covers and keeps the choice in the URL, as
/// `period` for a named period or `from` and `to` for a custom one. The page
/// reads them back with [`resolve_date_span`] and hands the result in as
/// `current`.
#[component]
pub fn PeriodFilter(#[prop(into)] current: Signal<Option<DateSpan>>) -> impl IntoView {
    let query = use_query_map();
    let location = use_location();
    let navigate = use_navigate();
    let go_to = move |parameters: ParamsMap| {
        let target = format!(
            "{}{}",
            location.pathname.get_untracked(),
            parameters.to_query_string()
        );
        navigate(
            &target,
            NavigateOptions {
                scroll: false,
                ..NavigateOptions::default()
            },
        );
    };

    let selected = RwSignal::new(query.with_untracked(selected_key));
    Effect::new(move |_| {
        let from_url = query.with(selected_key);
        if selected.get_untracked() != from_url {
            selected.set(from_url);
        }
    });
    let choose_period = go_to.clone();
    Effect::new(move |_| {
        let key = selected.get();
        if query.with_untracked(selected_key) != key {
            choose_period(with_period(
                query.get_untracked(),
                &key,
                current.get_untracked(),
            ));
        }
    });

    let move_date = move |chosen: String, reshape: fn(DateSpan, NaiveDate) -> DateSpan| {
        let Ok(chosen) = NaiveDate::parse_from_str(&chosen, DATE_FORMAT) else {
            return;
        };
        if let Some(span) = current.get_untracked() {
            go_to(with_custom_dates(
                query.get_untracked(),
                reshape(span, chosen),
            ));
        }
    };
    let move_first = move_date.clone();
    let custom_dates = move || {
        let span = current.get().filter(|_| selected.get() == CUSTOM_KEY)?;
        let move_first = move_first.clone();
        let move_last = move_date.clone();
        Some(view! {
            {date_picker(span.first_text(), move |chosen| move_first(chosen, moved_first_date))}
            <span class="text-sm text-gray-500">"to"</span>
            {date_picker(span.last_text(), move |chosen| move_last(chosen, moved_last_date))}
        })
    };
    let caption = move || {
        current
            .get()
            .filter(|_| selected.get() != CUSTOM_KEY)
            .map(|span| view! { <span class="text-sm text-gray-500">{span.describe()}</span> })
    };

    view! {
        <div class="flex flex-wrap items-center gap-3">
            <div class="w-48">
                <Listbox options=period_options() selected=selected />
            </div>
            {custom_dates}
            {caption}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{resolve_date_span, DatePeriod, DateSpan};

    fn day(year: i32, month: u32, day_of_month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day_of_month).unwrap()
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "compared against DatePeriod::span, which returns an Option"
    )]
    fn span(first: NaiveDate, last: NaiveDate) -> Option<DateSpan> {
        Some(DateSpan { first, last })
    }

    #[test]
    fn quarters_follow_the_calendar_year() {
        let today = day(2026, 10, 27);
        assert_eq!(
            DatePeriod::ThisQuarter.span(today),
            span(day(2026, 10, 1), day(2026, 12, 31))
        );
        assert_eq!(
            DatePeriod::LastQuarter.span(today),
            span(day(2026, 7, 1), day(2026, 9, 30))
        );
    }

    #[test]
    fn last_quarter_in_january_reaches_back_a_year() {
        assert_eq!(
            DatePeriod::LastQuarter.span(day(2026, 1, 15)),
            span(day(2025, 10, 1), day(2025, 12, 31))
        );
    }

    #[test]
    fn last_month_knows_february_lengths() {
        assert_eq!(
            DatePeriod::LastMonth.span(day(2028, 3, 10)),
            span(day(2028, 2, 1), day(2028, 2, 29))
        );
    }

    #[test]
    fn custom_dates_win_over_a_named_period() {
        assert_eq!(
            resolve_date_span(
                Some("last-year"),
                Some("2026-02-01"),
                Some("2026-02-14"),
                day(2026, 10, 27)
            ),
            span(day(2026, 2, 1), day(2026, 2, 14))
        );
    }

    #[test]
    fn a_reversed_custom_range_falls_back_to_this_quarter() {
        assert_eq!(
            resolve_date_span(
                None,
                Some("2026-03-01"),
                Some("2026-02-01"),
                day(2026, 10, 27)
            ),
            span(day(2026, 10, 1), day(2026, 12, 31))
        );
    }

    #[test]
    fn the_comparison_stops_where_this_period_has_reached() {
        let quarter = DateSpan {
            first: day(2026, 10, 1),
            last: day(2026, 12, 31),
        };
        assert_eq!(
            quarter.through(day(2026, 10, 28)).a_year_earlier(),
            span(day(2025, 10, 1), day(2025, 10, 28))
        );
    }

    #[test]
    fn a_leap_day_compares_with_the_end_of_february() {
        let leap_day = DateSpan {
            first: day(2028, 2, 29),
            last: day(2028, 2, 29),
        };
        assert_eq!(
            leap_day.a_year_earlier(),
            span(day(2027, 2, 28), day(2027, 2, 28))
        );
    }

    #[test]
    fn a_range_within_one_year_names_the_year_once() {
        let quarter = DateSpan {
            first: day(2026, 7, 1),
            last: day(2026, 9, 30),
        };
        assert_eq!(quarter.describe(), "July 1 to September 30, 2026");
    }
}
