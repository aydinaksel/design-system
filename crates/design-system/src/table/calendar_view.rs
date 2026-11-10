use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{Datelike, Local, Months, NaiveDate};
use leptos::prelude::*;
use table_query::TableQuery;

use super::cell_value::CellValue;
use super::column::Column;
use super::data_table::{
    error_alert, refresh_cover, DataTable, RowAttribute, Search, HEADER_RULE_CLASS,
};
use super::url_query::UrlTableQuery;
use crate::dialog::{Dialog, DialogSize};
use crate::icons;
use crate::tooltip::Tooltip;
use crate::typography::LABEL_CLASS;

const MONTH_ROW_CAP: u32 = 500;
const DATE_FORMAT: &str = "%Y-%m-%d";
const DAYS_PER_WEEK: u32 = 7;
const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const CELL_SHAPE_CLASS: &str = "flex min-h-20 flex-col border-b border-r border-gray-100 p-1.5 [&:nth-child(7n)]:border-r-0 [&:nth-last-child(-n+7)]:border-b-0";
const DAY_NUMBER_CLASS: &str =
    "flex h-5 w-5 items-center justify-center text-xs leading-none text-gray-500";
const TODAY_NUMBER_CLASS: &str = "flex h-5 w-5 items-center justify-center rounded-full bg-blue-600 text-xs font-semibold leading-none text-white";
const COUNT_BADGE_CLASS: &str = "flex h-7 min-w-7 items-center justify-center rounded-full bg-blue-100 px-1 text-xs font-medium text-blue-700 tabular-nums";
const MONTH_BAR_CLASS: &str =
    "flex items-center justify-between gap-2 rounded-t-surface-inner bg-gray-50 py-2.5 pl-1.5 pr-2";
const MONTH_STEP_CLASS: &str =
    "cursor-pointer rounded p-1 text-gray-500 hover:bg-gray-200 hover:text-gray-800 focus-ring";
const TODAY_BUTTON_CLASS: &str = "cursor-pointer rounded px-2 py-1 text-xs font-medium text-gray-600 hover:bg-gray-200 hover:text-gray-900 focus-ring";

type DateValue<Row> = Arc<dyn Fn(&Row) -> CellValue + Send + Sync>;
type RowsByDay<Row> = BTreeMap<NaiveDate, Vec<Row>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Month {
    first_day: NaiveDate,
}

impl Month {
    fn containing(date: NaiveDate) -> Self {
        Self {
            first_day: date.with_day(1).unwrap_or(date),
        }
    }

    fn current() -> Self {
        Self::containing(Local::now().date_naive())
    }

    fn shown_by(query: &TableQuery, date_column_key: &str) -> Self {
        query
            .date_range
            .as_ref()
            .filter(|range| range.column_key == date_column_key)
            .and_then(|range| NaiveDate::parse_from_str(&range.from, DATE_FORMAT).ok())
            .map_or_else(Self::current, Self::containing)
    }

    fn last_day(self) -> NaiveDate {
        self.first_day
            .checked_add_months(Months::new(1))
            .and_then(|next_month| next_month.pred_opt())
            .unwrap_or(self.first_day)
    }

    fn previous(self) -> Self {
        self.first_day
            .checked_sub_months(Months::new(1))
            .map_or(self, Self::containing)
    }

    fn next(self) -> Self {
        self.first_day
            .checked_add_months(Months::new(1))
            .map_or(self, Self::containing)
    }

    fn label(self) -> String {
        self.first_day.format("%B %Y").to_string()
    }

    fn days(self) -> impl Iterator<Item = NaiveDate> {
        let month_number = self.first_day.month();
        self.first_day
            .iter_days()
            .take_while(move |day| day.month() == month_number)
    }

    fn leading_blank_days(self) -> u32 {
        self.first_day.weekday().num_days_from_monday()
    }

    fn trailing_blank_days(self) -> u32 {
        let filled_days = self
            .leading_blank_days()
            .saturating_add(self.last_day().day());
        DAYS_PER_WEEK
            .saturating_sub(filled_days.checked_rem(DAYS_PER_WEEK).unwrap_or_default())
            .checked_rem(DAYS_PER_WEEK)
            .unwrap_or_default()
    }

    fn show_in(self, query: &mut TableQuery, date_column_key: &str) {
        query.show_month(
            date_column_key,
            self.first_day.format(DATE_FORMAT).to_string(),
            self.last_day().format(DATE_FORMAT).to_string(),
        );
    }
}

/// Every row of the month the query shows, or of the current month when it
/// names none, in one unsorted page.
pub(crate) fn month_query(mut query: TableQuery, date_column_key: &str) -> TableQuery {
    Month::shown_by(&query, date_column_key).show_in(&mut query, date_column_key);
    query.sort_column_key = None;
    query.sort_ascending = true;
    query.rows_per_page = MONTH_ROW_CAP;
    query
}

