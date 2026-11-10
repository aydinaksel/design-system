use leptos::prelude::*;

use super::{line_key, ChartColor};

#[derive(Clone, Debug, PartialEq)]
pub struct ChartMetric {
    pub label: String,
    pub value: String,
    pub color: ChartColor,
}

const METRIC_CLASS: &str = "flex flex-1 cursor-pointer flex-col justify-center gap-1 border-t border-gray-200 px-6 py-4 text-left transition-colors even:border-l hover:bg-gray-50 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus aria-pressed:bg-gray-50 sm:border-t-0 sm:border-l sm:px-8 sm:py-6";

fn metric_button(index: usize, metric: ChartMetric, selected: RwSignal<usize>) -> impl IntoView {
    view! {
        <button
            type="button"
            class=METRIC_CLASS
            aria-pressed=move || (selected.get() == index).to_string()
            on:click=move |_| selected.set(index)
        >
            <span class="flex items-center gap-2 text-xs text-gray-500">
                {line_key(metric.color)} {metric.label}
            </span>
            <span class="text-lg leading-none font-bold text-gray-900 sm:text-3xl">
                {metric.value}
            </span>
        </button>
    }
}

#[component]
pub fn ChartCard(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] metrics: Vec<ChartMetric>,
    #[prop(optional)] selected_metric: Option<RwSignal<usize>>,
    children: Children,
) -> impl IntoView {
    let metric_row = selected_metric
        .filter(|_| !metrics.is_empty())
        .map(|selected| {
            view! {
                <div class="flex">
                    {metrics
                        .into_iter()
                        .enumerate()
                        .map(|(index, metric)| metric_button(index, metric, selected))
                        .collect_view()}
                </div>
            }
        });
    view! {
        <section class="overflow-hidden rounded-surface border border-gray-200 bg-surface shadow-xs">
            <header class="flex flex-col items-stretch border-b border-gray-200 sm:flex-row">
                <div class="flex flex-1 flex-col justify-center gap-1 px-6 py-4">
                    <h3 class="text-base font-semibold text-gray-900">{title}</h3>
                    {description.map(|text| view! { <p class="text-sm text-gray-500">{text}</p> })}
                </div>
                {metric_row}
            </header>
            <div class="px-2 py-4 sm:p-6">{children()}</div>
        </section>
    }
    .into_any()
}
