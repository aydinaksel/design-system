use leptos::prelude::*;

use crate::layout::PAGE_WIDTH_CLASS;
use crate::menu::{DropdownMenu, MenuEntry};
use crate::page_header::PageHeader;

const ACTIONS_LABEL: &str = "Actions";

/// A page built around one table: the title, with every page action gathered
/// into one Actions menu at the top right, then the table, at the same width
/// as a `DetailPage`. Hand the table its `views` so the tabs share the search
/// row.
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ListPage(
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] description: Option<Signal<String>>,
    #[prop(optional)] actions: Vec<MenuEntry>,
    children: Children,
) -> impl IntoView {
    let actions = (!actions.is_empty())
        .then(|| view! { <DropdownMenu label=ACTIONS_LABEL entries=actions /> }.into_any());
    let header = match (description, actions) {
        (Some(description), Some(actions)) => view! {
            <PageHeader title=title description=description>
                {actions}
            </PageHeader>
        }
        .into_any(),
        (Some(description), None) => {
            view! { <PageHeader title=title description=description /> }.into_any()
        }
        (None, Some(actions)) => {
            view! { <PageHeader title=title>{actions}</PageHeader> }.into_any()
        }
        (None, None) => view! { <PageHeader title=title /> }.into_any(),
    };
    view! { <main class=format!("{PAGE_WIDTH_CLASS} py-8")>{header} {children()}</main> }.into_any()
}
