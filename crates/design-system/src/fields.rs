use std::sync::atomic::{AtomicU64, Ordering};

use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html::{Div, Section};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{Element, HtmlElement};
use leptos_use::use_timeout_fn;

use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::controls::FieldElementIds;
use crate::field_actions::{field_actions_view, FieldAction};
use crate::icons;
use crate::toast::notify_success;
use crate::typography::LABEL_CLASS;

static NEXT_FIELD_NUMBER: AtomicU64 = AtomicU64::new(0);

const MINIMUM_SAVING_MILLISECONDS: f64 = 300.0;
const SAVED_HIGHLIGHT_MILLISECONDS: f64 = 1600.0;
const FIRST_CONTROL_SELECTOR: &str = "input:not([disabled]), textarea:not([disabled]), button[role=combobox]:not([disabled]), button[role=switch]:not([disabled])";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SectionState {
    #[default]
    Viewing,
    Editing,
    Saving,
    Saved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SaveOutcome {
    Succeeded,
    Failed(String),
    Unchanged,
}

#[derive(Clone, Copy)]
pub struct FieldSave {
    outcome: RwSignal<Option<SaveOutcome>>,
}

impl FieldSave {
    pub fn succeeded(self) {
        self.outcome.set(Some(SaveOutcome::Succeeded));
    }

    pub fn failed(self, message: impl Into<String>) {
        self.outcome.set(Some(SaveOutcome::Failed(message.into())));
    }

    pub fn unchanged(self) {
        self.outcome.set(Some(SaveOutcome::Unchanged));
    }
}

pub struct SectionEditing {
    pub can_edit: bool,
    pub is_editing: RwSignal<bool>,
    pub on_edit: Callback<()>,
    pub on_cancel: Callback<()>,
    pub on_save: Callback<FieldSave>,
}

#[derive(Clone, Copy)]
struct SectionContext {
    state: Signal<SectionState>,
}

#[derive(Clone, Copy)]
struct ActiveSection(RwSignal<Option<u64>>);

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FieldGrid(children: Children) -> impl IntoView {
    if use_context::<ActiveSection>().is_none() {
        provide_context(ActiveSection(RwSignal::new(None)));
    }
    view! { <div class="grid grid-cols-1 items-start gap-6 xl:grid-cols-2">{children()}</div> }
        .into_any()
}

fn saves_on_enter(event: &KeyboardEvent) -> bool {
    let Some(element) = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
    else {
        return false;
    };
    let tag = element.tag_name().to_ascii_lowercase();
    let is_select = element.get_attribute("role").as_deref() == Some("combobox");
    tag == "input" || (tag == "button" && is_select)
}

fn saved_message(title: Option<&str>) -> String {
    title.map_or_else(
        || "Changes Saved".to_owned(),
        |title| format!("{title} Saved"),
    )
}

fn focus_first_control(container: &HtmlElement) {
    let control = container
        .query_selector(FIRST_CONTROL_SELECTOR)
        .ok()
        .flatten()
        .and_then(|element| element.dyn_into::<HtmlElement>().ok());
    if let Some(control) = control {
        control.focus().unwrap_or_default();
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one widget: the header, the rows, and the saving overlay share the edit state"
)]
#[component]
pub fn FieldSection(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] actions: Option<AnyView>,
    #[prop(optional)] editing: Option<SectionEditing>,
    children: Children,
) -> impl IntoView {
    let section_number = NEXT_FIELD_NUMBER.fetch_add(1, Ordering::Relaxed);
    let active_section = use_context::<ActiveSection>();
    let state = RwSignal::new(SectionState::Viewing);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let outcome: RwSignal<Option<SaveOutcome>> = RwSignal::new(None);
    let minimum_elapsed = RwSignal::new(false);
    let section: NodeRef<Section> = NodeRef::new();
    let section_context = SectionContext {
        state: state.into(),
    };

    let release = move || {
        if let Some(ActiveSection(active)) = active_section {
            if active.get_untracked() == Some(section_number) {
                active.set(None);
            }
        }
    };
    let blocked_by_another = move || {
        active_section.is_some_and(|ActiveSection(active)| {
            active
                .get()
                .is_some_and(|editing_number| editing_number != section_number)
        })
    };

    let finish_minimum = use_timeout_fn(
        move |()| minimum_elapsed.set(true),
        MINIMUM_SAVING_MILLISECONDS,
    )
    .start;
    let saved_title = title.clone();
    let finish_highlight = use_timeout_fn(
        move |()| {
            if state.get_untracked() == SectionState::Saved {
                state.set(SectionState::Viewing);
            }
        },
        SAVED_HIGHLIGHT_MILLISECONDS,
    )
    .start;

    Effect::new(move |_| {
        if state.get() != SectionState::Saving {
            return;
        }
        let Some(result) = outcome.get() else {
            return;
        };
        if result != SaveOutcome::Unchanged && !minimum_elapsed.get() {
            return;
        }
        outcome.set(None);
        match result {
            SaveOutcome::Succeeded => {
                state.set(SectionState::Saved);
                release();
                finish_highlight(());
                notify_success(saved_message(saved_title.as_deref()));
            }
            SaveOutcome::Unchanged => {
                state.set(SectionState::Viewing);
                release();
            }
            SaveOutcome::Failed(message) => {
                error.set(Some(message));
                state.set(SectionState::Editing);
            }
        }
    });

    if let Some(is_editing) = editing.as_ref().map(|editing| editing.is_editing) {
        Effect::new(move |_| {
            let editing_now = matches!(state.get(), SectionState::Editing | SectionState::Saving);
            if is_editing.get_untracked() != editing_now {
                is_editing.set(editing_now);
            }
        });
    }

    let editing = editing.filter(|editing| editing.can_edit).map(|editing| {
        let SectionEditing {
            on_edit,
            on_cancel,
            on_save,
            ..
        } = editing;
        let start = move || {
            if let Some(ActiveSection(active)) = active_section {
                active.set(Some(section_number));
            }
            on_edit.run(());
            error.set(None);
            state.set(SectionState::Editing);
            request_animation_frame(move || {
                if let Some(element) = section.get_untracked() {
                    focus_first_control(&element);
                }
            });
        };
        let cancel = move || {
            on_cancel.run(());
            error.set(None);
            state.set(SectionState::Viewing);
            release();
        };
        let save = move || {
            if state.get_untracked() != SectionState::Editing {
                return;
            }
            error.set(None);
            outcome.set(None);
            minimum_elapsed.set(false);
            state.set(SectionState::Saving);
            finish_minimum(());
            on_save.run(FieldSave { outcome });
        };
        (start, cancel, save)
    });

    let keyboard_editing = editing.clone();
    let on_keydown = move |event: KeyboardEvent| {
        let Some((_, cancel, save)) = keyboard_editing.clone() else {
            return;
        };
        if state.get_untracked() != SectionState::Editing {
            return;
        }
        match event.key().as_str() {
            "Enter" if saves_on_enter(&event) => {
                event.prevent_default();
                save();
            }
            "Escape" => {
                event.prevent_default();
                cancel();
            }
            _ => {}
        }
    };

    let edit_controls = editing.map(|(start, cancel, save)| {
        let start = StoredValue::new(start);
        let cancel = StoredValue::new(cancel);
        let save = StoredValue::new(save);
        view! {
            {move || match state.get() {
                SectionState::Viewing | SectionState::Saved => {
                    view! {
                        <button
                            type="button"
                            class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                            prop:disabled=blocked_by_another
                            on:click=move |_| start.with_value(|start| start())
                        >
                            "Edit"
                        </button>
                    }
                        .into_any()
                }
                SectionState::Editing | SectionState::Saving => {
                    let busy = move || state.get() == SectionState::Saving;
                    view! {
                        <button
                            type="button"
                            class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                            prop:disabled=busy
                            on:click=move |_| cancel.with_value(|cancel| cancel())
                        >
                            "Cancel"
                        </button>
                        <button
                            type="button"
                            class=button_class(ButtonTone::Primary, ButtonSize::Compact)
                            prop:disabled=busy
                            on:click=move |_| save.with_value(|save| save())
                        >
                            "Save"
                        </button>
                    }
                        .into_any()
                }
            }}
        }
        .into_any()
    });

    let has_controls = actions.is_some() || edit_controls.is_some();
    let heading = (title.is_some() || has_controls).then(|| {
        view! {
            <div class="mb-2 flex min-h-8 items-center justify-between gap-3">
                <h2 class="text-sm font-semibold text-gray-900">{title}</h2>
                <div class="flex gap-2">{actions} {edit_controls}</div>
            </div>
        }
    });
    let saving = move || state.get() == SectionState::Saving;

    view! {
        <section node_ref=section on:keydown=on_keydown>
            {heading}
            {move || {
                error
                    .get()
                    .map(|message| {
                        view! {
                            <p
                                role="alert"
                                class="mb-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700"
                            >
                                {message}
                            </p>
                        }
                    })
            }}
            <div class="relative">
                <dl
                    aria-busy=move || saving().to_string()
                    class=move || {
                        format!(
                            "divide-y divide-gray-100 rounded-lg border border-gray-200 bg-white motion-safe:transition-opacity motion-safe:duration-200 {}",
                            if saving() { "opacity-40" } else { "opacity-100" },
                        )
                    }
                >
                    <Provider value=section_context>{children()}</Provider>
                </dl>
                <div
                    role="status"
                    aria-live="polite"
                    class=move || {
                        format!(
                            "absolute inset-0 flex items-center justify-center gap-2 text-sm font-medium text-gray-700 motion-safe:transition-opacity motion-safe:duration-200 {}",
                            if saving() { "opacity-100" } else { "pointer-events-none opacity-0" },
                        )
                    }
                >
                    {move || {
                        saving()
                            .then(|| {
                                view! {
                                    {icons::spinner("h-5 w-5 text-blue-600")}
                                    "Saving\u{2026}"
                                }
                            })
                    }}
                </div>
            </div>
        </section>
    }
    .into_any()
}

