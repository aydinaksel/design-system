use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes, A},
    StaticSegment,
};

use design_system::{Brand, Layout, NotFoundPage, ToastProvider};

use crate::pages::{calendar::CalendarPage, overview::OverviewPage, tables::TablesPage};

const STYLESHEET_HREF: &str = "/pkg/component-gallery.css";

const NAVIGATION: [(&str, &str); 3] = [
    ("/", "Overview"),
    ("/tables", "Tables"),
    ("/calendar", "Calendar"),
];

pub fn shell(options: LeptosOptions) -> impl IntoView {
    design_system::shell(options, STYLESHEET_HREF, App)
}

fn navigation() -> AnyView {
    view! {
        <nav class="flex gap-4 text-sm">
            {NAVIGATION
                .iter()
                .map(|(href, label)| {
                    view! {
                        <A href=*href attr:class="text-gray-500 hover:text-gray-900">
                            {*label}
                        </A>
                    }
                })
                .collect_view()}
        </nav>
    }
    .into_any()
}

fn brand() -> Brand {
    Brand {
        name: "Component Gallery",
        home_href: "/",
        home_link_label: "Overview",
        footer_note: "Every component here is drawn from design-system with fake data.",
        header_action: None,
    }
    .with_header_action(navigation)
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(brand());

    view! {
        <Title text="Component Gallery" />
        <ToastProvider>
            <Router>
                <Routes fallback=NotFoundPage>
                    <ParentRoute path=StaticSegment("") view=Layout>
                        <Route path=StaticSegment("") view=OverviewPage />
                        <Route path=StaticSegment("tables") view=TablesPage />
                        <Route path=StaticSegment("calendar") view=CalendarPage />
                    </ParentRoute>
                </Routes>
            </Router>
        </ToastProvider>
    }
    .into_any()
}
