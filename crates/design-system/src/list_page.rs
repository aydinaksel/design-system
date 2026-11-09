use leptos::prelude::*;

use crate::menu::{DropdownMenu, MenuEntry};
use crate::page_header::PageHeader;

const ACTIONS_LABEL: &str = "Actions";

/// A page built around one table: the title, with every page action gathered
/// into one Actions menu at the top right, then the table. Hand the table its
/// `views` so the tabs share the search row.
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ListPage(
    #[prop(into)] title: Signal<String>,
    #[prop(optional)] actions: Vec<MenuEntry>,
    children: Children,
) -> impl IntoView {
    let header = if actions.is_empty() {
        view! { <PageHeader title=title /> }.into_any()
    } else {
        view! {
            <PageHeader title=title>
                <DropdownMenu label=ACTIONS_LABEL entries=actions />
            </PageHeader>
        }
        .into_any()
    };
    view! {
        {header}
        {children()}
    }
    .into_any()
}
