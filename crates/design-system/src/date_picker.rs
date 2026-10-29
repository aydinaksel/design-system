use chrono::{Local, NaiveDate};
use leptos::prelude::*;
use leptos_use::{use_calendar_with_options, CalendarDate, UseCalendarOptions, UseCalendarReturn};

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::icons;

const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const DAY_BUTTON: &str =
    "flex h-9 w-9 items-center justify-center rounded-full text-sm cursor-pointer";

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn DatePicker(
    #[prop(into)] date: String,
    #[prop(into)] on_select: Callback<String>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let selected_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok();

    view! {
        <div class="relative">
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                {icons::icon(icons::LucideGlyph::CalendarDays, "h-4 w-4 shrink-0 opacity-70")}
                <span class="tabular-nums">{date}</span>
            </button>
            {move || {
                is_open
                    .get()
                    .then(|| {
                        view! {
                            <div
                                class="fixed inset-0 z-10"
                                on:click=move |_| is_open.set(false)
                            ></div>
                            <CalendarPanel
                                selected_date=selected_date
                                is_open=is_open
                                on_select=on_select
                            />
                        }
                    })
            }}
        </div>
    }
    .into_any()
}

#[component]
fn CalendarPanel(
    selected_date: Option<NaiveDate>,
    is_open: RwSignal<bool>,
    on_select: Callback<String>,
) -> impl IntoView {
    let options = UseCalendarOptions::default().initial_date(selected_date);
    let UseCalendarReturn {
        previous_month,
        next_month,
        dates,
        ..
    } = use_calendar_with_options(options);

    let month_label = move || {
        dates
            .get()
            .iter()
            .find(|date| !date.is_other_month())
            .map(|date| date.format("%B %Y").to_string())
            .unwrap_or_default()
    };

    let select_day = move |chosen: NaiveDate| {
        is_open.set(false);
        on_select.run(chosen.format("%Y-%m-%d").to_string());
    };

    let day_cells = move || {
        dates
            .get()
            .into_iter()
            .map(|calendar_date| {
                let chosen: NaiveDate = *calendar_date;
                let label = calendar_date.format("%-d").to_string();
                let class = day_class(calendar_date, selected_date);
                view! {
                    <button type="button" class=class on:click=move |_| select_day(chosen)>
                        {label}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <div class="absolute left-1/2 top-full z-20 mt-2 w-72 -translate-x-1/2 rounded-lg border border-gray-200 bg-white p-3 shadow-lg">
            <div class="mb-2 flex items-center justify-between">
                <button
                    type="button"
                    class="cursor-pointer rounded p-1.5 text-gray-600 hover:bg-gray-100"
                    on:click=move |_| previous_month()
                >
                    {icons::icon(icons::LucideGlyph::ChevronLeft, "h-4 w-4")}
                </button>
                <span class="text-sm font-semibold text-gray-900">{month_label}</span>
                <button
                    type="button"
                    class="cursor-pointer rounded p-1.5 text-gray-600 hover:bg-gray-100"
                    on:click=move |_| next_month()
                >
                    {icons::icon(icons::LucideGlyph::ChevronRight, "h-4 w-4")}
                </button>
            </div>
            <div class="grid grid-cols-7 justify-items-center">
                {WEEKDAY_LABELS
                    .iter()
                    .map(|label| {
                        view! {
                            <div class="flex h-8 w-9 items-center justify-center text-xs font-medium text-gray-400">
                                {*label}
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="grid grid-cols-7 justify-items-center gap-y-0.5">{day_cells}</div>
            <div class="mt-2 flex justify-center border-t border-gray-100 pt-2">
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                    on:click=move |_| select_day(Local::now().date_naive())
                >
                    "Today"
                </button>
            </div>
        </div>
    }
    .into_any()
}

fn day_class(calendar_date: CalendarDate, selected_date: Option<NaiveDate>) -> String {
    let tone = if selected_date.is_some_and(|selected| calendar_date.is_selected(&selected)) {
        "bg-blue-600 font-medium text-white"
    } else if calendar_date.is_today() {
        "font-medium text-blue-700 ring-1 ring-inset ring-blue-300 hover:bg-blue-50"
    } else if calendar_date.is_other_month() {
        "text-gray-300 hover:bg-gray-50"
    } else {
        "text-gray-700 hover:bg-gray-100"
    };
    format!("{DAY_BUTTON} {tone}")
}
