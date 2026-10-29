use leptos::html::Span;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};
use leptos_router::params::ParamsMap;

use crate::layout_motion::slide_between;
use crate::table::clear_table_query_parameters;

const TAB_CLASS: &str = "relative cursor-pointer px-3 pb-2.5 pt-2 text-sm font-medium";
const UNDERLINE_CLASS: &str = "pointer-events-none absolute inset-x-0 bottom-0 h-0.5 bg-blue-600";

pub const VIEW_PARAMETER: &str = "view";

/// One named subset of a page's data, offered as a tab. The first view is the
/// default and keeps the URL free of its parameter. Changing tab starts the
/// page's tables over, with nothing searched, sorted, filtered or paged.
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

/// Rewrites the rest of the URL as one choice gives way to another, in the same
/// navigation that records it.
pub type ChoiceTransition = std::sync::Arc<dyn Fn(&'static str, &mut ParamsMap) + Send + Sync>;

/// The key the URL names under `parameter`, or the first key when it names none
/// of them.
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

/// Navigates to the current page with `parameter` set to `key`, leaving the
/// first key implicit so the default URL stays clean.
pub(crate) fn choice_navigator(
    parameter: &'static str,
    default_key: &'static str,
    transition: Option<ChoiceTransition>,
) -> impl Fn(&'static str) + Clone {
    let query = use_query_map();
    let location = use_location();
    let navigate = use_navigate();
    move |key: &'static str| {
        let mut parameters = query.get_untracked();
        parameters.remove(parameter);
        if key != default_key {
            parameters.insert(parameter, key.to_string());
        }
        if let Some(transition) = transition.as_ref() {
            transition(key, &mut parameters);
        }
        let target = format!(
            "{}{}",
            location.pathname.get_untracked(),
            parameters.to_query_string(),
        );
        navigate(
            &target,
            leptos_router::NavigateOptions {
                scroll: false,
                ..Default::default()
            },
        );
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

fn slide_underline_on_change<Key>(
    selected: impl Fn() -> Key + 'static,
    underlines: Vec<(Key, NodeRef<Span>)>,
) where
    Key: PartialEq + Clone + 'static,
{
    let underline_for = move |key: &Key| {
        underlines
            .iter()
            .find(|(candidate, _)| candidate == key)
            .and_then(|(_, node_ref)| node_ref.get_untracked())
    };
    Effect::new(move |previous: Option<Key>| {
        let current = selected();
        if let Some(previous) = previous.filter(|previous| *previous != current) {
            if let (Some(from), Some(to)) = (underline_for(&previous), underline_for(&current)) {
                slide_between(&from, &to);
            }
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
    let choose = choice_navigator(VIEW_PARAMETER, default_key, Some(start_the_tab_over));
    let underlines: Vec<(&'static str, NodeRef<Span>)> = views
        .iter()
        .map(|view| (view.key, NodeRef::new()))
        .collect();
    slide_underline_on_change(move || selected.get(), underlines.clone());

    view! {
        <div class="mb-4 flex flex-wrap gap-2 border-b border-gray-200" role="tablist">
            {views
                .iter()
                .zip(underlines)
                .map(|(view, (_, underline_ref))| {
                    let PageView { key, label } = *view;
                    let choose = choose.clone();
                    let is_selected = move || selected.get() == key;
                    view! {
                        <button
                            type="button"
                            role="tab"
                            aria-selected=move || is_selected().to_string()
                            class=move || tab_class(is_selected())
                            on:click=move |_| choose(key)
                        >
                            {label}
                            {underline(underline_ref, is_selected)}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

#[must_use]
pub fn tab_navigation() -> leptos_router::NavigateOptions {
    leptos_router::NavigateOptions {
        replace: true,
        scroll: false,
        ..Default::default()
    }
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
    let navigate = use_navigate();
    let location = use_location();
    let underlines: Vec<(usize, NodeRef<Span>)> = (0..tabs.len())
        .map(|index| (index, NodeRef::new()))
        .collect();
    slide_underline_on_change(move || active.get(), underlines.clone());

    view! {
        <div class="flex flex-wrap gap-2 border-b border-gray-200 mb-6">
            {tabs
                .into_iter()
                .zip(underlines)
                .map(|(label, (index, underline_ref))| {
                    let navigate = navigate.clone();
                    let pathname = location.pathname;
                    let target = move || format!("{}?tab={}", pathname.get(), tab_slug(label));
                    view! {
                        <button
                            type="button"
                            class=move || tab_class(active.get() == index)
                            on:click=move |_| navigate(&target(), tab_navigation())
                        >
                            {label}
                            {underline(underline_ref, move || active.get() == index)}
                        </button>
                    }
                        .into_any()
                })
                .collect::<Vec<AnyView>>()}
        </div>
    }
    .into_any()
}
