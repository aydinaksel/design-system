use std::cell::Cell;

use leptos::html::Span;
use leptos::prelude::*;
use leptos::web_sys::HtmlSpanElement as HtmlSpan;
use leptos_router::hooks::{use_location, use_query_map};
use leptos_router::params::ParamsMap;

use crate::layout_motion::{horizontal_extent, slide_between, slide_from, HorizontalExtent};
use crate::table::clear_table_query_parameters;

const TAB_CLASS: &str = "relative cursor-pointer px-3 pb-2.5 pt-2 text-sm font-medium";
const UNDERLINE_CLASS: &str = "pointer-events-none absolute inset-x-0 bottom-0 h-0.5 bg-blue-600";
const TAB_LIST_CLASS: &str = "mb-6 flex flex-wrap gap-2 border-b border-edge";
const SHARED_BORDER_TAB_LIST_CLASS: &str = "-mb-px flex flex-wrap gap-2";

#[derive(Clone, Copy)]
pub(crate) struct TabsOnSharedBorder;

fn tab_list_class() -> &'static str {
    if use_context::<TabsOnSharedBorder>().is_some() {
        SHARED_BORDER_TAB_LIST_CLASS
    } else {
        TAB_LIST_CLASS
    }
}

pub const VIEW_PARAMETER: &str = "view";
const VIEWS_LABEL: &str = "Views";
const SECTIONS_LABEL: &str = "Sections";

#[derive(Clone, Copy)]
pub struct PageView {
    pub key: &'static str,
    pub label: &'static str,
}

