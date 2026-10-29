use std::cell::RefCell;

use leptos::prelude::*;
use leptos_toaster::{Toast, ToastId, ToastVariant, Toaster, ToasterPosition, Toasts};

thread_local! {
    static TOASTER: RefCell<Option<Toasts>> = const { RefCell::new(None) };
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    view! {
        <Toaster position=ToasterPosition::BottomRight>
            <ToasterRegistration />
            {children()}
        </Toaster>
    }
    .into_any()
}

#[component]
fn ToasterRegistration() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    if let Some(toasts) = use_context::<Toasts>() {
        TOASTER.with(|cell| *cell.borrow_mut() = Some(toasts));
    }
}

pub fn notify_success(title: impl Into<String>) {
    notify(ToastVariant::Success, title.into(), None);
}

pub fn notify_failure(title: impl Into<String>, description: Option<String>) {
    notify(ToastVariant::Error, title.into(), description);
}

fn notify(variant: ToastVariant, title: String, description: Option<String>) {
    let Some(toaster) = TOASTER.with(|cell| *cell.borrow()) else {
        return;
    };
    let toast_id = ToastId::new();
    toaster.toast(
        move || {
            let title = title.clone();
            let description = description
                .clone()
                .map(|text| ViewFn::from(move || text.clone()));
            view! { <Toast toast_id variant=variant title=move || title.clone() description=description /> }
        },
        Some(toast_id),
        None,
    );
}
