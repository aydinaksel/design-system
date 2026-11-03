use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html::Div;
use leptos::prelude::*;

use crate::alert::Alert;
use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::controls::{FieldElementIds, Input};
use crate::editing::{
    focus_first, saves_on_enter, timer, FieldSave, PendingSave, SaveOutcome,
    ATTENTION_MILLISECONDS, FIRST_CONTROL_SELECTOR, FIRST_INVALID_CONTROL_SELECTOR,
    SAVED_HIGHLIGHT_MILLISECONDS,
};
use crate::popover::{
    Dismissal, Popover, PopoverContent, PopoverHeader, PopoverState, PopoverTitle, PopoverTrigger,
};
use crate::tone::Tone;

static NEXT_CELL_EDITOR_NUMBER: AtomicU64 = AtomicU64::new(0);

const TRIGGER_CLASS: &str = "group -mx-1.5 -my-0.5 inline-flex max-w-[calc(100%+0.75rem)] cursor-pointer items-center rounded-md px-1.5 py-0.5 text-left focus-ring";
const VALUE_CLASS: &str = "truncate border-b border-dotted border-gray-400 group-hover:border-solid group-hover:border-blue-600 group-hover:text-blue-700";
const EMPTY_VALUE_CLASS: &str = "text-gray-400 group-hover:text-blue-700";
const EMPTY_VALUE_LABEL: &str = "+ Add";
const CONTENT_WIDTH_CLASS: &str = "w-72";
const ATTENTION_CLASS: &str = "ring-2 ring-focus ring-offset-2";

type Display<Value> = Arc<dyn Fn(&Value) -> String + Send + Sync>;
type EditorView<Value> = Arc<dyn Fn(RwSignal<Value>) -> AnyView + Send + Sync>;
type Normalize<Value> = Arc<dyn Fn(&Value) -> Value + Send + Sync>;
type Validate<Value> = Arc<dyn Fn(&Value) -> Option<String> + Send + Sync>;
type Save<Value> = Arc<dyn Fn(Value, FieldSave) + Send + Sync>;

pub struct CellEditor<Value> {
    label: String,
    saved: Value,
    display: Display<Value>,
    editor: EditorView<Value>,
    normalize: Normalize<Value>,
    validate: Validate<Value>,
    hint: Option<EditorView<Value>>,
    on_save: Save<Value>,
}

impl CellEditor<String> {
    #[must_use]
    pub fn text(
        label: impl Into<String>,
        saved: impl Into<String>,
        on_save: impl Fn(String, FieldSave) + Send + Sync + 'static,
    ) -> Self {
        Self::new(
            label,
            saved.into(),
            String::clone,
            |draft| view! { <Input value=draft /> }.into_any(),
            on_save,
        )
    }
}