impl PageView {
    #[must_use]
    pub const fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

pub type ChoiceTransition = std::sync::Arc<dyn Fn(&'static str, &mut ParamsMap) + Send + Sync>;

pub(crate) fn selected_key(parameter: &'static str, keys: Vec<&'static str>) -> Memo<&'static str> {
    let query = use_query_map();
    Memo::new(move |_| {
        let requested = query.read().get(parameter).unwrap_or_default();
        keys.iter()
            .find(|key| **key == requested)
            .or_else(|| keys.first())
            .copied()
            .unwrap_or_default()
    })
}

pub(crate) fn choice_href(
    parameter: &'static str,
    default_key: &'static str,
    transition: Option<ChoiceTransition>,
) -> impl Fn(&'static str) -> String + Clone + Send + Sync + 'static {
    let query = use_query_map();
    let location = use_location();
    move |key: &'static str| {
        let mut parameters = query.get();
        parameters.remove(parameter);
        if key != default_key {
            parameters.insert(parameter, key.to_string());
        }
        if let Some(transition) = transition.as_ref() {
            transition(key, &mut parameters);
        }
        format!(
            "{}{}",
            location.pathname.get(),
            parameters.to_query_string()
        )
    }
}

fn underline(
    node_ref: NodeRef<Span>,
    is_selected: impl Fn() -> bool + Send + Sync + 'static,
) -> AnyView {
    view! {
        <span
            node_ref=node_ref
            aria-hidden="true"
            class=move || {
                if is_selected() {
                    format!("{UNDERLINE_CLASS} opacity-100")
                } else {
                    format!("{UNDERLINE_CLASS} opacity-0")
                }
            }
        ></span>
    }
    .into_any()
}

fn tab_class(is_selected: bool) -> String {
    if is_selected {
        format!("{TAB_CLASS} text-blue-700")
    } else {
        format!("{TAB_CLASS} text-gray-500 hover:text-gray-800")
    }
}

#[derive(Clone, Copy)]
struct DepartedUnderline {
    destination_key: &'static str,
    extent: HorizontalExtent,
}

thread_local! {
    static DEPARTED_VIEW_UNDERLINE: Cell<Option<DepartedUnderline>> = const { Cell::new(None) };
}

fn remember_departed_underline(destination_key: &'static str, underline: Option<HtmlSpan>) {
    let departed = underline.map(|underline| DepartedUnderline {
        destination_key,
        extent: horizontal_extent(&underline),
    });
    DEPARTED_VIEW_UNDERLINE.with(|cell| cell.set(departed));
}

fn take_departed_underline() -> Option<DepartedUnderline> {
    DEPARTED_VIEW_UNDERLINE.with(Cell::take)
}

fn underline_for<Key: PartialEq>(
    underlines: &[(Key, NodeRef<Span>)],
    key: &Key,
) -> Option<HtmlSpan> {
    underlines
        .iter()
        .find(|(candidate, _)| candidate == key)
        .and_then(|(_, node_ref)| node_ref.get_untracked())
}

fn slide_underline_on_change<Key>(
    selected: impl Fn() -> Key + 'static,
    underlines: Vec<(Key, NodeRef<Span>)>,
    arrival_from_a_replaced_strip: impl Fn(&Key) -> Option<HorizontalExtent> + 'static,
) where
    Key: PartialEq + Clone + 'static,
{
    Effect::new(move |previous: Option<Key>| {
        let current = selected();
        let Some(to) = underline_for(&underlines, &current) else {
            return current;
        };
        match previous {
            None => {
                if let Some(extent) = arrival_from_a_replaced_strip(&current) {
                    slide_from(extent, &to);
                }
            }
            Some(previous) if previous != current => {
                take_departed_underline();
                if let Some(from) = underline_for(&underlines, &previous) {
                    slide_between(&from, &to);
                }
            }
            Some(_) => {}
        }
        current
    });
}

#[must_use]
pub fn selected_view(views: &'static [PageView]) -> Memo<&'static str> {
    selected_key(VIEW_PARAMETER, views.iter().map(|view| view.key).collect())
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ViewTabs(views: &'static [PageView]) -> impl IntoView {
    let selected = selected_view(views);
    let default_key = views.first().map_or("", |view| view.key);
    let start_the_tab_over: ChoiceTransition =
        std::sync::Arc::new(|_, parameters| clear_table_query_parameters(parameters));
    let href = choice_href(VIEW_PARAMETER, default_key, Some(start_the_tab_over));
    let underlines: Vec<(&'static str, NodeRef<Span>)> = views
        .iter()
        .map(|view| (view.key, NodeRef::new()))
        .collect();
    slide_underline_on_change(
        move || selected.get(),
        underlines.clone(),
        |current_key| {
            take_departed_underline()
                .filter(|departed| departed.destination_key == *current_key)
                .map(|departed| departed.extent)
        },
    );
    let departing_underline = {
        let underlines = underlines.clone();
        move || underline_for(&underlines, &selected.get_untracked())
    };

    view! {
        <nav class=tab_list_class() aria-label=VIEWS_LABEL>
            {views
                .iter()
                .zip(underlines)
                .map(|(view, (_, underline_ref))| {
                    let PageView { key, label } = *view;
                    let href = href.clone();
                    let departing_underline = departing_underline.clone();
                    let is_selected = move || selected.get() == key;
                    view! {
                        <a
                            href=move || href(key)
                            data-noscroll=""
                            aria-current=move || is_selected().then_some("page")
                            class=move || tab_class(is_selected())
                            on:click=move |_| {
                                remember_departed_underline(key, departing_underline());
                            }
                        >
                            {label}
                            {underline(underline_ref, is_selected)}
                        </a>
                    }
                })
                .collect_view()}
        </nav>
    }
    .into_any()
}

#[must_use]
pub fn tab_slug(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[must_use]
pub fn selected_tab(labels: &[&str], requested: Option<&str>) -> usize {
    requested
        .and_then(|slug| labels.iter().position(|label| tab_slug(label) == slug))
        .unwrap_or_default()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn TabStrip(#[prop(into)] tabs: Vec<&'static str>, active: Signal<usize>) -> impl IntoView {
    let location = use_location();
    let underlines: Vec<(usize, NodeRef<Span>)> = (0..tabs.len())
        .map(|index| (index, NodeRef::new()))
        .collect();
    slide_underline_on_change(move || active.get(), underlines.clone(), |_| None);

    view! {
        <nav class=tab_list_class() aria-label=SECTIONS_LABEL>
            {tabs
                .into_iter()
                .zip(underlines)
                .map(|(label, (index, underline_ref))| {
                    let pathname = location.pathname;
                    let href = move || format!("{}?tab={}", pathname.get(), tab_slug(label));
                    view! {
                        <a
                            href=href
                            data-noscroll=""
                            prop:replace=true
                            aria-current=move || (active.get() == index).then_some("page")
                            class=move || tab_class(active.get() == index)
                        >
                            {label}
                            {underline(underline_ref, move || active.get() == index)}
                        </a>
                    }
                        .into_any()
                })
                .collect::<Vec<AnyView>>()}
        </nav>
    }
    .into_any()
}
