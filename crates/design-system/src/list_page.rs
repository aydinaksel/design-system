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
    #[prop(optional)] actions: Vec<MenuEntry>,
    #[prop(optional)] views: Option<&'static [PageView]>,
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
    let tabs = views.map(|views| ViewFn::from(move || view! { <ViewTabs views=views /> }));
    view! { <main>{banded_page(header, tabs, children().into_any())}</main> }.into_any()
}
