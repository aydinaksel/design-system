use std::collections::HashSet;
use std::sync::Arc;

use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html::Tr;
use leptos::prelude::*;

use crate::button::{button_class, button_icon_class, icon_button_class, ButtonSize, ButtonTone};
use crate::confirm_dialog::ConfirmDialog;
use crate::controls::FieldElementIds;
use crate::element_key::ElementKey;
use crate::field_values::missing_value;
use crate::icons;
use crate::menu::{RowActions, DESTRUCTIVE_BUTTON_CLASS, DESTRUCTIVE_ICON_CLASS};
use crate::tooltip::Tooltip;

use super::cell_value::Align;
use super::column::Column;
use super::data_table::{DataTable, RowAttribute, RowReplacement, Search};

const ACTIONS_KEY: &str = "actions";
const ACTIONS_WIDTH_CHARACTERS: u16 = 9;
const EDITING_ROW_CLASS: &str = "bg-blue-50/40";
const EDITING_CELL_CLASS: &str = "px-1 py-1.5 align-top";
const RIGHT_ALIGNED_EDITING_CLASS: &str = "[&_input]:text-right [&_.line-shown]:justify-end [&_.line-calculated]:flex-row-reverse [&_.line-calculated]:justify-start [&_.line-overridden]:flex-row-reverse";
const STATIC_CELL_CLASS: &str = "truncate px-4 py-2.5 align-top text-gray-700";
const ISSUES_CELL_CLASS: &str = "bg-blue-50/40 px-4 pb-4 pt-3";
const ISSUE_CLASS: &str = "flex items-start gap-1.5 text-xs text-red-600";
const ISSUE_ICON_CLASS: &str = "mt-px h-3.5 w-3.5 shrink-0";
const CALCULATED_CLASS: &str =
    "line-calculated flex h-9 items-center justify-between gap-1 px-3 text-sm tabular-nums";
const OVERRIDDEN_CLASS: &str = "line-overridden flex items-center gap-1";
const CALCULATED_VALUE_CLASS: &str = "text-gray-500";
const SHOWN_CLASS: &str =
    "line-shown flex h-9 items-center px-3 text-sm tabular-nums text-gray-900";
const ICON_CLASS: &str = "h-4 w-4";
const SAVE_ICON_CLASS: &str = "h-4 w-4 text-blue-600";
const ACTION_SIZE: ButtonSize = ButtonSize::Compact;
const FIX_SIZE: ButtonSize = ButtonSize::Dense;
const ADD_SIZE: ButtonSize = ButtonSize::Compact;
const SAVE_LABEL: &str = "Save Line";
const CANCEL_LABEL: &str = "Cancel";
const OVERRIDE_LABEL: &str = "Override";
const USE_CALCULATED_LABEL: &str = "Use Calculated";
const CALCULATED_LABEL: &str = "Calculated";
const DISCARD_TITLE: &str = "Discard Changes?";
const DISCARD_QUESTION: &str = "The line you are editing has changes that are not saved.";
const DISCARD_LABEL: &str = "Discard Changes";
const ACTIONS_CELL_CLASS: &str = "px-4 py-1.5 align-top";
const DELETE_QUESTION: &str = "This removes the line from the order.";
const NOT_CALCULATED: &str = "\u{2014}";

pub struct LineFix<Draft> {
    pub label: String,
    pub apply: Arc<dyn Fn(&mut Draft) + Send + Sync>,
}

impl<Draft> Clone for LineFix<Draft> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            apply: Arc::clone(&self.apply),
        }
    }
}

pub struct LineIssue<Draft> {
    pub field: &'static str,
    pub message: String,
    pub fixes: Vec<LineFix<Draft>>,
}

