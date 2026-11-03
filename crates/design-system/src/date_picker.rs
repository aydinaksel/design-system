use chrono::{Local, NaiveDate};
use leptos::prelude::*;
use leptos_use::{use_calendar_with_options, CalendarDate, UseCalendarOptions, UseCalendarReturn};

use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::controls::{field_control_id, field_description_id, with_field_error};
use crate::icons;
use crate::popover::{Popover, PopoverContent, PopoverState, PopoverTrigger};

const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const DAY_BUTTON: &str =
    "flex h-9 w-9 items-center justify-center rounded-full text-sm cursor-pointer";
const DATE_FORMAT: &str = "%Y-%m-%d";
const DEFAULT_PLACEHOLDER: &str = "Select Date";
const DEFAULT_CLEAR_LABEL: &str = "Clear Date";
const INVALID_TRIGGER_CLASS: &str = "border-red-400 focus-visible:ring-red-500";

fn parsed_date(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), DATE_FORMAT).ok()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn DatePicker(
    value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional)] clearable: bool,
    #[prop(optional, into)] clear_label: Option<String>,
    #[prop(optional, into)] invalid: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let invalid = with_field_error(invalid);
    let popover = PopoverState::new();
    let placeholder = placeholder.unwrap_or_else(|| DEFAULT_PLACEHOLDER.to_owned());
    let clear_label = clear_label.unwrap_or_else(|| DEFAULT_CLEAR_LABEL.to_owned());
    let selected_date = Signal::derive(move || value.with(|text| parsed_date(text)));
    let trigger_class = Signal::derive(move || {
        let base = button_class(ButtonTone::Secondary, ButtonSize::Standard);
        if invalid.get() {
            format!("{base} {INVALID_TRIGGER_CLASS}")
        } else {
            base
        }
    });
    let trigger_text = move || match selected_date.get() {
        Some(date) => {
            view! { <span class="tabular-nums">{date.format(DATE_FORMAT).to_string()}</span> }
                .into_any()
        }
        None => view! { <span class="text-gray-400">{placeholder.clone()}</span> }.into_any(),
    };
    let clear_button = clearable.then(|| {
        view! {
            <button
                type="button"
                class=move || {
                    format!(
                        "{} {}",
                        icon_button_class(ButtonTone::Ghost, ButtonSize::Standard),
                        if selected_date.get().is_some() { "" } else { "invisible" },
                    )
                }
                aria-label=clear_label.clone()
                title=clear_label
                prop:disabled=move || disabled.get()
                on:click=move |_| value.set(String::new())
            >
                {icons::icon(icons::LucideGlyph::X, "h-4 w-4")}
            </button>
        }
    });

    view! {
        <div class="inline-flex items-center gap-1">
            <Popover state=popover>
                <PopoverTrigger
                    class=trigger_class
                    disabled=disabled
                    attr:id=field_control_id()
                    attr:aria-describedby=field_description_id()
                    attr:aria-invalid=move || invalid.get().to_string()
                >
                    {icons::icon(icons::LucideGlyph::CalendarDays, "h-4 w-4 shrink-0 opacity-70")}
                    {trigger_text}
                </PopoverTrigger>
                <PopoverContent>
                    <CalendarPanel
                        selected_date=selected_date.get_untracked()
                        on_select=Callback::new(move |chosen: NaiveDate| {
                            value.set(chosen.format(DATE_FORMAT).to_string());
                            popover.close();
                        })
                    />
                </PopoverContent>
            </Popover>
            {clear_button}
        </div>
    }
    .into_any()
}

#[component]
fn CalendarPanel(
    selected_date: Option<NaiveDate>,
    on_select: Callback<NaiveDate>,
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

    let select_day = move |chosen: NaiveDate| on_select.run(chosen);

    let day_cells = move || {
        dates
            .get()
            .into_iter()
            .map(|calendar_date| {
                let chosen: NaiveDate = *calendar_date;
                let label = calendar_date.format("%-d").to_string();
                let class = day_class(calendar_date, selected_date);
                let autofocus = starts_focused(calendar_date, selected_date).then_some("");
                view! {
                    <button
                        type="button"
                        class=class
                        data-autofocus=autofocus
                        on:click=move |_| select_day(chosen)
                    >
                        {label}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <div>
            <div class="mb-2 flex items-center justify-between">
                <button
                    type="button"
                    class="cursor-pointer rounded p-1.5 text-gray-600 hover:bg-gray-100"
                    aria-label="Previous Month"
                    on:click=move |_| previous_month()
                >
                    {icons::icon(icons::LucideGlyph::ChevronLeft, "h-4 w-4")}
                </button>
                <span class="text-sm font-semibold text-gray-900">{month_label}</span>
                <button
                    type="button"
                    class="cursor-pointer rounded p-1.5 text-gray-600 hover:bg-gray-100"
                    aria-label="Next Month"
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

fn starts_focused(calendar_date: CalendarDate, selected_date: Option<NaiveDate>) -> bool {
    match selected_date {
        Some(selected) => calendar_date.is_selected(&selected),
        None => calendar_date.is_today(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_iso_date_and_treats_anything_else_as_empty() {
        assert_eq!(
            parsed_date("2026-10-15"),
            NaiveDate::from_ymd_opt(2026, 10, 15)
        );
        assert_eq!(
            parsed_date(" 2026-10-15 "),
            NaiveDate::from_ymd_opt(2026, 10, 15)
        );
        assert_eq!(parsed_date(""), None);
        assert_eq!(parsed_date("15/10/2026"), None);
    }
}