fn revert_button(
    label: String,
    on_revert: Callback<()>,
    unsaved: Signal<bool>,
    row: NodeRef<Div>,
) -> AnyView {
    let revert = move |_| {
        on_revert.run(());
        if let Some(element) = row.get_untracked() {
            focus_first_control(&element);
        }
    };
    view! {
        <button
            type="button"
            class=move || {
                format!(
                    "{} {}",
                    icon_button_class(ButtonTone::Ghost, ButtonSize::Compact),
                    if unsaved.get() { "" } else { "invisible" },
                )
            }
            aria-label=label.clone()
            title=label
            on:click=revert
        >
            {icons::icon(icons::LucideGlyph::RotateCcw, "h-4 w-4")}
        </button>
    }
    .into_any()
}

#[component]
pub fn FieldRow(
    #[prop(into)] label: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] editor: Option<ViewFn>,
    #[prop(optional, into)] changed: Signal<bool>,
    #[prop(optional, into)] on_revert: Option<Callback<()>>,
    #[prop(optional)] actions: Vec<FieldAction>,
    children: Children,
) -> impl IntoView {
    let row: NodeRef<Div> = NodeRef::new();
    let actions_view = field_actions_view(actions);
    let revert_label = format!("Revert {label}");
    let field_number = NEXT_FIELD_NUMBER.fetch_add(1, Ordering::Relaxed);
    let control_id = format!("field-{field_number}");
    let label_id = format!("field-{field_number}-label");
    let description_id = description
        .as_ref()
        .map(|_| format!("field-{field_number}-description"));
    provide_context(FieldElementIds {
        control: control_id.clone(),
        label: label_id.clone(),
        description: description_id.clone(),
    });

    let state = use_context::<SectionContext>().map_or_else(
        || Signal::stored(SectionState::Viewing),
        |context| context.state,
    );
    let has_editor = editor.is_some();
    let showing_editor = move || has_editor && state.get() == SectionState::Editing;
    let changed_when_saved = RwSignal::new(false);
    Effect::new(move |_| {
        if state.get() == SectionState::Saving {
            changed_when_saved.set(changed.get_untracked());
        }
    });
    let highlighted = move || state.get() == SectionState::Saved && changed_when_saved.get();
    let unsaved = move || showing_editor() && changed.get();
    let row_highlight = move || {
        if unsaved() {
            "bg-amber-50"
        } else if highlighted() {
            "motion-safe:animate-field-saved"
        } else {
            ""
        }
    };
    let revert_button = on_revert
        .map(|on_revert| revert_button(revert_label, on_revert, Signal::derive(unsaved), row));

    view! {
        <div
            node_ref=row
            class=move || {
                format!(
                    "flex flex-col gap-0.5 px-4 py-3 first:rounded-t-[7px] last:rounded-b-[7px] motion-safe:transition-colors sm:flex-row sm:items-center sm:gap-0 {}",
                    row_highlight(),
                )
            }
        >
            <dt class=format!("{LABEL_CLASS} sm:w-48 sm:shrink-0 sm:pr-4")>
                <label id=label_id for=control_id>
                    {label}
                </label>
            </dt>
            <dd class="min-w-0 flex-1 text-sm text-gray-900">
                <div class=move || {
                    if showing_editor() { "hidden" } else { "flex items-center gap-3" }
                }>
                    <div class="min-w-0 flex-1">{children()}</div>
                    {actions_view}
                </div>
                {editor
                    .map(|editor| {
                        view! {
                            <div class=move || {
                                if showing_editor() { "flex items-center gap-2" } else { "hidden" }
                            }>
                                <div class="min-w-0 max-w-sm flex-1">{editor.run()}</div>
                                {revert_button}
                            </div>
                        }
                    })}
                {description
                    .map(|description| {
                        view! {
                            <p id=description_id class="mt-1 text-xs text-gray-500">
                                {description}
                            </p>
                        }
                    })}
            </dd>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_saved_section() {
        assert_eq!(
            saved_message(Some("Classification")),
            "Classification Saved"
        );
    }

    #[test]
    fn falls_back_when_the_section_has_no_title() {
        assert_eq!(saved_message(None), "Changes Saved");
    }
}