impl<Draft> Clone for LineIssue<Draft> {
    fn clone(&self) -> Self {
        Self {
            field: self.field,
            message: self.message.clone(),
            fixes: self.fixes.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EditTarget {
    Existing(String),
    New,
}

pub struct LineSave<Draft: Send + Sync + 'static> {
    editing: RwSignal<Option<EditTarget>>,
    saving: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    rejected: RwSignal<Vec<LineIssue<Draft>>>,
    revealed: RwSignal<bool>,
}

impl<Draft: Send + Sync + 'static> Clone for LineSave<Draft> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Draft: Send + Sync + 'static> Copy for LineSave<Draft> {}

impl<Draft: Send + Sync + 'static> LineSave<Draft> {
    pub fn succeeded(self) {
        self.saving.set(false);
        self.editing.set(None);
    }

    pub fn failed(self, message: impl Into<String>) {
        self.saving.set(false);
        self.error.set(Some(message.into()));
    }

    pub fn rejected(self, issues: Vec<LineIssue<Draft>>) {
        self.saving.set(false);
        self.revealed.set(true);
        self.rejected.set(issues);
    }
}

#[derive(Clone, Copy)]
pub struct CalculatedField {
    pub calculated: Signal<Option<String>>,
    pub overridden: Signal<bool>,
    pub on_override: Callback<()>,
    pub on_use_calculated: Callback<()>,
}

pub type LineCell = (&'static str, AnyView);

pub struct LineForm<Draft: Send + Sync + 'static> {
    draft: RwSignal<Draft>,
    issues: Signal<Vec<LineIssue<Draft>>>,
    touched: RwSignal<HashSet<&'static str>>,
    calculated_keys: RwSignal<HashSet<&'static str>>,
    headers: Arc<[(&'static str, &'static str)]>,
}

impl<Draft: Send + Sync + 'static> Clone for LineForm<Draft> {
    fn clone(&self) -> Self {
        Self {
            draft: self.draft,
            issues: self.issues,
            touched: self.touched,
            calculated_keys: self.calculated_keys,
            headers: Arc::clone(&self.headers),
        }
    }
}

impl<Draft: Clone + Send + Sync + 'static> LineForm<Draft> {
    #[must_use]
    pub fn draft(&self) -> RwSignal<Draft> {
        self.draft
    }

    pub fn bind(
        &self,
        read: impl Fn(&Draft) -> String + Send + Sync + Clone + 'static,
        write: impl Fn(&mut Draft, String) + Send + Sync + 'static,
    ) -> RwSignal<String> {
        let draft = self.draft;
        let part = RwSignal::new(draft.with_untracked(&read));
        Effect::watch(
            move || part.get(),
            move |text, _, _| draft.update(|value| write(value, text.clone())),
            false,
        );
        Effect::watch(
            move || draft.with(&read),
            move |text, _, _| {
                if part.with_untracked(|current| current != text) {
                    part.set(text.clone());
                }
            },
            false,
        );
        part
    }

    pub fn cell<Control: IntoView + 'static>(
        &self,
        key: &'static str,
        control: impl FnOnce() -> Control,
    ) -> LineCell {
        (key, self.labelled(key, control))
    }

    #[must_use]
    pub fn shown(&self, key: &'static str, value: Signal<String>) -> LineCell {
        let view = view! { <div class=SHOWN_CLASS>{move || value.get()}</div> };
        (key, view.into_any())
    }

    pub fn calculated<Control: IntoView + 'static>(
        &self,
        key: &'static str,
        field: CalculatedField,
        override_control: impl FnOnce() -> Control,
    ) -> LineCell {
        let CalculatedField {
            calculated,
            overridden,
            on_override,
            on_use_calculated,
        } = field;
        self.calculated_keys.update(|keys| {
            keys.insert(key);
        });
        let control = self.labelled(key, override_control);
        let override_class = icon_button_class(ButtonTone::Ghost, ButtonSize::Dense);
        let reset_class = override_class.clone();
        let view = view! {
            <div class=OVERRIDDEN_CLASS class:hidden=move || !overridden.get()>
                <div class="min-w-0 flex-1">{control}</div>
                <Tooltip label=USE_CALCULATED_LABEL>
                    <button
                        type="button"
                        class=reset_class
                        aria-label=USE_CALCULATED_LABEL
                        on:click=move |_| on_use_calculated.run(())
                    >
                        {icons::icon(icons::LucideGlyph::RotateCcw, ICON_CLASS)}
                    </button>
                </Tooltip>
            </div>
            <div class=CALCULATED_CLASS class:hidden=move || overridden.get()>
                <Tooltip label=CALCULATED_LABEL>
                    <span class=CALCULATED_VALUE_CLASS>
                        {move || calculated.get().unwrap_or_else(|| NOT_CALCULATED.to_owned())}
                    </span>
                </Tooltip>
                <Tooltip label=OVERRIDE_LABEL>
                    <button
                        type="button"
                        class=override_class
                        aria-label=OVERRIDE_LABEL
                        on:click=move |_| on_override.run(())
                    >
                        {icons::icon(icons::LucideGlyph::Pencil, ICON_CLASS)}
                    </button>
                </Tooltip>
            </div>
        };
        (key, view.into_any())
    }

    fn header_of(&self, key: &str) -> &'static str {
        header_in(&self.headers, key)
    }

    fn labelled<Control: IntoView + 'static>(
        &self,
        key: &'static str,
        control: impl FnOnce() -> Control,
    ) -> AnyView {
        let field_number = ElementKey::next();
        let control_id = format!("line-cell-{field_number}");
        let label_id = format!("line-cell-{field_number}-label");
        let touched = self.touched;
        let issues = self.issues;
        let invalid = Signal::derive(move || {
            issues.with(|issues| issues.iter().any(|issue| issue.field == key))
        });
        let owner = Owner::new();
        let control_view = owner.with(|| {
            provide_context(FieldElementIds {
                control: control_id.clone(),
                label: label_id.clone(),
                description: None,
                invalid,
            });
            control().into_any()
        });
        view! {
            <label id=label_id for=control_id class="sr-only">
                {self.header_of(key)}
            </label>
            <div
                class="contents"
                on:focusout=move |_| {
                    touched
                        .update(|keys| {
                            keys.insert(key);
                        });
                }
            >
                {control_view}
            </div>
        }
        .into_any()
    }
}

