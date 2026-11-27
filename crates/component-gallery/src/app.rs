use leptos::context::Provider;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    ParamSegment, StaticSegment, WildcardSegment,
};

use design_system::{Brand, NotFoundPage, ToastProvider};

use crate::layout::GalleryLayout;
use crate::pages::{component::ComponentPage, index::IndexPage, sidebar::SidebarPage};

const STYLESHEET_HREF: &str = "/pkg/component-gallery.css";

pub fn shell(options: LeptosOptions) -> impl IntoView {
    design_system::shell(options, STYLESHEET_HREF, App)
}

const fn brand() -> Brand {
    Brand {
        name: "Component Gallery",
        home_href: "/",
        home_link_label: "Overview",
        footer_note: "Every component here is drawn from design-system with fake data.",
        header_action: None,
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="Component Gallery" />
        <Provider value=brand()>
            <ToastProvider>
                <Router>
                    <Routes fallback=NotFoundPage>
                        <Route path=StaticSegment("sidebar") view=SidebarPage />
                        <Route
                            path=(StaticSegment("sidebar"), WildcardSegment("page"))
                            view=SidebarPage
                        />
                        <ParentRoute path=StaticSegment("") view=GalleryLayout>
                            <Route path=StaticSegment("") view=IndexPage />
                            <Route path=ParamSegment("slug") view=ComponentPage />
                        </ParentRoute>
                    </Routes>
                </Router>
            </ToastProvider>
        </Provider>
    }
    .into_any()
}
