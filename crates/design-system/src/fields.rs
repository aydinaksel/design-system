use std::fmt::Display;

use leptos::context::Provider;
use leptos::ev::{BeforeUnloadEvent, KeyboardEvent, MouseEvent};
use leptos::html::{Button, Div, Section};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{Element, HtmlAnchorElement, ScrollIntoViewOptions, ScrollLogicalPosition};
use leptos_use_event_listener::{
    use_event_listener, use_event_listener_with_options, UseEventListenerOptions,
};
use leptos_use_window::use_window;
use send_wrapper::SendWrapper;

use crate::alert::Alert;
use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::confirm_dialog::ConfirmDialog;
use crate::controls::FieldElementIds;
use crate::editing::{
    focus_first, focus_is_within_or_lost, saves_on_enter, timer, FieldSave, PendingSave,
    SaveOutcome, ATTENTION_MILLISECONDS, FIRST_CONTROL_SELECTOR, FIRST_INVALID_CONTROL_SELECTOR,
    SAVED_HIGHLIGHT_MILLISECONDS,
};
use crate::element_key::ElementKey;
use crate::field_actions::{field_actions_view, FieldAction};
use crate::icons;
use crate::loading_state::LoadingState;
use crate::tone::Tone;
use crate::tooltip::{HelpTerm, Tooltip};
use crate::typography::LABEL_CLASS;

type PendingNavigation = StoredValue<Option<SendWrapper<HtmlAnchorElement>>>;

