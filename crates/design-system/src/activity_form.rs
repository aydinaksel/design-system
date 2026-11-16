use leptos::prelude::*;

use crate::activity_feed::ActivityKind;
use crate::controls::Input;
use crate::date_picker::DatePicker;
use crate::dialog::DialogSize;
use crate::fieldset::Field;
use crate::form_dialog::FormDialog;
use crate::listbox::{Listbox, ListboxOption};
use crate::markdown_editor::MarkdownEditor;

const BODY_PLACEHOLDER: &str = "What happened, and what should the team know?";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityDraft {
    pub kind: ActivityKind,
    pub date: String,
    pub subject: String,
    pub body: String,
}

impl ActivityDraft {
    #[must_use]
    pub fn new(kind: ActivityKind, date: impl Into<String>) -> Self {
        Self {
            kind,
            date: date.into(),
            subject: String::new(),
            body: String::new(),
        }
    }
}

fn kind_options() -> Vec<ListboxOption> {
    ActivityKind::LOGGABLE
        .into_iter()
        .map(|kind| ListboxOption::new(kind.key(), kind.label()))
        .collect()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ActivityFormDialog(
    #[prop(into)] title: String,
    draft: ActivityDraft,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_save: Callback<ActivityDraft>,
    #[prop(optional, into)] saving: Signal<bool>,
    #[prop(optional, into)] error: Signal<Option<String>>,
) -> impl IntoView {
    let kind = RwSignal::new(draft.kind.key().to_owned());
    let date = RwSignal::new(draft.date);
    let subject = RwSignal::new(draft.subject);
    let body = RwSignal::new(draft.body);
    let complete = Signal::derive(move || {
        let has_subject = subject.with(|text| !text.trim().is_empty());
        let has_date = date.with(|text| !text.is_empty());
        let has_kind = kind.with(|key| ActivityKind::from_key(key).is_some());
        has_subject && has_date && has_kind
    });
    let submit = Callback::new(move |()| {
        let Some(chosen_kind) = ActivityKind::from_key(&kind.get_untracked()) else {
            return;
        };
        on_save.run(ActivityDraft {
            kind: chosen_kind,
            date: date.get_untracked(),
            subject: subject.get_untracked().trim().to_owned(),
            body: body.get_untracked().trim().to_owned(),
        });
    });
    view! {
        <FormDialog
            title=title
            on_close=on_close
            on_submit=submit
            saving=saving
            error=error
            can_submit=complete
            size=DialogSize::Large
        >
            <div class="grid gap-6 sm:grid-cols-[1fr_auto]">
                <Field label="Type">
                    <Listbox options=Signal::stored(kind_options()) selected=kind />
                </Field>
                <Field label="Date">
                    <DatePicker value=date />
                </Field>
            </div>
            <Field label="Subject">
                <Input value=subject autofocus=true />
            </Field>
            <Field label="Body">
                <MarkdownEditor value=body placeholder=BODY_PLACEHOLDER />
            </Field>
        </FormDialog>
    }
    .into_any()
}