fn header_in(headers: &[(&'static str, &'static str)], key: &str) -> &'static str {
    headers
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map_or("", |(_, header)| *header)
}

fn issue_view<Draft: Send + Sync + 'static>(
    issue: LineIssue<Draft>,
    header: &'static str,
    draft: RwSignal<Draft>,
) -> AnyView {
    let has_fixes = !issue.fixes.is_empty();
    let fixes = issue
        .fixes
        .into_iter()
        .map(|fix| {
            let apply = Arc::clone(&fix.apply);
            view! {
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, FIX_SIZE)
                    on:click=move |_| draft.update(|value| apply(value))
                >
                    {fix.label}
                </button>
            }
        })
        .collect_view();
    let message = if header.is_empty() {
        issue.message
    } else {
        format!("{header}: {}", issue.message)
    };
    view! {
        <div class="space-y-1.5">
            <p class=ISSUE_CLASS>
                {icons::icon(icons::LucideGlyph::TriangleAlert, ISSUE_ICON_CLASS)}
                <span>{message}</span>
            </p>
            {has_fixes.then(|| view! { <div class="flex flex-wrap gap-1.5">{fixes}</div> })}
        </div>
    }
    .into_any()
}

type FormCells<Draft> = Arc<dyn Fn(&LineForm<Draft>) -> Vec<LineCell> + Send + Sync>;
type DraftCheck<Draft> = Arc<dyn Fn(&Draft) -> Vec<LineIssue<Draft>> + Send + Sync>;
type DraftOf<Row, Draft> = Arc<dyn Fn(&Row) -> Draft + Send + Sync>;

#[cfg(feature = "hydrate")]
fn focus_first_control(row: &web_sys::HtmlTableRowElement) {
    use leptos::wasm_bindgen::JsCast;
    let first = row
        .query_selector(crate::editing::FIRST_CONTROL_SELECTOR)
        .ok()
        .flatten()
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
    if let Some(control) = first {
        control.focus().unwrap_or_default();
    }
}