impl<Value> CellEditor<Value>
where
    Value: Clone + PartialEq + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        saved: Value,
        display: impl Fn(&Value) -> String + Send + Sync + 'static,
        editor: impl Fn(RwSignal<Value>) -> AnyView + Send + Sync + 'static,
        on_save: impl Fn(Value, FieldSave) + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            saved,
            display: Arc::new(display),
            editor: Arc::new(editor),
            normalize: Arc::new(Value::clone),
            validate: Arc::new(|_| None),
            hint: None,
            on_save: Arc::new(on_save),
        }
    }

    #[must_use]
    pub fn normalize(self, normalize: impl Fn(&Value) -> Value + Send + Sync + 'static) -> Self {
        Self {
            normalize: Arc::new(normalize),
            ..self
        }
    }

    #[must_use]
    pub fn validate(
        self,
        validate: impl Fn(&Value) -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            validate: Arc::new(validate),
            ..self
        }
    }

    #[must_use]
    pub fn hint(self, hint: impl Fn(RwSignal<Value>) -> AnyView + Send + Sync + 'static) -> Self {
        Self {
            hint: Some(Arc::new(hint)),
            ..self
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one widget: the trigger, the panel and the save share the draft"
    )]
    #[must_use]
    pub fn into_view(self) -> AnyView {
        let Self {
            label,
            saved,
            display,
            editor,
            normalize,
            validate,
            hint,
            on_save,
        } = self;
        let ids = CellIds::new(NEXT_CELL_EDITOR_NUMBER.fetch_add(1, Ordering::Relaxed));
        let has_hint = hint.is_some();
        let display = StoredValue::new(display);
        let editor = StoredValue::new(editor);
        let normalize = StoredValue::new(normalize);
        let validate = StoredValue::new(validate);
        let hint = StoredValue::new(hint);
        let on_save = StoredValue::new(on_save);
        let label = StoredValue::new(label);

        let popover = PopoverState::new();
        let displayed = RwSignal::new(saved.clone());
        let draft = RwSignal::new(saved);
        let saving = RwSignal::new(false);
        let errors_revealed = RwSignal::new(false);
        let failure: RwSignal<Option<String>> = RwSignal::new(None);
        let flashing = RwSignal::new(false);
        let attention = RwSignal::new(false);
        let content: NodeRef<Div> = NodeRef::new();
        let pending = PendingSave::new();
        let start_flash_timer = timer(move || flashing.set(false), SAVED_HIGHLIGHT_MILLISECONDS);
        let start_attention_timer = timer(move || attention.set(false), ATTENTION_MILLISECONDS);

        let normalized = Signal::derive(move || {
            draft.with(|value| normalize.with_value(|normalize| normalize(value)))
        });
        let changed =
            Signal::derive(move || normalized.with(|value| displayed.with(|shown| value != shown)));
        let error = Signal::derive(move || {
            normalized.with(|value| validate.with_value(|validate| validate(value)))
        });
        let visible_error = Signal::derive(move || {
            if errors_revealed.get() {
                error.get()
            } else {
                None
            }
        });

        let focus_content = move |selector: &'static str| {
            request_animation_frame(move || {
                if let Some(element) = content.get_untracked() {
                    focus_first(&element, selector);
                }
            });
        };
        let begin = Callback::new(move |()| {
            draft.set(displayed.get_untracked());
            errors_revealed.set(false);
            failure.set(None);
        });
        let before_dismiss = Callback::new(move |reason: Dismissal| {
            if saving.get_untracked() {
                return false;
            }
            if reason == Dismissal::Escape || !changed.get_untracked() {
                return true;
            }
            attention.set(true);
            start_attention_timer.run(());
            focus_content(FIRST_CONTROL_SELECTOR);
            false
        });
        let save = move || {
            if saving.get_untracked() {
                return;
            }
            errors_revealed.set(true);
            failure.set(None);
            if error.with_untracked(Option::is_some) {
                focus_content(FIRST_INVALID_CONTROL_SELECTOR);
                return;
            }
            if !changed.get_untracked() {
                popover.close();
                return;
            }
            saving.set(true);
            let value = normalized.get_untracked();
            on_save.with_value(|on_save| on_save(value, pending.begin()));
        };
        let cancel = move || {
            if !saving.get_untracked() {
                popover.close();
            }
        };

        pending.resolve_with(
            move || saving.get(),
            move |result| {
                saving.set(false);
                match result {
                    SaveOutcome::Succeeded => {
                        displayed.set(normalized.get_untracked());
                        popover.close();
                        flashing.set(true);
                        start_flash_timer.run(());
                    }
                    SaveOutcome::Unchanged => popover.close(),
                    SaveOutcome::Invalid => focus_content(FIRST_INVALID_CONTROL_SELECTOR),
                    SaveOutcome::Failed(message) => failure.set(Some(message)),
                }
            },
        );

        let on_keydown = move |event: KeyboardEvent| {
            if event.key() == "Enter" && !event.default_prevented() && saves_on_enter(&event) {
                event.prevent_default();
                save();
            }
        };
        let displayed_text =
            move || displayed.with(|value| display.with_value(|display| display(value)));
        let trigger_label = move || {
            let text = displayed_text();
            label.with_value(|label| accessible_name(&text, label))
        };
        let trigger_class = Signal::derive(move || {
            format!(
                "{TRIGGER_CLASS} {}",
                if flashing.get() {
                    "motion-safe:animate-field-saved"
                } else {
                    ""
                },
            )
        });
        let trigger_content = move || {
            let text = displayed_text();
            if text.trim().is_empty() {
                view! { <span class=EMPTY_VALUE_CLASS>{EMPTY_VALUE_LABEL}</span> }.into_any()
            } else {
                view! { <span class=VALUE_CLASS>{text}</span> }.into_any()
            }
        };
        let content_class = Signal::derive(move || {
            format!(
                "{CONTENT_WIDTH_CLASS} {}",
                if attention.get() { ATTENTION_CLASS } else { "" },
            )
        });
        let description = move || match visible_error.get() {
            Some(message) => message.into_any(),
            None => hint
                .with_value(|hint| hint.as_ref().map(|hint| hint(draft)))
                .unwrap_or_else(|| ().into_any()),
        };
        let description_class = move || {
            if visible_error.with(Option::is_some) {
                "mt-1.5 text-xs text-red-600"
            } else if has_hint {
                "mt-1.5 text-xs text-gray-500"
            } else {
                "hidden"
            }
        };
        let field_ids = StoredValue::new(FieldElementIds {
            control: ids.control.clone(),
            label: ids.control.clone(),
            description: Some(ids.description.clone()),
            invalid: Signal::derive(move || visible_error.with(Option::is_some)),
        });
        let ids = StoredValue::new(ids);

        view! {
            <Popover state=popover on_open=begin before_dismiss=before_dismiss>
                <PopoverTrigger class=trigger_class attr:aria-label=trigger_label>
                    {trigger_content}
                </PopoverTrigger>
                <PopoverContent class=content_class>
                    <div node_ref=content on:keydown=on_keydown>
                        <PopoverHeader>
                            <PopoverTitle control=ids
                                .with_value(|ids| {
                                    ids.control.clone()
                                })>{label.get_value()}</PopoverTitle>
                        </PopoverHeader>
                        <div inert=move || saving.get().then_some("")>
                            <Provider value=field_ids
                                .get_value()>{editor.with_value(|editor| editor(draft))}</Provider>
                        </div>
                        <p id=ids.with_value(|ids| ids.description.clone()) class=description_class>
                            {description}
                        </p>
                        {move || {
                            failure
                                .get()
                                .map(|message| {
                                    view! {
                                        <div class="mt-3">
                                            <Alert tone=Tone::Critical>{message}</Alert>
                                        </div>
                                    }
                                })
                        }}
                        <div class="mt-3 flex justify-end gap-2">
                            <button
                                type="button"
                                class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                                aria-disabled=move || saving.get().to_string()
                                on:click=move |_| cancel()
                            >
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class=button_class(ButtonTone::Primary, ButtonSize::Compact)
                                aria-disabled=move || saving.get().to_string()
                                on:click=move |_| save()
                            >
                                {move || if saving.get() { "Saving\u{2026}" } else { "Save" }}
                            </button>
                        </div>
                    </div>
                </PopoverContent>
            </Popover>
        }
        .into_any()
    }
}

#[derive(Clone)]
struct CellIds {
    control: String,
    description: String,
}

impl CellIds {
    fn new(number: u64) -> Self {
        Self {
            control: format!("cell-{number}"),
            description: format!("cell-{number}-description"),
        }
    }
}

fn accessible_name(text: &str, label: &str) -> String {
    if text.trim().is_empty() {
        format!("Add {label}")
    } else {
        format!("{text}, Edit {label}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_filled_cell_by_its_value_first() {
        assert_eq!(
            accessible_name("CSQU3054383", "Container"),
            "CSQU3054383, Edit Container"
        );
    }

    #[test]
    fn names_an_empty_cell_as_an_invitation() {
        assert_eq!(accessible_name("  ", "Container"), "Add Container");
    }
}
