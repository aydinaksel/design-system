use leptos::prelude::*;
use leptos_router::components::A;

use crate::brand::brand;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn NotFoundPage() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
            response.set_status(http::StatusCode::NOT_FOUND);
        }
    }

    let brand = brand();

    view! {
        <main class="mx-auto max-w-5xl px-6 py-20 text-center">
            <h1 class="text-2xl font-semibold tracking-tight text-gray-900">"Page not found"</h1>
            <p class="mt-2 text-sm text-gray-600">"The page you asked for does not exist."</p>
            <p class="mt-6">
                <A href=brand.home_href>
                    <span class="text-sm text-gray-500 hover:text-gray-900">
                        {brand.home_link_label}
                    </span>
                </A>
            </p>
        </main>
    }
    .into_any()
}
