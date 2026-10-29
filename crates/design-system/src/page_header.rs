use leptos::prelude::*;

use crate::clipboard::CopyIconButton;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn PageHeader(
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] document_number: Option<Signal<String>>,
    #[prop(optional)] status: Option<AnyView>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="relative mb-4 flex flex-wrap items-center justify-between gap-3">
            <div class="flex flex-wrap items-center gap-2">
                <h1 class="inline-flex items-center gap-2 text-xl font-bold tracking-tight">
                    <span>{move || title.get()}</span>
                    {document_number
                        .map(|document_number| {
                            view! {
                                <span>{move || document_number.get()}</span>
                                <CopyIconButton value=document_number />
                            }
                        })}
                </h1>
                {status}
            </div>
            {children
                .map(|actions| view! { <div class="flex items-center gap-2">{actions()}</div> })}
        </div>
    }
    .into_any()
}
