use leptos::prelude::*;

pub struct Total {
    label: &'static str,
    value: String,
}

#[must_use]
pub fn total(label: &'static str, value: impl Into<String>) -> Total {
    Total {
        label,
        value: value.into(),
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn TotalsBar(totals: Vec<Total>) -> impl IntoView {
    view! {
        <dl class="mt-4 flex flex-col divide-y divide-gray-100 rounded-lg border border-gray-200 bg-white text-sm sm:flex-row sm:flex-wrap sm:justify-end sm:gap-x-8 sm:gap-y-2 sm:divide-y-0 sm:px-5 sm:py-3">
            {totals
                .into_iter()
                .map(|Total { label, value }| {
                    view! {
                        <div class="flex items-baseline justify-between gap-4 px-4 py-2 sm:gap-1.5 sm:p-0">
                            <dt class="text-gray-500">{label}</dt>
                            <dd class="font-semibold tabular-nums text-gray-900">{value}</dd>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </dl>
    }
    .into_any()
}
