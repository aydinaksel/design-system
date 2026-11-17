use design_system::icons::LucideGlyph;
use design_system::{NavigationItem, NavigationSection, Sidebar};
use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::contents::{Group, GROUPS};

const SIDEBAR_COOKIE_PREFIX: &str = "gallery_navigation";

fn group_navigation(group: &Group) -> AnyView {
    let items = group
        .entries
        .iter()
        .map(|entry| view! { <NavigationItem href=entry.href label=entry.title glyph=entry.glyph /> })
        .collect_view();
    view! {
        <NavigationSection key=group.key title=group.title>
            {items}
        </NavigationSection>
    }
    .into_any()
}

fn navigation() -> AnyView {
    let groups = GROUPS.iter().map(group_navigation).collect_view();
    view! {
        <NavigationItem href="/" label="Overview" glyph=LucideGlyph::LayoutDashboard />
        {groups}
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn GalleryLayout() -> impl IntoView {
    view! {
        <div class="flex h-dvh overflow-hidden">
            <Sidebar cookie_prefix=SIDEBAR_COOKIE_PREFIX>{navigation()}</Sidebar>
            <div class="min-w-0 flex-1 overflow-y-auto pt-12 sm:pt-0">
                <Outlet />
            </div>
        </div>
    }
    .into_any()
}