struct LineEditor<Row: Send + Sync + 'static, Draft: Send + Sync + 'static> {
    columns: Arc<[Column<Row>]>,
    editing: RwSignal<Option<EditTarget>>,
    draft: RwSignal<Draft>,
    baseline: RwSignal<Option<Draft>>,
    switching: RwSignal<Option<(EditTarget, Draft)>>,
    confirming: RwSignal<bool>,
    save_state: LineSave<Draft>,
    issues: Signal<Vec<LineIssue<Draft>>>,
    shown_issues: Signal<Vec<LineIssue<Draft>>>,
    touched: RwSignal<HashSet<&'static str>>,
    calculated_keys: RwSignal<HashSet<&'static str>>,
    form: FormCells<Draft>,
    on_save: Callback<(Option<String>, Draft, LineSave<Draft>)>,
}

impl<Row: Send + Sync + 'static, Draft: Send + Sync + 'static> Clone for LineEditor<Row, Draft> {
    fn clone(&self) -> Self {
        Self {
            columns: Arc::clone(&self.columns),
            editing: self.editing,
            draft: self.draft,
            baseline: self.baseline,
            switching: self.switching,
            confirming: self.confirming,
            save_state: self.save_state,
            issues: self.issues,
            shown_issues: self.shown_issues,
            touched: self.touched,
            calculated_keys: self.calculated_keys,
            form: Arc::clone(&self.form),
            on_save: self.on_save,
        }
    }
}