const ALIGNED_WITH_CONTROL_TEXT_CLASS: &str = "sm:py-[0.5625rem]";
const ROW_CLASS: &str = "relative flex flex-col px-4 py-1.5 first:rounded-t-surface-inner after:pointer-events-none after:absolute after:inset-x-4 after:bottom-0 after:h-px after:bg-gray-100 last:after:hidden sm:flex-row sm:items-center sm:gap-4 motion-safe:transition-colors";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeadingLevel {
    #[default]
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SectionState {
    #[default]
    Viewing,
    Editing,
    Saving,
    Saved,
}

impl SectionState {
    fn is_open(self) -> bool {
        matches!(self, Self::Editing | Self::Saving)
    }
}

pub struct SectionEditing {
    allowed: bool,
    on_save: Callback<FieldSave>,
    on_editing_change: Option<Callback<bool>>,
}

impl SectionEditing {
    #[must_use]
    pub fn new(on_save: impl Fn(FieldSave) + Send + Sync + 'static) -> Self {
        Self {
            allowed: true,
            on_save: Callback::new(on_save),
            on_editing_change: None,
        }
    }

    #[must_use]
    pub fn allowed(self, allowed: bool) -> Self {
        Self { allowed, ..self }
    }

    #[must_use]
    pub fn action<Input, Output, Error>(
        action: Action<Input, Result<Output, Error>>,
        request: impl Fn() -> Result<Input, SaveRejection> + Send + Sync + 'static,
    ) -> Self
    where
        Input: Send + Sync + 'static,
        Output: Send + Sync + 'static,
        Error: Display + Send + Sync + 'static,
    {
        let pending_save = StoredValue::new(None::<FieldSave>);
        Effect::new(move |_| {
            let Some(failure) = action
                .value()
                .with(|result| result.as_ref().map(failure_message))
            else {
                return;
            };
            let Some(save) = pending_save.try_update_value(Option::take).flatten() else {
                return;
            };
            match failure {
                None => save.succeeded(),
                Some(message) => save.failed(message),
            }
        });
        Self::new(move |save: FieldSave| match request() {
            Ok(input) => {
                pending_save.set_value(Some(save));
                action.dispatch(input);
            }
            Err(SaveRejection::Failed(message)) => save.failed(message),
            Err(SaveRejection::Invalid) => save.invalid(),
        })
    }

    #[must_use]
    pub fn on_editing_change(self, on_change: impl Fn(bool) + Send + Sync + 'static) -> Self {
        Self {
            on_editing_change: Some(Callback::new(on_change)),
            ..self
        }
    }
}

fn failure_message<Output, Error: Display>(result: &Result<Output, Error>) -> Option<String> {
    result.as_ref().err().map(ToString::to_string)
}

pub enum SaveRejection {
    Failed(String),
    Invalid,
}

impl From<&str> for SaveRejection {
    fn from(message: &str) -> Self {
        Self::Failed(message.to_owned())
    }
}

pub struct FieldEditor {
    view: ViewFn,
    changed: Signal<bool>,
    revert: Callback<()>,
    error: Signal<Option<String>>,
}

impl FieldEditor {
    #[must_use]
    pub fn new<Value>(
        draft: RwSignal<Value>,
        saved: impl Into<Signal<Value>>,
        view: impl Into<ViewFn>,
    ) -> Self
    where
        Value: Clone + PartialEq + Send + Sync + 'static,
    {
        let saved = saved.into();
        Self {
            view: view.into(),
            changed: Signal::derive(move || draft.with(|draft| saved.with(|saved| draft != saved))),
            revert: Callback::new(move |()| draft.set(saved.get_untracked())),
            error: Signal::default(),
        }
    }

    #[must_use]
    pub fn tracked(
        changed: impl Into<Signal<bool>>,
        revert: impl Fn() + Send + Sync + 'static,
        view: impl Into<ViewFn>,
    ) -> Self {
        Self {
            view: view.into(),
            changed: changed.into(),
            revert: Callback::new(move |()| revert()),
            error: Signal::default(),
        }
    }

    #[must_use]
    pub fn error(self, error: impl Fn() -> Option<String> + Send + Sync + 'static) -> Self {
        Self {
            error: Signal::derive(error),
            ..self
        }
    }
}

#[derive(Clone, Copy)]
struct RowStatus {
    number: ElementKey,
    changed: Signal<bool>,
    invalid: Signal<bool>,
    revert: Callback<()>,
}

#[derive(Clone, Copy)]
struct SectionContext {
    state: Signal<SectionState>,
    errors_revealed: Signal<bool>,
    rows: RwSignal<Vec<RowStatus>>,
}

#[derive(Clone, Copy)]
struct ActiveEditor {
    section_number: ElementKey,
    has_changes: Signal<bool>,
    discard: Callback<()>,
    draw_attention: Callback<()>,
}

#[derive(Clone, Copy)]
struct EditingScope(RwSignal<Option<ActiveEditor>>);

pub fn provide_editing_scope() {
    if use_context::<EditingScope>().is_none() {
        provide_context(EditingScope(RwSignal::new(None)));
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FieldGrid(children: Children) -> impl IntoView {
    provide_editing_scope();
    view! { <div class="grid grid-cols-1 items-start gap-6 xl:grid-cols-2">{children()}</div> }
        .into_any()
}

fn saved_announcement(title: Option<&str>) -> String {
    title.map_or_else(
        || "Changes saved.".to_owned(),
        |title| format!("{title} saved."),
    )
}

fn unsaved_summary(count: usize) -> String {
    match count {
        0 => "No Changes".to_owned(),
        1 => "1 Unsaved Change".to_owned(),
        _ => format!("{count} Unsaved Changes"),
    }
}

fn discard_warning(count: usize, title: Option<&str>) -> String {
    let changes = if count == 1 {
        "unsaved change".to_owned()
    } else {
        format!("{count} unsaved changes")
    };
    let subject = if count == 1 { "Your" } else { "All" };
    let verb = if count == 1 { "is" } else { "are" };
    match title {
        Some(title) => format!("{subject} {changes} to {title} {verb} lost if you continue."),
        None => format!("{subject} {changes} {verb} lost if you continue."),
    }
}

fn is_same_document(anchor: &HtmlAnchorElement) -> bool {
    let location = window().location();
    anchor.host() == location.host().unwrap_or_default()
        && anchor.pathname() == location.pathname().unwrap_or_default()
        && anchor.search() == location.search().unwrap_or_default()
}

fn navigating_anchor(event: &MouseEvent) -> Option<HtmlAnchorElement> {
    let modified = event.meta_key() || event.ctrl_key() || event.shift_key() || event.alt_key();
    if event.default_prevented() || event.button() != 0 || modified {
        return None;
    }
    let anchor = event
        .target()?
        .dyn_into::<Element>()
        .ok()?
        .closest("a[href]")
        .ok()??
        .dyn_into::<HtmlAnchorElement>()
        .ok()?;
    let opens_elsewhere = !matches!(anchor.target().as_str(), "" | "_self");
    let leaves_page = !opens_elsewhere && !anchor.has_attribute("download");
    (leaves_page && !is_same_document(&anchor)).then_some(anchor)
}

#[derive(Clone, Copy)]
struct EditSession {
    number: ElementKey,
    state: RwSignal<SectionState>,
    error: RwSignal<Option<String>>,
    errors_revealed: RwSignal<bool>,
    pending: PendingSave,
    attention: RwSignal<bool>,
    discard_prompt_open: RwSignal<bool>,
    rows: RwSignal<Vec<RowStatus>>,
    scope: Option<EditingScope>,
    section: NodeRef<Section>,
    edit_button: NodeRef<Button>,
    on_save: Callback<FieldSave>,
    start_highlight_timer: Callback<()>,
    start_attention_timer: Callback<()>,
}

impl EditSession {
    fn changed_count(self) -> usize {
        self.rows
            .with(|rows| rows.iter().filter(|row| row.changed.get()).count())
    }

    fn changed_count_untracked(self) -> usize {
        self.rows.with_untracked(|rows| {
            rows.iter()
                .filter(|row| row.changed.get_untracked())
                .count()
        })
    }

    fn has_unsaved_changes_untracked(self) -> bool {
        self.state.get_untracked().is_open() && self.changed_count_untracked() > 0
    }

    fn has_invalid_rows_untracked(self) -> bool {
        self.rows
            .with_untracked(|rows| rows.iter().any(|row| row.invalid.get_untracked()))
    }

    fn revert_rows(self) {
        let reverts: Vec<Callback<()>> = self
            .rows
            .with_untracked(|rows| rows.iter().map(|row| row.revert).collect());
        for revert in reverts {
            revert.run(());
        }
    }

    fn claim_scope(self, entry: ActiveEditor) -> bool {
        let Some(EditingScope(active)) = self.scope else {
            return true;
        };
        let other = active
            .get_untracked()
            .filter(|editor| editor.section_number != self.number);
        match other {
            Some(other) if other.has_changes.get_untracked() => {
                other.draw_attention.run(());
                return false;
            }
            Some(other) => other.discard.run(()),
            None => {}
        }
        active.set(Some(entry));
        true
    }

    fn release(self) {
        let Some(EditingScope(active)) = self.scope else {
            return;
        };
        let holds_scope = active
            .try_get_untracked()
            .flatten()
            .is_some_and(|editor| editor.section_number == self.number);
        if holds_scope {
            active.set(None);
        }
    }

    fn focus_after_render(self, selector: &'static str) {
        let section = self.section;
        request_animation_frame(move || {
            if let Some(element) = section.try_get_untracked().flatten() {
                focus_first(&element, selector);
            }
        });
    }

    fn return_focus_to_edit_button(self) {
        let (section, edit_button) = (self.section, self.edit_button);
        request_animation_frame(move || {
            let (Some(section), Some(edit_button)) = (
                section.try_get_untracked().flatten(),
                edit_button.try_get_untracked().flatten(),
            ) else {
                return;
            };
            if focus_is_within_or_lost(&section) {
                edit_button.focus().unwrap_or_default();
            }
        });
    }

    fn start(self, entry: ActiveEditor) {
        if !self.claim_scope(entry) {
            return;
        }
        self.revert_rows();
        self.error.set(None);
        self.errors_revealed.set(false);
        self.state.set(SectionState::Editing);
        self.focus_after_render(FIRST_CONTROL_SELECTOR);
    }

    fn close(self, state: SectionState) {
        self.state.set(state);
        self.release();
        self.return_focus_to_edit_button();
    }

    fn discard(self) {
        self.revert_rows();
        self.error.set(None);
        self.errors_revealed.set(false);
        self.close(SectionState::Viewing);
    }

    fn request_discard(self) {
        if self.changed_count_untracked() > 0 {
            self.discard_prompt_open.set(true);
        } else {
            self.discard();
        }
    }

    fn save(self) {
        if self.state.get_untracked() != SectionState::Editing {
            return;
        }
        self.error.set(None);
        self.errors_revealed.set(true);
        if self.has_invalid_rows_untracked() {
            self.focus_after_render(FIRST_INVALID_CONTROL_SELECTOR);
            return;
        }
        if self.changed_count_untracked() == 0 {
            self.close(SectionState::Viewing);
            return;
        }
        self.state.set(SectionState::Saving);
        self.on_save.run(self.pending.begin());
    }

    fn resolve(self, result: SaveOutcome) {
        match result {
            SaveOutcome::Succeeded => {
                self.close(SectionState::Saved);
                self.start_highlight_timer.run(());
            }
            SaveOutcome::Unchanged => self.close(SectionState::Viewing),
            SaveOutcome::Invalid => {
                self.errors_revealed.set(true);
                self.state.set(SectionState::Editing);
                self.focus_after_render(FIRST_INVALID_CONTROL_SELECTOR);
            }
            SaveOutcome::Failed(message) => {
                self.error.set(Some(message));
                self.state.set(SectionState::Editing);
            }
        }
    }

    fn draw_attention(self) {
        let Some(element) = self.section.get_untracked() else {
            return;
        };
        focus_first(&element, FIRST_CONTROL_SELECTOR);
        let scrolling = ScrollIntoViewOptions::new();
        scrolling.set_block(ScrollLogicalPosition::Center);
        element.scroll_into_view_with_scroll_into_view_options(&scrolling);
        self.attention.set(true);
        self.start_attention_timer.run(());
    }

    fn on_keydown(self, event: &KeyboardEvent) {
        if event.default_prevented() || self.state.get_untracked() != SectionState::Editing {
            return;
        }
        match event.key().as_str() {
            "Enter" if saves_on_enter(event) => {
                event.prevent_default();
                self.save();
            }
            "Escape" => {
                event.prevent_default();
                self.request_discard();
            }
            _ => {}
        }
    }
}

fn guard_navigation(session: EditSession, pending_navigation: PendingNavigation) {
    drop(use_event_listener_with_options(
        use_window(),
        leptos::ev::click,
        move |event: MouseEvent| {
            if !session.has_unsaved_changes_untracked() {
                return;
            }
            let Some(anchor) = navigating_anchor(&event) else {
                return;
            };
            event.prevent_default();
            event.stop_propagation();
            pending_navigation.set_value(Some(SendWrapper::new(anchor)));
            session.discard_prompt_open.set(true);
        },
        UseEventListenerOptions::default().capture(true),
    ));
    drop(use_event_listener(
        use_window(),
        leptos::ev::beforeunload,
        move |event: BeforeUnloadEvent| {
            if session.has_unsaved_changes_untracked() {
                event.prevent_default();
                event.set_return_value("");
            }
        },
    ));
}

fn discard_dialog(
    session: EditSession,
    title: Option<String>,
    pending_navigation: PendingNavigation,
) -> AnyView {
    let warning =
        Signal::derive(move || discard_warning(session.changed_count(), title.as_deref()));
    let on_confirm = Callback::new(move |()| {
        session.discard();
        let navigation = pending_navigation.try_update_value(Option::take).flatten();
        if let Some(anchor) = navigation {
            anchor.click();
        }
    });
    view! {
        <ConfirmDialog
            open=session.discard_prompt_open
            title="Discard Changes?"
            body=warning
            confirm_label="Discard Changes"
            cancel_label="Keep Editing"
            tone=ButtonTone::Danger
            on_confirm=on_confirm
        />
    }
    .into_any()
}

fn edit_button(session: EditSession, entry: ActiveEditor, label: String) -> AnyView {
    let class = move || {
        format!(
            "{} {}",
            button_class(ButtonTone::Subtle, ButtonSize::Dense),
            if session.state.get().is_open() {
                "hidden"
            } else {
                ""
            },
        )
    };
    view! {
        <button
            node_ref=session.edit_button
            type="button"
            class=class
            aria-label=label
            on:click=move |_| session.start(entry)
        >
            "Edit"
        </button>
    }
    .into_any()
}

fn editing_footer(session: EditSession) -> AnyView {
    let saving = move || session.state.get() == SectionState::Saving;
    let class = move || {
        format!(
            "flex flex-wrap items-center justify-between gap-x-3 gap-y-2 rounded-b-surface-inner border-t border-gray-200 bg-gray-50 px-4 py-2 {}",
            if session.state.get().is_open() { "" } else { "hidden" },
        )
    };
    view! {
        <div class=class>
            <p class="text-sm text-gray-500">{move || unsaved_summary(session.changed_count())}</p>
            <div class="flex gap-2">
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                    aria-disabled=move || saving().to_string()
                    on:click=move |_| {
                        if !saving() {
                            session.discard();
                        }
                    }
                >
                    "Cancel"
                </button>
                <button
                    type="button"
                    class=button_class(ButtonTone::Primary, ButtonSize::Compact)
                    aria-disabled=move || saving().to_string()
                    on:click=move |_| session.save()
                >
                    "Save"
                </button>
            </div>
        </div>
    }
    .into_any()
}

fn heading_view(level: HeadingLevel, id: String, title: String) -> AnyView {
    match level {
        HeadingLevel::Two => view! {
            <h2 id=id class=LABEL_CLASS>
                {title}
            </h2>
        }
        .into_any(),
        HeadingLevel::Three => view! {
            <h3 id=id class=LABEL_CLASS>
                {title}
            </h3>
        }
        .into_any(),
        HeadingLevel::Four => view! {
            <h4 id=id class=LABEL_CLASS>
                {title}
            </h4>
        }
        .into_any(),
    }
}

fn status_view(state: RwSignal<SectionState>, announcement: String) -> impl IntoView {
    move || match state.get() {
        SectionState::Saving => Some(
            view! {
                {icons::spinner("h-5 w-5 text-blue-600")}
                "Saving\u{2026}"
            }
            .into_any(),
        ),
        SectionState::Saved => {
            Some(view! { <span class="sr-only">{announcement.clone()}</span> }.into_any())
        }
        SectionState::Viewing | SectionState::Editing => None,
    }
}

const SECTION_LOADING_HEIGHT_CLASS: &str = "h-48";

#[expect(
    clippy::too_many_lines,
    reason = "one widget: the header, the rows, the footer and the saving overlay share the edit state"
)]
#[component]
pub fn FieldSection(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] heading_level: HeadingLevel,
    #[prop(optional)] actions: Option<AnyView>,
    #[prop(optional)] editing: Option<SectionEditing>,
    #[prop(optional, into)] loading: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let section_number = ElementKey::next();
    let heading_id = title
        .as_ref()
        .map(|_| format!("field-section-{section_number}-heading"));
    let state = RwSignal::new(SectionState::Viewing);
    let errors_revealed = RwSignal::new(false);
    let rows: RwSignal<Vec<RowStatus>> = RwSignal::new(Vec::new());
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let attention = RwSignal::new(false);
    let section: NodeRef<Section> = NodeRef::new();
    let section_context = SectionContext {
        state: state.into(),
        errors_revealed: errors_revealed.into(),
        rows,
    };

    let editing = editing.filter(|editing| editing.allowed);
    let session = editing.as_ref().map(|editing| EditSession {
        number: section_number,
        state,
        error,
        errors_revealed,
        pending: PendingSave::new(),
        attention,
        discard_prompt_open: RwSignal::new(false),
        rows,
        scope: use_context::<EditingScope>(),
        section,
        edit_button: NodeRef::new(),
        on_save: editing.on_save,
        start_highlight_timer: timer(
            move || {
                if state.get_untracked() == SectionState::Saved {
                    state.set(SectionState::Viewing);
                }
            },
            SAVED_HIGHLIGHT_MILLISECONDS,
        ),
        start_attention_timer: timer(move || attention.set(false), ATTENTION_MILLISECONDS),
    });

    if let Some(session) = session {
        session.pending.resolve_with(
            move || state.get() == SectionState::Saving,
            move |result| session.resolve(result),
        );
        on_cleanup(move || session.release());
    }

    if let Some(on_change) = editing.and_then(|editing| editing.on_editing_change) {
        Effect::new(move |previous: Option<bool>| {
            let open = state.get().is_open();
            if previous.is_some_and(|previous| previous != open) {
                on_change.run(open);
            }
            open
        });
    }

    let pending_navigation: PendingNavigation = StoredValue::new(None);
    let edit_controls = session.map(|session| {
        guard_navigation(session, pending_navigation);
        let entry = ActiveEditor {
            section_number,
            has_changes: Signal::derive(move || session.changed_count() > 0),
            discard: Callback::new(move |()| session.discard()),
            draw_attention: Callback::new(move |()| session.draw_attention()),
        };
        let label = title
            .as_ref()
            .map_or_else(|| "Edit".to_owned(), |title| format!("Edit {title}"));
        edit_button(session, entry, label)
    });
    let footer = session.map(editing_footer);
    let dialog = session.map(|session| discard_dialog(session, title.clone(), pending_navigation));

    let announcement = saved_announcement(title.as_deref());
    let heading = (title.is_some() || actions.is_some() || edit_controls.is_some()).then(|| {
        let heading = title
            .zip(heading_id.clone())
            .map(|(title, id)| heading_view(heading_level, id, title));
        view! {
            <div class="mb-1 flex min-h-7 items-center justify-between gap-3">
                {heading}
                <div class="ml-auto flex items-center gap-2" class:hidden=move || loading.get()>
                    {actions}
                    {edit_controls}
                </div>
            </div>
        }
    });
    let saving = move || state.get() == SectionState::Saving;
    let card_class = move || {
        format!(
            "rounded-surface border border-gray-200 bg-white motion-safe:transition-shadow motion-safe:duration-300 {} {}",
            if attention.get() { "ring-2 ring-focus ring-offset-2" } else { "" },
            if loading.get() { "hidden" } else { "" },
        )
    };
    let on_keydown = move |event: KeyboardEvent| {
        if let Some(session) = session {
            session.on_keydown(&event);
        }
    };

    view! {
        <section
            node_ref=section
            aria-labelledby=heading_id
            class="w-full max-w-2xl"
            on:keydown=on_keydown
        >
            {heading}
            {move || {
                loading.get().then(|| view! { <LoadingState class=SECTION_LOADING_HEIGHT_CLASS /> })
            }}
            <div class=card_class>
                <div class="relative">
                    <dl
                        aria-busy=move || saving().to_string()
                        inert=move || saving().then_some("")
                        class=move || {
                            format!(
                                "motion-safe:transition-opacity motion-safe:duration-200 {}",
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
                                if saving() {
                                    "opacity-100"
                                } else {
                                    "pointer-events-none opacity-0"
                                },
                            )
                        }
                    >
                        {status_view(state, announcement)}
                    </div>
                </div>
                {footer}
            </div>
            {move || {
                error
                    .get()
                    .map(|message| {
                        view! {
                            <div class="mt-2">
                                <Alert tone=Tone::Critical>{message}</Alert>
                            </div>
                        }
                    })
            }}
            {dialog}
        </section>
    }
    .into_any()
}

