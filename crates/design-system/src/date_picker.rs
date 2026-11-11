use chrono::{Datelike, Local, NaiveDate};
use leptos::prelude::*;

use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::calendar::{weekday_labels, GridDay, Month};
use crate::controls::{field_control_id, field_description_id, with_field_error};
use crate::icons;
use crate::popover::{Popover, PopoverContent, PopoverState, PopoverTrigger};
use crate::tooltip::Tooltip;

const WEEKDAY_CLASS: &str =
    "flex h-8 w-9 items-center justify-center text-xs font-medium text-gray-400";
const MONTH_STEP_CLASS: &str = "cursor-pointer rounded p-1.5 text-gray-600 hover:bg-gray-100";
const DAY_BUTTON: &str =
    "flex h-9 w-9 items-center justify-center rounded-full text-sm cursor-pointer";
const DATE_FORMAT: &str = "%Y-%m-%d";
const DEFAULT_PLACEHOLDER: &str = "Select Date";
const DEFAULT_CLEAR_LABEL: &str = "Clear Date";
const INVALID_TRIGGER_CLASS: &str = "border-red-400";
const TRIGGER_WIDTH_CLASS: &str = "min-w-40";

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
        let base = format!(
            "{} {TRIGGER_WIDTH_CLASS}",
            button_class(ButtonTone::Secondary, ButtonSize::Standard)
        );
        if invalid.get() {
            format!("{base} {INVALID_TRIGGER_CLASS}")
        } else {
            base
        }
    });
    let trigger_text = move || {
        match selected_date.get() {
        Some(date) => {
            view! { <span class="flex-1 text-left tabular-nums">{date.format(DATE_FORMAT).to_string()}</span> }
                .into_any()
        }
        None => view! { <span class="flex-1 text-left text-gray-400">{placeholder.clone()}</span> }.into_any(),
    }
    };
    let clear_button = clearable.then(|| {
        view! {
            <Tooltip label=clear_label.clone()>
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
                    prop:disabled=move || disabled.get()
                    on:click=move |_| value.set(String::new())
                >
                    {icons::icon(icons::LucideGlyph::X, "h-4 w-4")}
                </button>
            </Tooltip>
        }
    });

    view! {
        <div class="flex items-center gap-1">
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

fn day_button(
    grid_day: GridDay,
    selected_date: Option<NaiveDate>,
    today: NaiveDate,
    on_select: Callback<NaiveDate>,
) -> AnyView {
    let date = grid_day.date();
    let autofocus = starts_focused(date, selected_date, today).then_some("");
    view! {
        <button
            type="button"
            class=day_class(grid_day, selected_date, today)
            data-autofocus=autofocus
            on:click=move |_| on_select.run(date)
        >
            {date.day()}
        </button>
    }
    .into_any()
}

#[component]
fn CalendarPanel(
    selected_date: Option<NaiveDate>,
    on_select: Callback<NaiveDate>,
) -> impl IntoView {
    let today = Local::now().date_naive();
    let shown = RwSignal::new(Month::containing(selected_date.unwrap_or(today)));
    let day_cells = move || {
        shown
            .get()
            .grid_days()
            .into_iter()
            .map(|grid_day| day_button(grid_day, selected_date, today, on_select))
            .collect_view()
    };

    view! {
        <div>
            <div class="mb-2 flex items-center justify-between">
                <button
                    type="button"
                    class=MONTH_STEP_CLASS
                    aria-label="Previous Month"
                    on:click=move |_| shown.update(|month| *month = month.previous())
                >
                    {icons::icon(icons::LucideGlyph::ChevronLeft, "h-4 w-4")}
                </button>
                <span class="text-sm font-semibold text-gray-900">
                    {move || shown.get().label()}
                </span>
                <button
                    type="button"
                    class=MONTH_STEP_CLASS
                    aria-label="Next Month"
                    on:click=move |_| shown.update(|month| *month = month.next())
                >
                    {icons::icon(icons::LucideGlyph::ChevronRight, "h-4 w-4")}
                </button>
            </div>
            <div class="grid grid-cols-7 justify-items-center">{weekday_labels(WEEKDAY_CLASS)}</div>
            <div class="grid grid-cols-7 justify-items-center gap-y-0.5">{day_cells}</div>
            <div class="mt-2 flex justify-center border-t border-gray-100 pt-2">
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                    on:click=move |_| on_select.run(today)
                >
                    "Today"
                </button>
            </div>
        </div>
    }
    .into_any()
}

fn starts_focused(date: NaiveDate, selected_date: Option<NaiveDate>, today: NaiveDate) -> bool {
    selected_date.map_or(date == today, |selected| date == selected)
}

fn day_class(grid_day: GridDay, selected_date: Option<NaiveDate>, today: NaiveDate) -> String {
    let date = grid_day.date();
    let tone = if selected_date == Some(date) {
        "bg-blue-600 font-medium text-white"
    } else if date == today {
        "font-medium text-blue-700 ring-1 ring-inset ring-blue-300 hover:bg-blue-50"
    } else if matches!(grid_day, GridDay::Neighbouring(_)) {
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
