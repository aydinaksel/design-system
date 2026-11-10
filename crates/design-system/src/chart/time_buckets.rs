use std::ops::RangeInclusive;

use chrono::{Datelike, Days, Months, NaiveDate};

use super::AxisLabel;

const LONGEST_DAILY_RANGE_DAYS: i64 = 62;
const LONGEST_WEEKLY_RANGE_DAYS: i64 = 190;
const DAYS_PER_WEEK: u64 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeGrain {
    Day,
    Week,
    Month,
}

impl TimeGrain {
    #[must_use]
    pub fn for_range(range: &RangeInclusive<NaiveDate>) -> Self {
        let length_days = range.end().signed_duration_since(*range.start()).num_days();
        if length_days <= LONGEST_DAILY_RANGE_DAYS {
            Self::Day
        } else if length_days <= LONGEST_WEEKLY_RANGE_DAYS {
            Self::Week
        } else {
            Self::Month
        }
    }

    fn next_start(self, start: NaiveDate) -> Option<NaiveDate> {
        match self {
            Self::Day => start.succ_opt(),
            Self::Week => start.checked_add_days(Days::new(DAYS_PER_WEEK)),
            Self::Month => NaiveDate::from_ymd_opt(start.year(), start.month(), 1)?
                .checked_add_months(Months::new(1)),
        }
    }

    fn label(self, start: NaiveDate, spans_years: bool) -> AxisLabel {
        let (short, long) = match self {
            Self::Day => (
                start.format("%b %-d").to_string(),
                start.format("%B %-d, %Y").to_string(),
            ),
            Self::Week => (
                start.format("%b %-d").to_string(),
                start.format("Week of %B %-d, %Y").to_string(),
            ),
            Self::Month if spans_years => (
                start.format("%b %Y").to_string(),
                start.format("%B %Y").to_string(),
            ),
            Self::Month => (
                start.format("%b").to_string(),
                start.format("%B %Y").to_string(),
            ),
        };
        AxisLabel { short, long }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BucketedSeries {
    pub labels: Vec<AxisLabel>,
    pub sums: Vec<Vec<f64>>,
}

fn bucket_starts(range: &RangeInclusive<NaiveDate>, grain: TimeGrain) -> Vec<NaiveDate> {
    std::iter::successors(Some(*range.start()), |start| grain.next_start(*start))
        .take_while(|start| start <= range.end())
        .collect()
}

#[must_use]
pub fn bucket_by_date(
    range: &RangeInclusive<NaiveDate>,
    series_count: usize,
    points: &[(NaiveDate, Vec<f64>)],
) -> BucketedSeries {
    let grain = TimeGrain::for_range(range);
    let starts = bucket_starts(range, grain);
    let spans_years = range.start().year() != range.end().year();
    let mut sums = vec![vec![0.0; starts.len()]; series_count];
    for (date, values) in points {
        let Some(bucket) = starts.partition_point(|start| start <= date).checked_sub(1) else {
            continue;
        };
        for (series, value) in sums.iter_mut().zip(values) {
            if let Some(slot) = series.get_mut(bucket) {
                *slot += value;
            }
        }
    }
    BucketedSeries {
        labels: starts
            .iter()
            .map(|start| grain.label(*start, spans_years))
            .collect(),
        sums,
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{bucket_by_date, TimeGrain};

    fn day(year: i32, month: u32, day_of_month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day_of_month).unwrap()
    }

    #[test]
    fn longer_ranges_use_coarser_buckets() {
        assert_eq!(
            TimeGrain::for_range(&(day(2026, 10, 1)..=day(2026, 10, 31))),
            TimeGrain::Day
        );
        assert_eq!(
            TimeGrain::for_range(&(day(2026, 7, 1)..=day(2026, 9, 30))),
            TimeGrain::Week
        );
        assert_eq!(
            TimeGrain::for_range(&(day(2026, 1, 1)..=day(2026, 12, 31))),
            TimeGrain::Month
        );
    }

    #[test]
    fn days_without_documents_still_get_a_point() {
        let bucketed = bucket_by_date(
            &(day(2026, 10, 1)..=day(2026, 10, 3)),
            1,
            &[(day(2026, 10, 3), vec![50.0])],
        );
        assert_eq!(bucketed.sums, vec![vec![0.0, 0.0, 50.0]]);
        assert_eq!(bucketed.labels.len(), 3);
    }

    #[test]
    fn months_add_up_every_day_in_them() {
        let bucketed = bucket_by_date(
            &(day(2026, 1, 1)..=day(2026, 12, 31)),
            2,
            &[
                (day(2026, 2, 3), vec![10.0, 1.0]),
                (day(2026, 2, 27), vec![5.0, 2.0]),
            ],
        );
        assert_eq!(
            bucketed.sums.first().and_then(|series| series.get(1)),
            Some(&15.0)
        );
        assert_eq!(
            bucketed.sums.get(1).and_then(|series| series.get(1)),
            Some(&3.0)
        );
        assert_eq!(
            bucketed.labels.get(1).map(|label| label.long.as_str()),
            Some("February 2026")
        );
    }
}
