use leptos::portal::Portal;
use leptos::prelude::*;

use crate::icons;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn BusyOverlay(
    #[prop(into)] visible: Signal<bool>,
    #[prop(into)] message: Signal<String>,
) -> impl IntoView {
    move || {
        visible.get().then(|| {
            view! {
                <Portal>
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900/40 p-4 transition-opacity duration-200 ease-out starting:opacity-0 motion-reduce:transition-none">
                        <div
                            role="status"
                            aria-live="polite"
                            class="flex items-center gap-3 rounded-lg bg-white px-6 py-4 shadow-lg"
                        >
                            {icons::spinner("h-5 w-5 text-blue-600")}
                            <span class="text-sm font-medium text-gray-700">
                                {move || message.get()}
                            </span>
                        </div>
                    </div>
                </Portal>
            }
        })
    }
}
