use design_system::{
    button_class, notify_failure, notify_success, use_dialog_close, BusyOverlay, ButtonSize,
    ButtonTone, ChoiceList, ChoiceOption, ConfirmDialog, Dialog, DialogSize, Input,
};
use leptos::prelude::*;

pub(crate) fn toasts() -> AnyView {
    view! {
        <div class="flex flex-wrap items-center gap-3">
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=|_| notify_success("Sales Order 10482 Created")
            >
                "Show Success"
            </button>
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=|_| notify_failure("Could Not Reach the Server", None)
            >
                "Show Failure"
            </button>
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=|_| {
                    notify_failure(
                        "Failed Saving Classification",
                        Some("The Service Layer timed out after 30 seconds.".to_owned()),
                    );
                }
            >
                "Show Failure with Detail"
            </button>
        </div>
    }
    .into_any()
}

pub(crate) const PRIORITIES: [&str; 4] = ["Hot", "High", "Normal", "Low"];
const BUSY_DEMONSTRATION_MILLISECONDS: u64 = 2000;

fn gallery_button(label: &'static str, tone: ButtonTone, on_click: impl Fn() + 'static) -> AnyView {
    view! {
        <button
            type="button"
            class=button_class(tone, ButtonSize::Standard)
            on:click=move |_| on_click()
        >
            {label}
        </button>
    }
    .into_any()
}

#[component]
fn RenameForm() -> impl IntoView {
    let name = RwSignal::new(String::from("Ivory Honed Travertine"));
    let close = use_dialog_close();
    let finish = move || {
        if let Some(close) = close {
            close.run(());
        }
    };
    view! {
        <Input value=name attr:data-autofocus="" />
        <div class="mt-5 flex justify-end gap-2">
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=move |_| finish()
            >
                "Cancel"
            </button>
            <button
                type="button"
                class=button_class(ButtonTone::Primary, ButtonSize::Standard)
                on:click=move |_| {
                    notify_success(format!("Renamed to {}", name.get_untracked()));
                    finish();
                }
            >
                "Save"
            </button>
        </div>
    }
}

fn form_dialog(open: RwSignal<bool>) -> impl IntoView {
    move || {
        open.get().then(|| {
            view! {
                <Dialog
                    title="Rename Item"
                    description="The new name shows on documents created from now on."
                    on_close=Callback::new(move |()| open.set(false))
                >
                    <RenameForm />
                </Dialog>
            }
        })
    }
}

fn priority_dialog(open: RwSignal<bool>, priority: RwSignal<String>) -> impl IntoView {
    let options: Vec<ChoiceOption> = PRIORITIES
        .iter()
        .map(|priority| ChoiceOption::new(*priority, *priority))
        .collect();
    let options = StoredValue::new(options);
    move || {
        open.get().then(|| {
            view! {
                <Dialog
                    title="Change Priority for PR 1042"
                    size=DialogSize::Small
                    on_close=Callback::new(move |()| open.set(false))
                >
                    <ChoiceList
                        options=options.get_value()
                        current=priority
                        label="Priority"
                        autofocus=true
                        on_choose=Callback::new(move |chosen: String| {
                            priority.set(chosen);
                            open.set(false);
                        })
                    />
                </Dialog>
            }
        })
    }
}

fn history_dialog(open: RwSignal<bool>) -> impl IntoView {
    move || {
        open.get().then(|| {
            let rows = (1..=30_u32)
                .map(|day| {
                    view! {
                        <li class="py-2.5 text-sm text-gray-700">
                            {format!("2026-09-{day:02} \u{b7} Quantity changed")}
                        </li>
                    }
                })
                .collect::<Vec<_>>();
            view! {
                <Dialog
                    title="Sales Order 24817 History"
                    size=DialogSize::Large
                    on_close=Callback::new(move |()| open.set(false))
                >
                    <ul class="divide-y divide-divider">{rows}</ul>
                </Dialog>
            }
        })
    }
}

pub(crate) fn dialogs() -> AnyView {
    let form_open = RwSignal::new(false);
    let priority_open = RwSignal::new(false);
    let history_open = RwSignal::new(false);
    let confirm_open = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let priority = RwSignal::new(String::from("Normal"));
    let show_busy = move || {
        busy.set(true);
        set_timeout(
            move || busy.set(false),
            std::time::Duration::from_millis(BUSY_DEMONSTRATION_MILLISECONDS),
        );
    };
    view! {
        <div class="flex flex-wrap items-center gap-3">
            {gallery_button("Rename Item", ButtonTone::Secondary, move || form_open.set(true))}
            {gallery_button(
                "Change Priority",
                ButtonTone::Secondary,
                move || priority_open.set(true),
            )}
            {gallery_button("Show History", ButtonTone::Secondary, move || history_open.set(true))}
            {gallery_button("Delete Something", ButtonTone::Danger, move || confirm_open.set(true))}
            {gallery_button("Show Busy Overlay", ButtonTone::Secondary, show_busy)}
            <span class="text-sm text-gray-500">
                {move || format!("Priority: {}", priority.get())}
            </span>
        </div>
        {form_dialog(form_open)}
        {priority_dialog(priority_open, priority)}
        {history_dialog(history_open)}
        <ConfirmDialog
            open=confirm_open
            title="Delete this line?"
            body="It is removed from the document and cannot be restored."
            confirm_label="Delete"
            tone=ButtonTone::Danger
            on_confirm=Callback::new(move |()| notify_success("Line Deleted"))
        />
        <BusyOverlay visible=busy message="Generating Labels\u{2026}" />
    }
    .into_any()
}
