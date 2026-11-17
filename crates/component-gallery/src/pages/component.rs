use design_system::{NotFoundPage, PageHeader};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::contents::{entry_for, Entry, Presentation};

const SLUG_PARAMETER: &str = "slug";

fn showcase(entry: Entry, content: AnyView) -> AnyView {
    view! {
        <main class="mx-auto max-w-screen-2xl space-y-6 px-6 py-8">
            <PageHeader title=entry.title description=entry.summary />
            <div class="rounded-xl border border-edge p-6">{content}</div>
        </main>
    }
    .into_any()
}

fn entry_view(slug: Option<&str>) -> AnyView {
    let Some(entry) = slug.and_then(entry_for) else {
        return view! { <NotFoundPage /> }.into_any();
    };
    match entry.presentation {
        Presentation::Showcase(content) => showcase(entry, content()),
        Presentation::Page(page) => page(),
        Presentation::Standalone => view! { <NotFoundPage /> }.into_any(),
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ComponentPage() -> impl IntoView {
    let parameters = use_params_map();
    let slug = Memo::new(move |_| parameters.read().get(SLUG_PARAMETER));
    move || entry_view(slug.get().as_deref())
}
