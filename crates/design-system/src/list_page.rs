use leptos::prelude::*;

use crate::menu::{DropdownMenu, MenuEntry};
use crate::page_band::banded_page;
use crate::page_header::PageHeader;
use crate::tabs::{PageView, ViewTabs};

const ACTIONS_LABEL: &str = "Actions";

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ListPage(
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] description: Option<Signal<String>>,
    /// Builds the Actions menu. It is asked again whenever something it
    /// reads changes, so the entries can follow the view the page is on.
    #[prop(optional, into)]
    actions: Option<Callback<(), Vec<MenuEntry>>>,
    #[prop(optional)] views: Option<&'static [PageView]>,
    children: Children,
) -> impl IntoView {
    let actions_menu = move || {
        let entries = actions.map(|actions| actions.run(())).unwrap_or_default();
        (!entries.is_empty())
            .then(|| view! { <DropdownMenu label=ACTIONS_LABEL entries=entries /> })
    };
    let header = match description {
        Some(description) => view! {
            <PageHeader title=title description=description>
                {actions_menu}
            </PageHeader>
        }
        .into_any(),
        None => view! { <PageHeader title=title>{actions_menu}</PageHeader> }.into_any(),
    };
    let tabs = views.map(|views| ViewFn::from(move || view! { <ViewTabs views=views /> }));
    view! { <main>{banded_page(header, tabs, children().into_any())}</main> }.into_any()
}
