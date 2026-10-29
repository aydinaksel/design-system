use std::sync::Arc;

use leptos::prelude::*;
use leptos_use::{use_calendar, UseCalendarReturn};

use super::column::Column;
use super::data_table::{DataTable, RowAttribute, Search};
use super::url_query::UrlTableQuery;
use crate::icons;

const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const DAY_CELL: &str = "flex min-h-20 flex-col border-b border-r border-gray-100 p-1.5";

pub type DateOfRow<Row> = Arc<dyn Fn(&Row) -> Option<String> + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibleMonth {
    pub first_day: String,
    pub last_day: String,
    pub label: String,
}

/// A month of rows, counted per day, with the chosen day's rows beneath.
#[component]
pub fn CalendarView<Row>(
    rows: Signal<Option<Result<Vec<Row>, String>>>,
    columns: Vec<Column<Row>>,
    date_of_row: DateOfRow<Row>,
    query: UrlTableQuery,
    date_column_key: &'static str,
    #[prop(optional)] row_href: Option<RowAttribute<Row>>,
    #[prop(optional, into)] empty_message: String,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
{
    let UseCalendarReturn {
        previous_month,
        next_month,
        dates,
        ..
    } = use_calendar();
    let selected_day: RwSignal<Option<String>> = RwSignal::new(None);

    let visible_month = Signal::derive(move || {
        let days = dates.get();
        let own_month: Vec<_> = days
            .iter()
            .filter(|date| !date.is_other_month())
            .collect::<Vec<_>>();
        let first = own_month.first()?;
        let last = own_month.last()?;
        Some(VisibleMonth {
            first_day: first.format("%Y-%m-%d").to_string(),
            last_day: last.format("%Y-%m-%d").to_string(),
            label: first.format("%B %Y").to_string(),
        })
    });

    Effect::new(move |_| {
        let Some(month) = visible_month.get() else {
            return;
        };
        query.update(|current| {
            current.show_month(date_column_key, month.first_day, month.last_day);
        });
        selected_day.set(None);
    });

    let date_of_row = StoredValue::new(date_of_row);
    let rows_on = move |day: &str| -> Vec<Row> {
        let Some(Ok(loaded)) = rows.get() else {
            return Vec::new();
        };
        date_of_row.with_value(|date_of_row| {
            loaded
                .into_iter()
                .filter(|row| date_of_row(row).as_deref() == Some(day))
                .collect()
        })
    };

    let day_cells = move || {
        dates
            .get()
            .into_iter()
            .map(|calendar_date| {
                if calendar_date.is_other_month() {
                    return view! { <div class=DAY_CELL></div> }.into_any();
                }
                let day = calendar_date.format("%Y-%m-%d").to_string();
                let day_number = calendar_date.format("%-d").to_string();
                let count = rows_on(&day).len();
                day_cell(day, day_number, count, selected_day)
            })
            .collect_view()
    };

    let day_table = DayTable {
        columns: StoredValue::new(columns),
        row_href: StoredValue::new(row_href.unwrap_or_else(|| Arc::new(|_| String::new()))),
        empty_message: StoredValue::new(empty_message),
    };

    let selected_rows = move || match selected_day.get() {
        Some(day) => day_table.render(rows_on(&day), &day, selected_day),
        None => ().into_any(),
    };

    let counted = move || {
        let shown = match rows.get() {
            Some(Ok(loaded)) => loaded.len(),
            _ => return String::new(),
        };
        let month = visible_month
            .get()
            .map(|month| month.label)
            .unwrap_or_default();
        format!("{shown} in {month}")
    };

    let month_label = Signal::derive(move || {
        visible_month
            .get()
            .map(|month| month.label)
            .unwrap_or_default()
    });
    view! {
        <div class="space-y-4">
            {month_navigation(month_label, counted, previous_month, next_month)}
            <div class="rounded-lg border border-gray-200 bg-white shadow-sm">
                {weekday_header()} <div class="grid grid-cols-7">{day_cells}</div>
            </div> {selected_rows}
        </div>
    }
    .into_any()
}
fn day_cell(
    day: String,
    day_number: String,
    count: usize,
    selected_day: RwSignal<Option<String>>,
) -> AnyView {
    let is_selected = selected_day.get().as_deref() == Some(day.as_str());
    let cell_class = if is_selected {
        format!("{DAY_CELL} cursor-pointer bg-blue-50 ring-1 ring-inset ring-blue-300")
    } else if count > 0 {
        format!("{DAY_CELL} cursor-pointer hover:bg-gray-50")
    } else {
        DAY_CELL.to_owned()
    };
    view! {
        <div
            class=cell_class
            on:click=move |_| {
                if count > 0 {
                    selected_day.set(Some(day.clone()));
                }
            }
        >
            <div class="text-xs leading-none text-gray-500">{day_number}</div>
            {(count > 0)
                .then(|| {
                    view! {
                        <div class="flex flex-1 items-center justify-center">
                            <span class="flex h-6 w-6 items-center justify-center rounded-full bg-blue-100 text-xs font-medium text-blue-700">
                                {count}
                            </span>
                        </div>
                    }
                })}
        </div>
    }
    .into_any()
}

fn month_navigation(
    label: Signal<String>,
    counted: impl Fn() -> String + Send + Sync + 'static,
    previous_month: impl Fn() + 'static,
    next_month: impl Fn() + 'static,
) -> AnyView {
    view! {
        <div class="flex items-center gap-3">
            <button
                class="cursor-pointer rounded p-1.5 text-gray-600 hover:bg-gray-100"
                on:click=move |_| previous_month()
            >
                {icons::icon(icons::LucideGlyph::ChevronLeft, "h-5 w-5")}
            </button>
            <h2 class="w-44 text-center text-lg font-semibold text-gray-900">{label}</h2>
            <button
                class="cursor-pointer rounded p-1.5 text-gray-600 hover:bg-gray-100"
                on:click=move |_| next_month()
            >
                {icons::icon(icons::LucideGlyph::ChevronRight, "h-5 w-5")}
            </button>
            <span class="text-sm text-gray-500 tabular-nums">{counted}</span>
        </div>
    }
    .into_any()
}

fn weekday_header() -> AnyView {
    view! {
        <div class="grid grid-cols-7 border-b border-gray-200 bg-gray-50">
            {WEEKDAY_LABELS
                .iter()
                .map(|label| {
                    view! {
                        <div class="px-2 py-2 text-center text-xs font-medium text-gray-500">
                            {*label}
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

#[derive(Clone)]
struct DayTable<Row: Clone + Send + Sync + 'static> {
    columns: StoredValue<Vec<Column<Row>>>,
    row_href: StoredValue<RowAttribute<Row>>,
    empty_message: StoredValue<String>,
}

impl<Row: Clone + Send + Sync + 'static> Copy for DayTable<Row> {}

impl<Row: Clone + Send + Sync + 'static> DayTable<Row> {
    fn render(
        self,
        chosen: Vec<Row>,
        day: &str,
        selected_day: RwSignal<Option<String>>,
    ) -> AnyView {
        if chosen.is_empty() {
            return ().into_any();
        }
        let heading = format!("{} on {day}", chosen.len());
        view! {
            <div class="space-y-2">
                <div class="flex items-center justify-between">
                    <h3 class="text-sm font-medium text-gray-900">{heading}</h3>
                    <button
                        class="cursor-pointer text-xs text-gray-400 hover:text-gray-600"
                        on:click=move |_| selected_day.set(None)
                    >
                        "Close"
                    </button>
                </div>
                <DataTable
                    rows=Signal::derive(move || Some(Ok(chosen.clone())))
                    columns=self.columns.get_value()
                    search=Search::Off
                    empty_message=self.empty_message.get_value()
                    row_href=self.row_href.get_value()
                />
            </div>
        }
        .into_any()
    }
}
