use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::dialog::{Dialog, DialogSize};

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FormDialog(
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] description: Option<Signal<String>>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
    #[prop(into)] saving: Signal<bool>,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(optional, into)] can_submit: MaybeProp<bool>,
    #[prop(default = "Save".into(), into)] submit_label: Signal<String>,
    #[prop(optional)] size: DialogSize,
    children: Children,
) -> impl IntoView {
    let close_unless_saving = Callback::new(move |()| {
        if !saving.get_untracked() {
            on_close.run(());
        }
    });
    let submit = move |event: SubmitEvent| {
        event.prevent_default();
        if !saving.get_untracked() && can_submit.get().unwrap_or(true) {
            on_submit.run(());
        }
    };
    let error_view = move || {
        error.get().map(|message| {
            view! {
                <p
                    role="alert"
                    class="mt-4 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700"
                >
                    {message}
                </p>
            }
        })
    };
    let content = view! {
        <form on:submit=submit novalidate>
            <fieldset prop:disabled=move || saving.get() class="min-w-0 space-y-6">
                {children()}
            </fieldset>
            {error_view}
            <div class="mt-6 flex justify-end gap-2">
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                    prop:disabled=move || saving.get()
                    on:click=move |_| close_unless_saving.run(())
                >
                    "Cancel"
                </button>
                <button
                    type="submit"
                    class=button_class(ButtonTone::Primary, ButtonSize::Standard)
                    prop:disabled=move || saving.get() || !can_submit.get().unwrap_or(true)
                >
                    {move || {
                        if saving.get() { "Saving\u{2026}".to_owned() } else { submit_label.get() }
                    }}
                </button>
            </div>
        </form>
    }
    .into_any();
    match description {
        Some(description) => view! {
            <Dialog title=title description=description on_close=close_unless_saving size=size>
                {content}
            </Dialog>
        }
        .into_any(),
        None => view! {
            <Dialog title=title on_close=close_unless_saving size=size>
                {content}
            </Dialog>
        }
        .into_any(),
    }
}