fn revert_button(
    label: String,
    revert: Callback<()>,
    unsaved: Signal<bool>,
    row: NodeRef<Div>,
) -> AnyView {
    let on_click = move |_| {
        revert.run(());
        if let Some(element) = row.get_untracked() {
            focus_first(&element, FIRST_CONTROL_SELECTOR);
        }
    };
    view! {
        <Tooltip label=label.clone()>
            <button
                type="button"
                class=move || {
                    format!(
                        "{} {}",
                        icon_button_class(ButtonTone::Ghost, ButtonSize::Compact),
                        if unsaved.get() { "" } else { "invisible" },
                    )
                }
                aria-label=label
                on:click=on_click
            >
                {icons::icon(icons::LucideGlyph::RotateCcw, "h-4 w-4")}
            </button>
        </Tooltip>
    }
    .into_any()
}

#[derive(Clone)]
struct RowIds {
    control: String,
    label: String,
    help: Option<String>,
    error: Option<String>,
}

impl RowIds {
    fn new(number: ElementKey, has_help: bool, has_editor: bool) -> Self {
        Self {
            control: format!("field-{number}"),
            label: format!("field-{number}-label"),
            help: has_help.then(|| format!("field-{number}-help")),
            error: has_editor.then(|| format!("field-{number}-error")),
        }
    }

