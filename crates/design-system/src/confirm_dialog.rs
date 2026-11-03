use leptos::prelude::*;

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::dialog::{Dialog, DialogSize};

const DEFAULT_CANCEL_LABEL: &str = "Cancel";

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ConfirmDialog(
    open: RwSignal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] body: Signal<String>,
    #[prop(into)] confirm_label: Signal<String>,
    #[prop(optional, into)] cancel_label: Option<String>,
    #[prop(optional)] tone: ButtonTone,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    let cancel_label =
        StoredValue::new(cancel_label.unwrap_or_else(|| DEFAULT_CANCEL_LABEL.to_owned()));
    move || {
        open.get().then(|| {
            view! {
                <Dialog
                    title=title
                    description=body
                    size=DialogSize::Medium
                    alert=true
                    on_close=Callback::new(move |()| open.set(false))
                >
                    <div class="flex justify-end gap-2">
                        <button
                            type="button"
                            data-autofocus
                            class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                            on:click=move |_| open.set(false)
                        >
                            {cancel_label.get_value()}
                        </button>
                        <button
                            type="button"
                            class=button_class(tone, ButtonSize::Standard)
                            on:click=move |_| {
                                open.set(false);
                                on_confirm.run(());
                            }
                        >
                            {move || confirm_label.get()}
                        </button>
                    </div>
                </Dialog>
            }
        })
    }
}
