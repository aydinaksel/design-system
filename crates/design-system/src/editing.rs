use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{Element, HtmlElement, Node};
use leptos_use_timeout_fn::use_timeout_fn;

const MINIMUM_SAVING_MILLISECONDS: f64 = 300.0;
const UNFINISHED_SAVE_MESSAGE: &str = "The save did not finish. Try again.";
pub(crate) const SAVED_HIGHLIGHT_MILLISECONDS: f64 = 1600.0;
pub(crate) const ATTENTION_MILLISECONDS: f64 = 1200.0;
pub(crate) const FIRST_CONTROL_SELECTOR: &str = "input:not([disabled]), textarea:not([disabled]), button[role=combobox]:not([disabled]), button[role=switch]:not([disabled])";
pub(crate) const FIRST_INVALID_CONTROL_SELECTOR: &str = "[aria-invalid=true]";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SaveOutcome {
    Succeeded,
    Failed(String),
    Unchanged,
    Invalid,
}

impl SaveOutcome {
    fn waits_for_minimum(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed(_))
    }
}

#[must_use = "the editor stays in its saving state until the save reports an outcome"]
pub struct FieldSave {
    outcome: RwSignal<Option<SaveOutcome>>,
    reported: bool,
}

impl FieldSave {
    fn new(outcome: RwSignal<Option<SaveOutcome>>) -> Self {
        Self {
            outcome,
            reported: false,
        }
    }

    fn report(mut self, result: SaveOutcome) {
        self.reported = true;
        self.outcome.try_set(Some(result));
    }

    pub fn succeeded(self) {
        self.report(SaveOutcome::Succeeded);
    }

    pub fn failed(self, message: impl Into<String>) {
        self.report(SaveOutcome::Failed(message.into()));
    }

    pub fn unchanged(self) {
        self.report(SaveOutcome::Unchanged);
    }

    pub fn invalid(self) {
        self.report(SaveOutcome::Invalid);
    }
}

impl Drop for FieldSave {
    fn drop(&mut self) {
        if !self.reported {
            self.outcome.try_set(Some(SaveOutcome::Failed(
                UNFINISHED_SAVE_MESSAGE.to_owned(),
            )));
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PendingSave {
    outcome: RwSignal<Option<SaveOutcome>>,
    minimum_elapsed: RwSignal<bool>,
    start_minimum_timer: Callback<()>,
}

impl PendingSave {
    pub(crate) fn new() -> Self {
        let minimum_elapsed = RwSignal::new(false);
        Self {
            outcome: RwSignal::new(None),
            minimum_elapsed,
            start_minimum_timer: timer(
                move || minimum_elapsed.set(true),
                MINIMUM_SAVING_MILLISECONDS,
            ),
        }
    }

    pub(crate) fn begin(self) -> FieldSave {
        self.outcome.set(None);
        self.minimum_elapsed.set(false);
        self.start_minimum_timer.run(());
        FieldSave::new(self.outcome)
    }

    pub(crate) fn resolve_with(
        self,
        is_saving: impl Fn() -> bool + 'static,
        resolve: impl Fn(SaveOutcome) + 'static,
    ) {
        Effect::new(move |_| {
            if !is_saving() {
                return;
            }
            let Some(result) = self.outcome.get() else {
                return;
            };
            if result.waits_for_minimum() && !self.minimum_elapsed.get() {
                return;
            }
            self.outcome.set(None);
            resolve(result);
        });
    }
}

pub(crate) fn timer(
    callback: impl Fn() + Clone + Send + Sync + 'static,
    milliseconds: f64,
) -> Callback<()> {
    let start = use_timeout_fn(move |()| callback(), milliseconds).start;
    Callback::new(move |()| start(()))
}

pub(crate) fn saves_on_enter(event: &KeyboardEvent) -> bool {
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

pub(crate) fn focus_first(container: &Element, selector: &str) {
    let control = container
        .query_selector(selector)
        .ok()
        .flatten()
        .and_then(|element| element.dyn_into::<HtmlElement>().ok());
    if let Some(control) = control {
        control.focus().unwrap_or_default();
    }
}

pub(crate) fn focus_is_within_or_lost(container: &Element) -> bool {
    document().active_element().is_none_or(|active| {
        let active: &Node = &active;
        active.node_name().eq_ignore_ascii_case("body") || container.contains(Some(active))
    })
}
