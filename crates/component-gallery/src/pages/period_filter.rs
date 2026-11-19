use chrono::Local;
use design_system::date_period::{FIRST_DATE_PARAMETER, LAST_DATE_PARAMETER, PERIOD_PARAMETER};
use design_system::{resolve_date_span, PeriodFilter};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

pub(crate) fn period_filter() -> AnyView {
    let query = use_query_map();
    let current = Signal::derive(move || {
        query.with(|parameters| {
            resolve_date_span(
                parameters.get(PERIOD_PARAMETER).as_deref(),
                parameters.get(FIRST_DATE_PARAMETER).as_deref(),
                parameters.get(LAST_DATE_PARAMETER).as_deref(),
                Local::now().date_naive(),
            )
        })
    });
    let covered = move || {
        current.get().map(|span| {
            format!(
                "The report would cover {} through {}.",
                span.first_text(),
                span.last_text()
            )
        })
    };
    view! {
        <PeriodFilter current=current />
        <p class="mt-4 text-sm text-gray-700" aria-live="polite">
            {covered}
        </p>
    }
    .into_any()
}
