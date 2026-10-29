use leptos::prelude::*;
use leptos_router::components::{Outlet, A};

use crate::brand::brand;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Layout() -> impl IntoView {
    let brand = brand();
    let header_action = brand.header_action.as_ref().map(|action| action());

    view! {
        <div class="min-h-screen bg-white">
            <header class="border-b border-gray-200">
                <div class="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
                    <A href=brand.home_href>
                        <span class="text-sm font-semibold tracking-tight text-gray-900">
                            {brand.name}
                        </span>
                    </A>
                    {header_action}
                </div>
            </header>

            <Outlet />

            <footer class="mt-16 border-t border-gray-200">
                <div class="mx-auto max-w-7xl px-6 py-8 text-xs text-gray-500">
                    {brand.footer_note}
                </div>
            </footer>
        </div>
    }
    .into_any()
}