    fn description(&self) -> Option<String> {
        let ids: Vec<&str> = [self.help.as_deref(), self.error.as_deref()]
            .into_iter()
            .flatten()
            .collect();
        (!ids.is_empty()).then(|| ids.join(" "))
    }
}

fn field_label(label: String, ids: &RowIds, help: Option<String>) -> AnyView {
    let label_element = if ids.error.is_some() {
        view! {
            <label id=ids.label.clone() for=ids.control.clone()>
                {label}
            </label>
        }
        .into_any()
    } else {
        view! { <span id=ids.label.clone()>{label}</span> }.into_any()
    };
    match (help, ids.help.clone()) {
        (Some(help), Some(help_id)) => view! {
            <HelpTerm help help_id>
                {label_element}
            </HelpTerm>
        }
        .into_any(),
        _ => label_element,
    }
}

fn register_row(rows: RwSignal<Vec<RowStatus>>, status: RowStatus) {
    rows.update(|rows| rows.push(status));
    on_cleanup(move || {
        rows.try_update(|rows| rows.retain(|row| row.number != status.number));
    });
}

#[expect(
    clippy::too_many_lines,
    reason = "one row: the label, the value, the editor and its error share the section state"
)]
#[component]
pub fn FieldRow(
    #[prop(into)] label: String,
    #[prop(optional, into)] help: Option<String>,
    #[prop(optional)] editor: Option<FieldEditor>,
    #[prop(optional)] actions: Vec<FieldAction>,
    children: Children,
) -> impl IntoView {
    let row: NodeRef<Div> = NodeRef::new();
    let row_number = ElementKey::next();
    let ids = RowIds::new(row_number, help.is_some(), editor.is_some());
    let context = use_context::<SectionContext>();
    let state = context.map_or_else(
        || Signal::stored(SectionState::Viewing),
        |context| context.state,
    );
    let errors_revealed =
        context.map_or_else(|| Signal::stored(false), |context| context.errors_revealed);
    let has_editor = editor.is_some();
    let section_open = move || state.get().is_open();
    let showing_editor = Signal::derive(move || has_editor && section_open());
    let changed = editor
        .as_ref()
        .map_or_else(Signal::default, |editor| editor.changed);
    let error = editor
        .as_ref()
        .map_or_else(Signal::default, |editor| editor.error);
    let visible_error = Signal::derive(move || {
        if showing_editor.get() && errors_revealed.get() {
            error.get()
        } else {
            None
        }
    });
    if let (Some(context), Some(editor)) = (context, editor.as_ref()) {
        register_row(
            context.rows,
            RowStatus {
                number: row_number,
                changed: editor.changed,
                invalid: Signal::derive(move || error.with(Option::is_some)),
                revert: editor.revert,
            },
        );
    }
    provide_context(FieldElementIds {
        control: ids.control.clone(),
        label: ids.label.clone(),
        description: ids.description(),
        invalid: Signal::derive(move || visible_error.with(Option::is_some)),
    });

    let changed_when_saved = RwSignal::new(false);
    Effect::new(move |_| {
        if state.get() == SectionState::Saving {
            changed_when_saved.set(changed.get_untracked());
        }
    });
    let highlighted = move || state.get() == SectionState::Saved && changed_when_saved.get();
    let unsaved = Signal::derive(move || showing_editor.get() && changed.get());
    let row_class = move || {
        let highlight = if unsaved.get() {
            "bg-amber-50"
        } else if highlighted() {
            "motion-safe:animate-field-saved"
        } else {
            ""
        };
        let bottom_corners = if section_open() {
            ""
        } else {
            "last:rounded-b-surface-inner"
        };
        format!("{ROW_CLASS} {highlight} {bottom_corners}")
    };
    let read_only_while_editing = move || !has_editor && section_open();

    let editor_view = editor.map(|editor| {
        let revert = revert_button(format!("Revert {label}"), editor.revert, unsaved, row);
        view! {
            <div class=move || {
                if showing_editor.get() {
                    "flex items-start gap-2 pb-1.5 pt-1 sm:justify-end sm:py-0"
                } else {
                    "hidden"
                }
            }>
                <div class="order-last mt-0.5 sm:order-none">{revert}</div>
                <div class="min-w-0 max-w-sm flex-1 text-left">
                    {editor.view.run()}
                    <p
                        id=ids.error.clone()
                        class=move || {
                            if visible_error.with(Option::is_some) {
                                "mt-1.5 text-xs text-red-600"
                            } else {
                                "hidden"
                            }
                        }
                    >
                        {move || visible_error.get()}
                    </p>
                </div>
            </div>
        }
    });
    let label_view = field_label(label, &ids, help);

    view! {
        <div node_ref=row class=row_class>
            <dt class=format!(
                "pt-1.5 text-sm text-gray-500 sm:w-48 sm:shrink-0 {ALIGNED_WITH_CONTROL_TEXT_CLASS}",
            )>{label_view}</dt>
            <dd class="min-w-0 flex-1 text-sm text-gray-900">
                <div class=move || {
                    if showing_editor.get() {
                        "hidden"
                    } else {
                        "flex items-center gap-3 sm:justify-end"
                    }
                }>
                    <div class=move || {
                        format!(
                            "min-w-0 pb-1.5 pt-0.5 motion-safe:transition-opacity sm:text-right {ALIGNED_WITH_CONTROL_TEXT_CLASS} {}",
                            if read_only_while_editing() { "opacity-60" } else { "" },
                        )
                    }>{children()}</div>
                    {field_actions_view(actions)}
                </div>
                {editor_view}
            </dd>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announces_the_saved_section() {
        assert_eq!(
            saved_announcement(Some("Classification")),
            "Classification saved."
        );
    }

    #[test]
    fn announces_a_save_without_a_title() {
        assert_eq!(saved_announcement(None), "Changes saved.");
    }

    #[test]
    fn counts_unsaved_changes() {
        assert_eq!(unsaved_summary(0), "No Changes");
        assert_eq!(unsaved_summary(1), "1 Unsaved Change");
        assert_eq!(unsaved_summary(3), "3 Unsaved Changes");
    }

    #[test]
    fn warns_about_one_unsaved_change() {
        assert_eq!(
            discard_warning(1, Some("Classification")),
            "Your unsaved change to Classification is lost if you continue."
        );
    }

    #[test]
    fn warns_about_several_unsaved_changes() {
        assert_eq!(
            discard_warning(2, None),
            "All 2 unsaved changes are lost if you continue."
        );
    }

    #[test]
    fn describes_a_row_by_its_help_and_error() {
        let key = ElementKey::next();
        let ids = RowIds::new(key, true, true);
        assert_eq!(
            ids.description(),
            Some(format!("field-{key}-help field-{key}-error"))
        );
    }

    #[test]
    fn leaves_a_plain_row_undescribed() {
        assert_eq!(
            RowIds::new(ElementKey::next(), false, false).description(),
            None
        );
    }
}
