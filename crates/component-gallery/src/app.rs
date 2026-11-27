use leptos::context::Provider;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    ParamSegment, StaticSegment, WildcardSegment,
};

use design_system::{Brand, DocumentLinks, NotFoundPage, ToastProvider};

use crate::layout::GalleryLayout;
use crate::pages::{component::ComponentPage, index::IndexPage, sidebar::SidebarPage};

pub const FAVICON_PATH: &str = "/favicon.svg";

const DOCUMENT_LINKS: DocumentLinks = DocumentLinks {
    stylesheet_href: "/pkg/component-gallery.css",
    icon_href: FAVICON_PATH,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    design_system::shell(options, DOCUMENT_LINKS, App)
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