impl<Row, Draft> LineEditor<Row, Draft>
where
    Row: Clone + Send + Sync + 'static,
    Draft: Clone + PartialEq + Send + Sync + 'static,
{
    fn has_changes(&self) -> bool {
        let baseline = self.baseline.get_untracked();
        self.draft
            .with_untracked(|draft| baseline.as_ref().is_some_and(|baseline| draft != baseline))
    }

    fn start(&self, target: EditTarget, draft: Draft) {
        if self.save_state.saving.get_untracked() {
            return;
        }
        let switching_away = self
            .editing
            .with_untracked(|current| current.as_ref().is_some_and(|current| *current != target));
        if switching_away && self.has_changes() {
            self.switching.set(Some((target, draft)));
            self.confirming.set(true);
            return;
        }
        self.open(target, draft);
    }

    fn discard_and_switch(&self) {
        let waiting = self.switching.get_untracked();
        self.switching.set(None);
        if let Some((target, draft)) = waiting {
            self.open(target, draft);
        }
    }

    fn open(&self, target: EditTarget, draft: Draft) {
        self.touched.set(HashSet::new());
        self.save_state
            .revealed
            .set(matches!(target, EditTarget::Existing(_)));
        self.baseline.set(Some(draft.clone()));
        self.draft.set(draft);
        self.save_state.error.set(None);
        self.save_state.rejected.set(Vec::new());
        self.editing.set(Some(target));
    }

    fn cancel(&self) {
        if !self.save_state.saving.get_untracked() {
            self.editing.set(None);
        }
    }

    fn save(&self) {
        if self.save_state.saving.get_untracked() {
            return;
        }
        if self.issues.with_untracked(|issues| !issues.is_empty()) {
            self.save_state.revealed.set(true);
            return;
        }
        let key = match self.editing.get_untracked() {
            Some(EditTarget::Existing(key)) => Some(key),
            Some(EditTarget::New) => None,
            None => return,
        };
        self.save_state.saving.set(true);
        self.save_state.error.set(None);
        self.on_save
            .run((key, self.draft.get_untracked(), self.save_state));
    }

    fn headers(&self) -> Arc<[(&'static str, &'static str)]> {
        self.columns
            .iter()
            .map(|column| (column.key, column.header))
            .collect()
    }

    fn save_controls(&self) -> AnyView {
        let saving = self.save_state.saving;
        let saver = self.clone();
        let canceller = self.clone();
        let save_class = icon_button_class(ButtonTone::Ghost, ACTION_SIZE);
        let cancel_class = format!("{save_class} {DESTRUCTIVE_BUTTON_CLASS}");
        let blocked = move || saving.get();
        let save_icon = move || {
            if saving.get() {
                icons::spinner(ICON_CLASS)
            } else {
                icons::icon(icons::LucideGlyph::Check, SAVE_ICON_CLASS)
            }
        };
        view! {
            <div class="flex h-9 items-center justify-end gap-0.5">
                <Tooltip label=SAVE_LABEL>
                    <button
                        type="button"
                        class=save_class
                        aria-label=SAVE_LABEL
                        disabled=blocked
                        on:click=move |_| saver.save()
                    >
                        {save_icon}
                    </button>
                </Tooltip>
                <Tooltip label=CANCEL_LABEL>
                    <button
                        type="button"
                        class=cancel_class
                        aria-label=CANCEL_LABEL
                        on:mousedown=|event: MouseEvent| event.prevent_default()
                        on:click=move |_| canceller.cancel()
                    >
                        {icons::icon(icons::LucideGlyph::X, DESTRUCTIVE_ICON_CLASS)}
                    </button>
                </Tooltip>
            </div>
        }
        .into_any()
    }

    fn editing_cell(
        &self,
        column: &Column<Row>,
        cells: &mut Vec<LineCell>,
        saved: Option<&Row>,
    ) -> AnyView {
        if column.key == ACTIONS_KEY {
            return view! { <td class=ACTIONS_CELL_CLASS>{self.save_controls()}</td> }.into_any();
        }
        let position = cells.iter().position(|(key, _)| *key == column.key);
        if let Some((_, cell)) = position.map(|index| cells.swap_remove(index)) {
            let alignment = match column.align {
                Align::Right => RIGHT_ALIGNED_EDITING_CLASS,
                Align::Left | Align::Center => "",
            };
            let class = format!("{EDITING_CELL_CLASS} {alignment}");
            return view! { <td class=class>{cell}</td> }.into_any();
        }
        let content = saved.map_or_else(missing_value, |row| (column.render)(row));
        let class = format!("{STATIC_CELL_CLASS} {}", column.align.text_class());
        view! { <td class=class>{content}</td> }.into_any()
    }

    fn editing_cells(&self, saved: Option<&Row>) -> AnyView {
        let form = LineForm {
            draft: self.draft,
            issues: self.shown_issues,
            touched: self.touched,
            calculated_keys: self.calculated_keys,
            headers: self.headers(),
        };
        let mut cells = (self.form)(&form);
        self.columns
            .iter()
            .map(|column| self.editing_cell(column, &mut cells, saved))
            .collect_view()
            .into_any()
    }

    fn issues_row(&self) -> impl IntoView {
        let issues = self.shown_issues;
        let error = self.save_state.error;
        let draft = self.draft;
        let headers = self.headers();
        let column_count = self.columns.len().to_string();
        move || {
            let listed = issues.get();
            let failure = error.get();
            if listed.is_empty() && failure.is_none() {
                return None;
            }
            let issue_views = listed
                .into_iter()
                .map(|issue| {
                    let header = header_in(&headers, issue.field);
                    issue_view(issue, header, draft)
                })
                .collect_view();
            let failure_view = failure.map(|message| view! { <p class=ISSUE_CLASS>{message}</p> });
            Some(view! {
                <tr>
                    <td colspan=column_count.clone() class=ISSUES_CELL_CLASS>
                        <div role="alert" class="space-y-2">
                            {issue_views}
                            {failure_view}
                        </div>
                    </td>
                </tr>
            })
        }
    }

    fn editing_rows(&self, saved: Option<&Row>) -> AnyView {
        let row: NodeRef<Tr> = NodeRef::new();
        #[cfg(feature = "hydrate")]
        Effect::new(move |_| {
            if let Some(element) = row.get() {
                focus_first_control(&element);
            }
        });
        let save_editor = self.clone();
        let cancel_editor = self.clone();
        let on_keydown = move |event: KeyboardEvent| {
            if event.default_prevented() {
                return;
            }
            match event.key().as_str() {
                "Enter" => {
                    event.prevent_default();
                    save_editor.save();
                }
                "Escape" => {
                    event.prevent_default();
                    cancel_editor.cancel();
                }
                _ => {}
            }
        };
        let cells = untrack(|| self.editing_cells(saved));
        view! {
            <tr node_ref=row class=EDITING_ROW_CLASS data-line-editor="" on:keydown=on_keydown>
                {cells}
            </tr>
            {self.issues_row()}
        }
        .into_any()
    }

    fn add_button(
        &self,
        editable: Signal<bool>,
        new_draft: Arc<dyn Fn() -> Draft + Send + Sync>,
        add_label: String,
    ) -> impl Fn() -> Option<AnyView> + Send + Sync + 'static {
        let editor = self.clone();
        let editing = self.editing;
        move || {
            let adding = editing.get() == Some(EditTarget::New);
            (editable.get() && !adding).then(|| {
                let add_editor = editor.clone();
                let new_draft = Arc::clone(&new_draft);
                view! {
                    <button
                        type="button"
                        class=button_class(ButtonTone::Secondary, ADD_SIZE)
                        on:click=move |_| add_editor.start(EditTarget::New, new_draft())
                    >
                        {icons::icon(icons::LucideGlyph::Plus, button_icon_class(ADD_SIZE))}
                        {add_label.clone()}
                    </button>
                }
                .into_any()
            })
        }
    }

    fn actions_column(
        &self,
        row_key: RowAttribute<Row>,
        draft_of: DraftOf<Row, Draft>,
        on_delete: Option<Callback<String>>,
        editable: Signal<bool>,
    ) -> Column<Row> {
        let editor = self.clone();
        let editing = self.editing;
        Column::actions(move |row: &Row| {
            if !editable.get_untracked() {
                return ().into_any();
            }
            let key = row_key(row);
            let start_editor = editor.clone();
            let line_draft = draft_of(row);
            let edit_key = key.clone();
            let on_edit = Callback::new(move |()| {
                start_editor.start(EditTarget::Existing(edit_key.clone()), line_draft.clone());
            });
            let Some(on_delete) = on_delete else {
                return view! { <RowActions on_edit=on_edit edit_label="Edit Line" /> }.into_any();
            };
            let on_delete = Callback::new(move |()| {
                if editing.get_untracked() == Some(EditTarget::Existing(key.clone())) {
                    editing.set(None);
                }
                on_delete.run(key.clone());
            });
            view! {
                <RowActions
                    on_edit=on_edit
                    edit_label="Edit Line"
                    on_delete=on_delete
                    delete_label="Delete Line"
                    delete_question=DELETE_QUESTION
                />
            }
            .into_any()
        })
        .minimum_width_characters(ACTIONS_WIDTH_CHARACTERS)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a component's props are its arguments; each one is a separate concern of the line table"
)]
#[component]
pub fn LineItems<Row, Draft>(
    #[prop(into)] rows: Signal<Option<Result<Vec<Row>, String>>>,
    columns: Vec<Column<Row>>,
    row_key: RowAttribute<Row>,
    draft_of: DraftOf<Row, Draft>,
    new_draft: Arc<dyn Fn() -> Draft + Send + Sync>,
    form: FormCells<Draft>,
    check: DraftCheck<Draft>,
    #[prop(into)] on_save: Callback<(Option<String>, Draft, LineSave<Draft>)>,
    #[prop(optional, into)] on_delete: Option<Callback<String>>,
    #[prop(into, default = Signal::stored(true))] editable: Signal<bool>,
    #[prop(optional, into)] locked_message: Option<String>,
    #[prop(into, default = "Add Line".to_owned())] add_label: String,
    /// For the lines of a document that is already posted: each line can be
    /// edited but none can be added.
    #[prop(optional)]
    existing_lines_only: bool,
    #[prop(optional, into)] empty_message: Signal<String>,
    #[prop(optional)] sticky_first_column: bool,
    #[prop(optional)] search: Search,
    #[prop(optional, into)] search_placeholder: String,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
    Draft: Clone + PartialEq + Send + Sync + 'static,
{
    let draft = RwSignal::new(new_draft());
    let rejected: RwSignal<Vec<LineIssue<Draft>>> = RwSignal::new(Vec::new());
    let save_state = LineSave {
        editing: RwSignal::new(None),
        saving: RwSignal::new(false),
        error: RwSignal::new(None),
        rejected,
        revealed: RwSignal::new(false),
    };
    let editing = save_state.editing;
    Effect::watch(
        move || draft.track(),
        move |(), _, _| rejected.set(Vec::new()),
        false,
    );
    let issues = Signal::derive(move || {
        let mut found = draft.with(|value| check(value));
        found.extend(rejected.get());
        found
    });
    let touched: RwSignal<HashSet<&'static str>> = RwSignal::new(HashSet::new());
    let calculated_keys: RwSignal<HashSet<&'static str>> = RwSignal::new(HashSet::new());
    let revealed = save_state.revealed;
    let shown_issues = visible_issues(issues, revealed, touched, calculated_keys);
    let without_columns: LineEditor<Row, Draft> = LineEditor {
        columns: Arc::from(Vec::new()),
        editing,
        draft,
        baseline: RwSignal::new(None),
        switching: RwSignal::new(None),
        confirming: RwSignal::new(false),
        save_state,
        issues,
        shown_issues,
        touched,
        calculated_keys,
        form,
        on_save,
    };
    let actions =
        without_columns.actions_column(Arc::clone(&row_key), draft_of, on_delete, editable);
    let columns: Arc<[Column<Row>]> = columns.into_iter().chain([actions]).collect();
    let editor = LineEditor {
        columns: Arc::clone(&columns),
        ..without_columns
    };

    let replacing_editor = editor.clone();
    let replacing_key = Arc::clone(&row_key);
    let row_replacement: RowReplacement<Row> = Arc::new(move |row: &Row| {
        let editing_this_row = editing.get() == Some(EditTarget::Existing(replacing_key(row)));
        editing_this_row.then(|| replacing_editor.editing_rows(Some(row)))
    });
    let new_line_editor = editor.clone();
    let trailing_rows = ViewFn::from(move || {
        (editing.get() == Some(EditTarget::New)).then(|| new_line_editor.editing_rows(None))
    });

    let confirming = editor.confirming;
    let discarding_editor = editor.clone();
    let on_discard = Callback::new(move |()| discarding_editor.discard_and_switch());
    let add_button =
        (!existing_lines_only).then(|| editor.add_button(editable, new_draft, add_label));
    let locked_note = move || {
        (!editable.get())
            .then(|| locked_message.clone())
            .flatten()
            .map(|message| view! { <p class="text-sm text-gray-500">{message}</p> })
    };

    view! {
        <div>
            <DataTable
                rows=rows
                columns=columns.to_vec()
                row_key=row_key
                row_replacement=row_replacement
                trailing_rows=trailing_rows
                empty_message=empty_message
                hides_row_count=true
                sticky_first_column=sticky_first_column
                search=search
                search_placeholder=search_placeholder
            />
            <div class="mt-3 flex items-center gap-3 empty:hidden">{add_button} {locked_note}</div>
            <ConfirmDialog
                open=confirming
                title=DISCARD_TITLE
                body=DISCARD_QUESTION
                confirm_label=DISCARD_LABEL
                tone=ButtonTone::Danger
                on_confirm=on_discard
            />
        </div>
    }
    .into_any()
}

fn visible_issues<Draft: Send + Sync + 'static>(
    issues: Signal<Vec<LineIssue<Draft>>>,
    revealed: RwSignal<bool>,
    touched: RwSignal<HashSet<&'static str>>,
    calculated_keys: RwSignal<HashSet<&'static str>>,
) -> Signal<Vec<LineIssue<Draft>>> {
    Signal::derive(move || {
        let all = issues.get();
        if revealed.get() {
            return all;
        }
        let anything_touched = touched.with(|touched| !touched.is_empty());
        all.into_iter()
            .filter(|issue| {
                touched.with(|touched| touched.contains(issue.field))
                    || (anything_touched && calculated_keys.with(|keys| keys.contains(issue.field)))
            })
            .collect()
    })
}