fn results_label(count: usize) -> String {
    if count == 1 {
        "1 result".to_owned()
    } else {
        format!("{count} results")
    }
}

fn long_date(day: NaiveDate) -> String {
    day.format("%A, %-d %B %Y").to_string()
}

fn group_by_day<Row>(
    rows: Option<&Result<Vec<Row>, String>>,
    date_value: Option<&DateValue<Row>>,
) -> RowsByDay<Row>
where
    Row: Clone,
{
    let mut by_day: RowsByDay<Row> = BTreeMap::new();
    let (Some(Ok(rows)), Some(date_value)) = (rows, date_value) else {
        return by_day;
    };
    for row in rows {
        let Ok(day) = NaiveDate::parse_from_str(&date_value(row).filter_key(), DATE_FORMAT) else {
            continue;
        };
        by_day.entry(day).or_default().push(row.clone());
    }
    by_day
}

/// A month of rows, counted per day; a day opens its rows in a dialog.
#[component]
pub(crate) fn CalendarView<Row>(
    rows: Signal<Option<Result<Vec<Row>, String>>>,
    columns: Vec<Column<Row>>,
    date_column_key: &'static str,
    query: UrlTableQuery,
    total_rows: Signal<Option<u64>>,
    pending: Signal<bool>,
    row_href: Option<RowAttribute<Row>>,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
{
    let month = Memo::new(move |_| query.with(|query| Month::shown_by(query, date_column_key)));
    let show = move |target: Month| query.update(|query| target.show_in(query, date_column_key));

    let date_value: Option<DateValue<Row>> = columns
        .iter()
        .find(|column| column.key == date_column_key)
        .map(|column| Arc::clone(&column.value));
    let rows_by_day =
        Signal::derive(move || rows.with(|rows| group_by_day(rows.as_ref(), date_value.as_ref())));
    let selected_day: RwSignal<Option<NaiveDate>> = RwSignal::new(None);

    let summary = move || {
        let loaded = rows.with(|rows| match rows {
            Some(Ok(loaded)) => Some(loaded.len()),
            _ => None,
        });
        match (loaded, total_rows.get()) {
            (None, _) => String::new(),
            (Some(loaded), Some(total))
                if u64::try_from(loaded).is_ok_and(|loaded| loaded < total) =>
            {
                format!("Showing {loaded} of {total}")
            }
            (Some(loaded), _) => results_label(loaded),
        }
    };

    let error = move || {
        rows.with(|rows| match rows {
            Some(Err(message)) => error_alert(message.clone()),
            _ => ().into_any(),
        })
    };

    let today = Local::now().date_naive();
    let day_cells = move || {
        let shown = month.get();
        rows_by_day.with(|by_day| {
            let leading = (0..shown.leading_blank_days()).map(|_| blank_day());
            let days = shown.days().map(|day| {
                let count = by_day.get(&day).map_or(0, Vec::len);
                day_cell(day, count, day == today, selected_day)
            });
            let trailing = (0..shown.trailing_blank_days()).map(|_| blank_day());
            leading.chain(days).chain(trailing).collect_view()
        })
    };

    let dialog_columns: Vec<Column<Row>> = columns
        .into_iter()
        .filter(|column| column.key != date_column_key)
        .collect();
    let dialog_columns = StoredValue::new(dialog_columns);
    let row_href = StoredValue::new(row_href.unwrap_or_else(|| Arc::new(|_: &Row| String::new())));
    let open_day = move || {
        let Some(day) = selected_day.get() else {
            return ().into_any();
        };
        let chosen = rows_by_day.with_untracked(|by_day| by_day.get(&day).cloned());
        day_dialog(
            day,
            chosen.unwrap_or_default(),
            dialog_columns.get_value(),
            row_href.get_value(),
            selected_day,
        )
    };

    view! {
        <div class="space-y-4">
            {error} <div class="relative">
                <div class="rounded-surface border border-gray-200 bg-white">
                    {month_navigation(month, summary, show)} {weekday_header()}
                    <div class="grid grid-cols-7" aria-busy=move || pending.get().to_string()>
                        {day_cells}
                    </div>
                </div>
                {refresh_cover(pending, "inset-px rounded-surface-inner")}
            </div> {open_day}
        </div>
    }
    .into_any()
}

fn blank_day() -> AnyView {
    view! { <div class=format!("{CELL_SHAPE_CLASS} bg-gray-50")></div> }.into_any()
}

fn day_cell(
    day: NaiveDate,
    count: usize,
    is_today: bool,
    selected_day: RwSignal<Option<NaiveDate>>,
) -> AnyView {
    let number_class = if is_today {
        TODAY_NUMBER_CLASS
    } else {
        DAY_NUMBER_CLASS
    };
    let number = view! { <div class=number_class>{day.day()}</div> };
    if count == 0 {
        return view! { <div class=CELL_SHAPE_CLASS>{number}</div> }.into_any();
    }
    let label = format!("{}, {}", results_label(count), long_date(day));
    view! {
        <button
            type="button"
            class=format!("{CELL_SHAPE_CLASS} relative cursor-pointer text-left hover:bg-gray-50 focus-ring")
            aria-label=label
            on:click=move |_| selected_day.set(Some(day))
        >
            {number}
            <div class="pointer-events-none absolute inset-0 flex items-center justify-center">
                <span class=COUNT_BADGE_CLASS>{count}</span>
            </div>
        </button>
    }
    .into_any()
}

fn day_dialog<Row>(
    day: NaiveDate,
    chosen: Vec<Row>,
    columns: Vec<Column<Row>>,
    row_href: RowAttribute<Row>,
    selected_day: RwSignal<Option<NaiveDate>>,
) -> AnyView
where
    Row: Clone + Send + Sync + 'static,
{
    let rows = Signal::derive(move || Some(Ok(chosen.clone())));
    view! {
        <Dialog
            title=long_date(day)
            on_close=Callback::new(move |()| selected_day.set(None))
            size=DialogSize::Large
        >
            <DataTable rows=rows columns=columns.clone() search=Search::Off row_href=row_href.clone() />
        </Dialog>
    }
    .into_any()
}

fn month_step_button(
    label: &'static str,
    glyph: icons::LucideGlyph,
    on_step: impl Fn() + Send + Sync + 'static,
) -> AnyView {
    view! {
        <Tooltip label=label>
            <button
                type="button"
                class=MONTH_STEP_CLASS
                aria-label=label
                on:click=move |_| on_step()
            >
                {icons::icon(glyph, "h-4 w-4")}
            </button>
        </Tooltip>
    }
    .into_any()
}

fn month_navigation(
    month: Memo<Month>,
    summary: impl Fn() -> String + Send + Sync + 'static,
    show: impl Fn(Month) + Copy + Send + Sync + 'static,
) -> AnyView {
    view! {
        <div class=format!("{MONTH_BAR_CLASS} {HEADER_RULE_CLASS}")>
            <div class="flex items-center">
                {month_step_button(
                    "Previous Month",
                    icons::LucideGlyph::ChevronLeft,
                    move || show(month.get_untracked().previous()),
                )}
                <h2 class="w-36 text-center text-sm font-semibold text-gray-900">
                    {move || month.get().label()}
                </h2>
                {month_step_button(
                    "Next Month",
                    icons::LucideGlyph::ChevronRight,
                    move || show(month.get_untracked().next()),
                )}
            </div>
            <div class="flex items-center gap-3">
                <span class="text-xs text-gray-500 tabular-nums max-sm:hidden">{summary}</span>
                <button
                    type="button"
                    class=TODAY_BUTTON_CLASS
                    on:click=move |_| show(Month::current())
                >
                    "Today"
                </button>
            </div>
        </div>
    }
    .into_any()
}

fn weekday_header() -> AnyView {
    view! {
        <div class="grid grid-cols-7 border-b border-gray-100">
            {WEEKDAY_LABELS
                .iter()
                .map(|label| {
                    view! { <div class=format!("py-2 text-center {LABEL_CLASS}")>{*label}</div> }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATE_COLUMN: &str = "arrival";

    fn date(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, DATE_FORMAT).unwrap()
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
    fn the_month_query_keeps_the_shown_month_and_drops_paging_and_sort() {
        let mut query = TableQuery {
            sort_column_key: Some("code".to_owned()),
            sort_ascending: false,
            skipped_rows: 96,
            rows_per_page: 48,
            ..TableQuery::default()
        };
        Month::containing(date("2026-03-15")).show_in(&mut query, DATE_COLUMN);
        let month = month_query(query, DATE_COLUMN);
        let range = month.date_range.unwrap();
        assert_eq!(
            (range.from.as_str(), range.to.as_str()),
            ("2026-03-01", "2026-03-31")
        );
        assert_eq!(month.sort_column_key, None);
        assert_eq!(month.skipped_rows, 0);
        assert_eq!(month.rows_per_page, MONTH_ROW_CAP);
    }

    #[test]
    fn a_range_on_another_column_does_not_pick_the_month() {
        let mut query = TableQuery::default();
        Month::containing(date("2020-05-01")).show_in(&mut query, "ordered");
        assert_eq!(Month::shown_by(&query, DATE_COLUMN), Month::current());
    }
}
