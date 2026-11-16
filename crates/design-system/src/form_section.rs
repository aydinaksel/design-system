use leptos::prelude::*;

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::controls::FieldElementIds;
use crate::element_key::ElementKey;
use crate::fields::{ALIGNED_WITH_CONTROL_TEXT_CLASS, ROW_CLASS};
use crate::tooltip::HelpTerm;
use crate::typography::{heading_element, HeadingLevel, LABEL_CLASS, SECTION_HEADING_ROW_CLASS};

const CARD_CLASS: &str = "rounded-xl border border-edge bg-surface";
const FOOTER_CLASS: &str = "flex flex-wrap items-center justify-between gap-x-3 gap-y-2 rounded-b-[calc(var(--radius-xl)-1px)] border-t border-edge bg-gray-50 px-4 py-2";
const ERROR_CLASS: &str = "text-sm text-red-600";
const LABEL_COLUMN_CLASS: &str = "pt-1.5 text-sm text-gray-500 sm:w-48 sm:shrink-0";
const CONTROL_COLUMN_CLASS: &str = "flex min-w-0 flex-1 pb-1.5 pt-1 text-sm sm:justify-end sm:py-0";
const CONTROL_CLASS: &str = "min-w-0 max-w-sm flex-1 text-left";
const SUBMITTING_LABEL: &str = "Saving\u{2026}";
const CANCEL_LABEL: &str = "Cancel";
const FOOTER_BUTTON_SIZE: ButtonSize = ButtonSize::Compact;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FormSection(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] heading_level: HeadingLevel,
    #[prop(into)] submit_label: String,
    #[prop(into)] on_submit: Callback<()>,
    #[prop(optional, into)] submitting: Signal<bool>,
    #[prop(optional, into)] incomplete: Signal<bool>,
    #[prop(optional, into)] cancel_href: Option<String>,
    #[prop(optional, into)] error: Signal<Option<String>>,
    children: Children,
) -> impl IntoView {
    let section_number = ElementKey::next();
    let heading_id = title
        .as_ref()
        .map(|_| format!("form-section-{section_number}-heading"));
    let heading = title.zip(heading_id.clone()).map(|(title, id)| {
        view! {
            <div class=SECTION_HEADING_ROW_CLASS>
                {heading_element(heading_level, Some(id), LABEL_CLASS, title)}
            </div>
        }
    });
    let cancel = cancel_href.map(|href| {
        view! {
            <a href=href class=button_class(ButtonTone::Secondary, FOOTER_BUTTON_SIZE)>
                {CANCEL_LABEL}
            </a>
        }
    });
    let error_text = move || {
        error
            .get()
            .map(|message| view! { <p role="alert" class=ERROR_CLASS>{message}</p> })
    };
    let submit_text = move || {
        if submitting.get() {
            SUBMITTING_LABEL.to_owned()
        } else {
            submit_label.clone()
        }
    };
    view! {
        <section aria-labelledby=heading_id class="w-full max-w-2xl">
            {heading}
            <div class=CARD_CLASS>
                <div>{children()}</div>
                <div class=FOOTER_CLASS>
                    <div aria-live="polite">{error_text}</div>
                    <div class="ml-auto flex gap-2">
                        {cancel}
                        <button
                            type="button"
                            class=button_class(ButtonTone::Primary, FOOTER_BUTTON_SIZE)
                            disabled=move || incomplete.get() || submitting.get()
                            on:click=move |_| on_submit.run(())
                        >
                            {submit_text}
                        </button>
                    </div>
                </div>
            </div>
        </section>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FormRow(
    #[prop(into)] label: String,
    #[prop(optional, into)] help: Option<String>,
    children: Children,
) -> impl IntoView {
    let row_number = ElementKey::next();
    let control_id = format!("form-row-{row_number}");
    let label_id = format!("form-row-{row_number}-label");
    let help_id = help.as_ref().map(|_| format!("form-row-{row_number}-help"));
    provide_context(FieldElementIds {
        control: control_id.clone(),
        label: label_id.clone(),
        description: help_id.clone(),
        invalid: Signal::default(),
    });
    let label_text = match help.zip(help_id) {
        Some((help, help_id)) => view! {
            <HelpTerm help help_id>
                {label}
            </HelpTerm>
        }
        .into_any(),
        None => label.into_any(),
    };
    view! {
        <div class=ROW_CLASS>
            <label
                id=label_id
                for=control_id
                class=format!("{LABEL_COLUMN_CLASS} {ALIGNED_WITH_CONTROL_TEXT_CLASS}")
            >
                {label_text}
            </label>
            <div class=CONTROL_COLUMN_CLASS>
                <div class=CONTROL_CLASS>{children()}</div>
            </div>
        </div>
    }
    .into_any()
}
