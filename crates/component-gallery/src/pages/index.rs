use design_system::icons;
use design_system::PageHeader;
use design_system::LABEL_CLASS;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::contents::{Entry, Group, GROUPS};

const ENTRY_GRID_CLASS: &str = "grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3";
const ENTRY_CLASS: &str = "flex items-center gap-3 rounded-xl border border-edge bg-surface p-4 transition-colors hover:border-edge-hover hover:bg-row-hover";
const ICON_TILE_CLASS: &str =
    "flex size-9 shrink-0 items-center justify-center rounded-lg bg-gray-100 text-gray-600";

fn entry_link(entry: Entry) -> AnyView {
    view! {
        <A href=entry.href attr:class=ENTRY_CLASS>
            <div class=ICON_TILE_CLASS>{icons::icon(entry.glyph, "size-4")}</div>
            <div class="min-w-0">
                <p class="truncate text-sm font-medium text-gray-900">{entry.title}</p>
                <p class="text-sm text-gray-500">{entry.summary}</p>
            </div>
        </A>
    }
    .into_any()
}

fn group_view(group: &Group) -> AnyView {
    let links = group.entries.iter().copied().map(entry_link).collect_view();
    view! {
        <section class="space-y-3">
            <h2 class=LABEL_CLASS>{group.title}</h2>
            <div class=ENTRY_GRID_CLASS>{links}</div>
        </section>
    }
    .into_any()
}

#[component]
pub fn IndexPage() -> impl IntoView {
    let groups = GROUPS.iter().map(group_view).collect_view();
    view! {
        <main class="mx-auto max-w-screen-2xl space-y-8 px-6 py-8">
            <PageHeader
                title="Component Gallery"
                description="Every component in the design system, drawn with fake data."
            />
            {groups}
        </main>
    }
    .into_any()
}
