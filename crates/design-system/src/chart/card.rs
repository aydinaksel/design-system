use leptos::prelude::*;

use super::frame::ChartFrame;
use super::{line_key, ChartColor};
use crate::element_key::ElementKey;
use crate::typography::{heading_element, HeadingLevel};

#[derive(Clone, Debug, PartialEq)]
pub struct ChartMetric {
    pub label: String,
    pub value: String,
    pub color: ChartColor,
}

const METRIC_CLASS: &str = "flex flex-1 cursor-pointer flex-col justify-center gap-1 border-t border-gray-200 px-6 py-4 text-left transition-colors even:border-l hover:bg-gray-50 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus aria-pressed:bg-gray-50 sm:border-t-0 sm:border-l sm:px-8 sm:py-6";
const TITLE_CLASS: &str = "text-base font-semibold text-gray-900";

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

fn spoken_name(title: &str, subtitle: Option<&String>) -> String {
    subtitle.map_or_else(|| title.to_owned(), |text| format!("{title}, {text}"))
}

fn source_line(source: Option<String>, source_href: Option<String>) -> Option<AnyView> {
    let text = source?;
    let named = match source_href {
        Some(href) => view! {
            <a href=href class="underline underline-offset-2 hover:text-gray-900">
                {text}
            </a>
        }
        .into_any(),
        None => text.into_any(),
    };
    Some(view! { <p>"Source: " {named}</p> }.into_any())
}

fn notes_list(notes: Vec<String>) -> Option<AnyView> {
    if notes.is_empty() {
        return None;
    }
    Some(
        view! {
            <ul class="space-y-0.5">
                {notes.into_iter().map(|note| view! { <li>{note}</li> }).collect_view()}
            </ul>
        }
        .into_any(),
    )
}

#[component]
pub fn ChartCard(
    #[prop(into)] title: String,
    #[prop(optional, into)] subtitle: Option<String>,
    #[prop(optional)] heading_level: HeadingLevel,
    #[prop(optional, into)] source: Option<String>,
    #[prop(optional, into)] source_href: Option<String>,
    #[prop(optional)] notes: Vec<String>,
    #[prop(optional)] metrics: Vec<ChartMetric>,
    #[prop(optional)] selected_metric: Option<RwSignal<usize>>,
    children: Children,
) -> impl IntoView {
    let card_number = ElementKey::next();
    let heading_id = format!("chart-card-{card_number}-title");
    let subtitle_id = format!("chart-card-{card_number}-subtitle");
    let labelled_by = if subtitle.is_some() {
        format!("{heading_id} {subtitle_id}")
    } else {
        heading_id.clone()
    };
    provide_context(ChartFrame {
        heading_level,
        labelled_by,
        name: spoken_name(&title, subtitle.as_ref()),
    });
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
    let footer_lines = [source_line(source, source_href), notes_list(notes)];
    let footer = footer_lines.iter().any(Option::is_some).then(|| {
        view! {
            <footer class="space-y-1 border-t border-gray-200 px-6 py-3 text-xs text-gray-500">
                {footer_lines}
            </footer>
        }
    });
    let section_labelled_by = heading_id.clone();
    view! {
        <section
            aria-labelledby=section_labelled_by
            class="overflow-hidden rounded-surface border border-gray-200 bg-surface shadow-xs"
        >
            <header class="flex flex-col items-stretch border-b border-gray-200 sm:flex-row">
                <div class="flex flex-1 flex-col justify-center gap-1 px-6 py-4">
                    {heading_element(heading_level, Some(heading_id), TITLE_CLASS, title)}
                    {subtitle
                        .map(|text| {
                            view! {
                                <p id=subtitle_id class="text-sm text-gray-500">
                                    {text}
                                </p>
                            }
                        })}
                </div>
                {metric_row}
            </header>
            <div class="px-2 py-4 sm:p-6">{children()}</div>
            {footer}
        </section>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::spoken_name;

    #[test]
    fn the_spoken_name_joins_title_and_subtitle() {
        assert_eq!(
            spoken_name("Sales by Day", Some(&"Jul 1 to Sep 30, 2026".to_owned())),
            "Sales by Day, Jul 1 to Sep 30, 2026"
        );
        assert_eq!(spoken_name("Sales by Day", None), "Sales by Day");
    }
}
