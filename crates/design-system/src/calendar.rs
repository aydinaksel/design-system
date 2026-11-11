use chrono::{Datelike, Days, Local, Months, NaiveDate};
use leptos::prelude::*;

use crate::typography::LABEL_CLASS;

const DAYS_PER_WEEK: u32 = 7;
const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GridDay {
    InMonth(NaiveDate),
    Neighbouring(NaiveDate),
}

impl GridDay {
    pub(crate) const fn date(self) -> NaiveDate {
        match self {
            Self::InMonth(date) | Self::Neighbouring(date) => date,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Month {
    pub(crate) first_day: NaiveDate,
}

impl Month {
    pub(crate) fn containing(date: NaiveDate) -> Self {
        Self {
            first_day: date.with_day(1).unwrap_or(date),
        }
    }

    pub(crate) fn current() -> Self {
        Self::containing(Local::now().date_naive())
    }

    pub(crate) fn last_day(self) -> NaiveDate {
        self.first_day
            .checked_add_months(Months::new(1))
            .and_then(|next_month| next_month.pred_opt())
            .unwrap_or(self.first_day)
    }

    pub(crate) fn previous(self) -> Self {
        self.first_day
            .checked_sub_months(Months::new(1))
            .map_or(self, Self::containing)
    }

    pub(crate) fn next(self) -> Self {
        self.first_day
            .checked_add_months(Months::new(1))
            .map_or(self, Self::containing)
    }

    pub(crate) fn label(self) -> String {
        self.first_day.format("%B %Y").to_string()
    }

    pub(crate) fn days(self) -> impl Iterator<Item = NaiveDate> {
        let month_number = self.first_day.month();
        self.first_day
            .iter_days()
            .take_while(move |day| day.month() == month_number)
    }

    pub(crate) fn leading_blank_days(self) -> u32 {
        self.first_day.weekday().num_days_from_monday()
    }

    pub(crate) fn trailing_blank_days(self) -> u32 {
        let filled_days = self
            .leading_blank_days()
            .saturating_add(self.last_day().day());
        DAYS_PER_WEEK
            .saturating_sub(filled_days.checked_rem(DAYS_PER_WEEK).unwrap_or_default())
            .checked_rem(DAYS_PER_WEEK)
            .unwrap_or_default()
    }
}

impl Month {
    pub(crate) fn grid_days(self) -> Vec<GridDay> {
        let leading_count = u64::from(self.leading_blank_days());
        let leading_start = self
            .first_day
            .checked_sub_days(Days::new(leading_count))
            .unwrap_or(self.first_day);
        let leading = leading_start
            .iter_days()
            .take_while(|day| *day < self.first_day)
            .map(GridDay::Neighbouring);
        let trailing_count = usize::try_from(self.trailing_blank_days()).unwrap_or_default();
        let trailing = self
            .last_day()
            .succ_opt()
            .into_iter()
            .flat_map(|next_day| next_day.iter_days())
            .take(trailing_count)
            .map(GridDay::Neighbouring);
        leading
            .chain(self.days().map(GridDay::InMonth))
            .chain(trailing)
            .collect()
    }
}

pub(crate) fn month_cells(
    month: Month,
    blank: impl Fn() -> AnyView,
    day: impl Fn(NaiveDate) -> AnyView,
) -> AnyView {
    month
        .grid_days()
        .into_iter()
        .map(|grid_day| match grid_day {
            GridDay::InMonth(date) => day(date),
            GridDay::Neighbouring(_) => blank(),
        })
        .collect_view()
        .into_any()
}

pub(crate) fn weekday_labels(label_class: &str) -> AnyView {
    WEEKDAY_LABELS
        .iter()
        .map(|label| view! { <div class=label_class.to_owned()>{*label}</div> })
        .collect_view()
        .into_any()
}

pub(crate) fn weekday_header() -> AnyView {
    view! {
        <div class="grid grid-cols-7 border-b border-gray-100">
            {weekday_labels(&format!("py-2 text-center {LABEL_CLASS}"))}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{GridDay, Month};

    fn date(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn a_leap_february_ends_on_the_twenty_ninth() {
        assert_eq!(
            Month::containing(date("2028-02-10")).last_day(),
            date("2028-02-29")
        );
    }

    #[test]
    fn the_grid_pads_to_whole_weeks() {
        let november = Month::containing(date("2026-11-09"));
        assert_eq!(november.leading_blank_days(), 6);
        assert_eq!(november.trailing_blank_days(), 6);
        let february = Month::containing(date("2027-02-01"));
        assert_eq!(february.leading_blank_days(), 0);
        assert_eq!(february.trailing_blank_days(), 0);
    }

    #[test]
    fn stepping_back_from_january_reaches_the_previous_december() {
        let january = Month::containing(date("2027-01-31"));
        assert_eq!(january.previous().first_day, date("2026-12-01"));
        assert_eq!(january.next().first_day, date("2027-02-01"));
    }

    #[test]
    fn the_grid_borrows_whole_weeks_from_neighbouring_months() {
        let days = Month::containing(date("2026-11-09")).grid_days();
        assert_eq!(days.len(), 42);
        assert_eq!(
            days.first(),
            Some(&GridDay::Neighbouring(date("2026-10-26")))
        );
        assert_eq!(days.get(6), Some(&GridDay::InMonth(date("2026-11-01"))));
        assert_eq!(
            days.last(),
            Some(&GridDay::Neighbouring(date("2026-12-06")))
        );
    }
}
