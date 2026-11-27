use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::catalog::CardGrid;
use crate::checkbox::{check_button, CheckState};
use crate::empty_state::EmptyState;
use crate::icons;
use crate::loading_state::LoadingState;
use crate::menu::{focus_on_hover, ICON_CLASS, ITEM_CLASS};
use crate::popover::{use_popover, Popover, PopoverAlign, PopoverContent, PopoverTrigger};
use crate::tabs::PageView;
use crate::tooltip::Tooltip;

use super::calendar_view::CalendarView;
use super::cell_value::{Align, CellValue};
use super::column::Column;
use super::column_choices::ColumnChoices;
use super::facet_row::FacetRow;
use super::layout::{Layout, ServerFiltering, ServerRows};
use super::row_selection::RowSelection;
use super::server_facet_row::{FacetColumn, ServerFacetRow};
use super::toolbar::TableToolbar;
use super::url_query::UrlTableQuery;
use table_query::TableQuery;

pub type RowAttribute<Row> = Arc<dyn Fn(&Row) -> String + Send + Sync>;
pub type RowView<Row> = Arc<dyn Fn(&Row) -> AnyView + Send + Sync>;
pub type RowReplacement<Row> = Arc<dyn Fn(&Row) -> Option<AnyView> + Send + Sync>;
pub(super) type TableRows<Row> = Signal<Option<Result<Vec<Row>, String>>>;
pub(super) type ClientColumnFilters = HashMap<usize, HashSet<String>>;

pub(super) fn matches_search<Row: Send + Sync>(
    columns: &[Column<Row>],
    row: &Row,
    search_lower: &str,
) -> bool {
    search_lower.is_empty()
        || columns
            .iter()
            .filter(|column| column.searchable)
            .any(|column| (column.value)(row).search_text().contains(search_lower))
}

pub(super) fn passes_column_filters<Row: Send + Sync>(
    columns: &[Column<Row>],
    filters: &ClientColumnFilters,
    row: &Row,
    ignored_column: Option<usize>,
) -> bool {
    filters
        .iter()
        .filter(|(index, _)| Some(**index) != ignored_column)
        .all(|(index, allowed)| {
            columns
                .get(*index)
                .is_none_or(|column| allowed.contains(&(column.value)(row).filter_key()))
        })
}

#[derive(Clone, Copy, Default)]
pub enum Search {
    #[default]
    Off,
    InMemory,
    OnServer,
}

const NO_ROWS_MESSAGE: &str = "No results.";
const NO_MATCHES_TITLE: &str = "Nothing matches these filters";
const NO_MATCHES_DESCRIPTION: &str = "Try a broader search or reset the filters.";
const LOAD_FAILED_TITLE: &str = "This Data Could Not Be Loaded";
const LOAD_FAILED_DESCRIPTION: &str =
    "Something went wrong while fetching it. Nothing has been changed.";

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyState {
    Loading,
    Failed,
    Empty,
    Rows,
}

fn sort_arrow_glyph(ascending: bool) -> icons::LucideGlyph {
    if ascending {
        icons::LucideGlyph::ArrowUp
    } else {
        icons::LucideGlyph::ArrowDown
    }
}

fn sort_direction_label(ascending: bool) -> &'static str {
    if ascending {
        "Sort Ascending"
    } else {
        "Sort Descending"
    }
}

const SORT_MENU_WIDTH_CLASS: &str = "w-44";
const HEADER_TRIGGER_CLASS: &str = "-mx-1.5 -my-0.5 inline-flex cursor-pointer select-none items-center rounded-md px-1.5 py-0.5 hover:bg-gray-100 aria-expanded:bg-gray-100 focus-ring";
pub(super) const FOOTER_ACTION_CLASS: &str = "cursor-pointer rounded-sm hover:underline focus-ring";
const CLICKABLE_ROW_CLASS: &str = "cursor-pointer hover:bg-row-hover";
const SELECTABLE_LINK_ROW_CLASS: &str =
    "cursor-pointer bg-surface hover:bg-row-hover data-selected:bg-blue-50";
const PICKABLE_ROW_CLASS: &str = "cursor-pointer bg-surface hover:bg-gray-50 data-selected:bg-blue-50 data-selected:hover:bg-blue-100";
const SELECTION_COLUMN_WIDTH: &str = "2.75rem";
const SELECTION_CELL_CLASS: &str = "pl-4 pr-0 text-left leading-none";
const SELECTION_SUMMARY_CLASS: &str = "flex items-center gap-2 text-sm text-gray-500 tabular-nums";
const SUMMARY_DIVIDER_CLASS: &str = "border-l border-edge pl-4";
const SELECT_ROW_LABEL: &str = "Select Row";
const SELECT_EVERY_ROW_LABEL: &str = "Select All Rows";
const FOOTER_CELL_CLASS: &str = "truncate px-4 text-sm font-semibold tabular-nums text-gray-900";
const FOOTER_LABEL: &str = "Total";
const FOOTER_ROW_CLASS: &str =
    "sticky bottom-0 z-10 bg-gray-50 shadow-[inset_0_1px_0_0_var(--color-edge)]";
const ROW_IDENTITY_SEPARATOR: char = '\u{1e}';
const SCROLLER_CLASS: &str = "table-scroller w-full max-w-full overflow-auto rounded-xl border border-edge bg-surface text-sm tabular-nums";
const TABLE_CLASS: &str = "w-full table-fixed";
const CELL_SEPARATOR: &str = "\u{1f}";

#[derive(Clone, Copy)]
enum PageStep {
    Previous,
    Next,
}

impl PageStep {
    fn label(self) -> &'static str {
        match self {
            PageStep::Previous => "Previous Page",
            PageStep::Next => "Next Page",
        }
    }

    fn glyph(self) -> icons::LucideGlyph {
        match self {
            PageStep::Previous => icons::LucideGlyph::ChevronLeft,
            PageStep::Next => icons::LucideGlyph::ChevronRight,
        }
    }
}

fn page_step_button(
    step: PageStep,
    disabled: bool,
    on_step: impl Fn() + Send + Sync + 'static,
) -> AnyView {
    view! {
        <Tooltip label=step.label()>
            <button
                type="button"
                aria-label=step.label()
                class=icon_button_class(ButtonTone::Secondary, ButtonSize::Compact)
                disabled=disabled
                on:click=move |_| on_step()
            >
                {icons::icon(step.glyph(), "h-4 w-4")}
            </button>
        </Tooltip>
    }
    .into_any()
}

fn menu_alignment(align: Align) -> PopoverAlign {
    match align {
        Align::Left => PopoverAlign::Start,
        Align::Right => PopoverAlign::End,
        Align::Center => PopoverAlign::Center,
    }
}

#[cfg(feature = "hydrate")]
fn text_is_being_selected() -> bool {
    web_sys::window()
        .and_then(|window| window.get_selection().ok().flatten())
        .is_some_and(|selection| !selection.is_collapsed())
}

#[cfg(not(feature = "hydrate"))]
fn text_is_being_selected() -> bool {
    false
}

#[cfg(feature = "hydrate")]
fn click_lands_outside_controls(event: &leptos::ev::MouseEvent) -> bool {
    use leptos::wasm_bindgen::JsCast;
    let Some(target) = event.target() else {
        return true;
    };
    let Some(element) = target.dyn_ref::<web_sys::Element>() else {
        return true;
    };
    element
        .closest("a, button, input, select, textarea, label")
        .ok()
        .flatten()
        .is_none()
}

#[cfg(not(feature = "hydrate"))]
fn click_lands_outside_controls(_event: &leptos::ev::MouseEvent) -> bool {
    true
}

fn click_asks_for_row_navigation(event: &leptos::ev::MouseEvent) -> bool {
    let modified = event.ctrl_key() || event.meta_key() || event.shift_key() || event.alt_key();
    !modified && !text_is_being_selected() && click_lands_outside_controls(event)
}

fn click_asks_for_row_selection(event: &leptos::ev::MouseEvent) -> bool {
    let modified = event.ctrl_key() || event.meta_key() || event.alt_key();
    let dragged_over_text = !event.shift_key() && text_is_being_selected();
    !modified && !dragged_over_text && click_lands_outside_controls(event)
}

const CELL_PADDING_WIDTH: &str = "2rem";
const REFRESH_COVER_CLASS: &str = "absolute z-20 items-center justify-center bg-gray-50";

pub const BODY_HEIGHT_CLASS: &str = "h-[calc(100vh-18rem)]";
pub(crate) const BODY_MAX_HEIGHT_CLASS: &str = "max-h-[calc(100vh-18rem)]";
pub(crate) const HEADER_RULE_CLASS: &str = "shadow-[inset_0_-1px_0_0_var(--color-edge)]";
pub(crate) const HEADER_ROW_HEIGHT_CLASS: &str = "h-9";
pub(crate) const ROW_HEIGHT_CLASS: &str = "h-10";

pub(super) fn error_alert(message: String) -> AnyView {
    view! {
        <div
            role="alert"
            class="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
        >
            <strong>"Error: "</strong>
            {message}
        </div>
    }
    .into_any()
}

fn pending_setter_outside_server_rendering(
    set_refreshing: WriteSignal<bool>,
) -> Option<SignalSetter<bool>> {
    cfg!(not(feature = "ssr")).then(|| set_refreshing.into())
}

pub(super) fn refresh_cover(pending: Signal<bool>, shape_class: &'static str) -> AnyView {
    view! {
        <div
            role="status"
            aria-label="Loading"
            class=move || {
                let visibility = if is_pending(pending) { "flex" } else { "hidden" };
                format!("{REFRESH_COVER_CLASS} {shape_class} {visibility}")
            }
        >
            {icons::spinner("h-6 w-6 text-gray-400")}
        </div>
    }
    .into_any()
}

#[component]
pub fn DataTable<Row>(
    #[prop(into)] rows: Signal<Option<Result<Vec<Row>, String>>>,
    columns: Vec<Column<Row>>,
    #[prop(optional)] search: Search,
    #[prop(optional, into)] search_placeholder: String,
    #[prop(optional, into)] empty_message: Signal<String>,
    #[prop(optional)] row_class: Option<RowAttribute<Row>>,
    #[prop(optional)] row_href: Option<RowAttribute<Row>>,
    #[prop(optional)] row_key: Option<RowAttribute<Row>>,
    #[prop(optional)] selection: Option<RowSelection<Row>>,
    #[prop(optional)] row_replacement: Option<RowReplacement<Row>>,
    #[prop(optional)] trailing_rows: Option<ViewFn>,
    #[prop(optional)] hides_row_count: bool,
    /// Asks for the rows again after they failed to load. With it, the
    /// failure state offers Try Again.
    #[prop(optional, into)]
    on_retry: Option<Callback<()>>,
    #[prop(optional_no_strip)] toolbar: Option<AnyView>,
    #[prop(optional_no_strip)] filter_toggle: Option<AnyView>,
    /// The page's own controls that narrow the rows, such as a date, drawn
    /// beside the search.
    #[prop(optional_no_strip)]
    filters: Option<AnyView>,
    #[prop(optional_no_strip)] views: Option<&'static [PageView]>,
    #[prop(optional_no_strip)] sidebar: Option<AnyView>,
    #[prop(optional)] server: Option<ServerRows>,
    #[prop(optional)] layout: Layout<Row>,
    #[prop(optional)] sticky_first_column: bool,
    #[prop(optional, into)]
    column_choices: MaybeProp<String>,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
{
    let query = server.map(|server| server.query);
    let total_rows = server.map_or_else(Signal::default, |server| server.total_rows);
    let filtering = server.and_then(|server| server.filtering);
    let calendar = server.and_then(|server| {
        server
            .calendar_date_column_key
            .map(|date_column_key| (date_column_key, server.query))
    });
    let row_key = row_key.or_else(|| selection.as_ref().map(RowSelection::row_key));
    let (refreshing, set_refreshing) = signal(false);
    let search_text = ArcRwSignal::new(
        query
            .map(|query| query.with_untracked(|query| query.search.clone()))
            .unwrap_or_default(),
    );
    let columns: Arc<[Column<Row>]> = columns.into();
    let column_filters: ArcRwSignal<ClientColumnFilters> = ArcRwSignal::new(HashMap::new());
    let facets = match (query, filtering) {
        (Some(query), Some(filtering)) => server_facet_row(&columns, query, filtering).render(),
        (Some(_), None) => None,
        (None, _) => FacetRow {
            rows,
            columns: Arc::clone(&columns),
            column_filters: column_filters.clone(),
            search_text: search_text.clone(),
        }
        .render(),
    };
    let column_choices = column_choices
        .get_untracked()
        .map(|table_key| ColumnChoices::new(&table_key, Arc::clone(&columns)));
    let controls = match (toolbar, column_choices.as_ref().map(ColumnChoices::menu)) {
        (Some(toolbar), Some(menu)) => Some(
            view! {
                {toolbar}
                {menu}
            }
            .into_any(),
        ),
        (toolbar, menu) => toolbar.or(menu),
    };
    let toolbar_row = TableToolbar {
        views,
        filter_toggle,
        facets,
        filters,
        controls,
        search,
        search_placeholder,
        query,
        search_text: RwSignal::from(search_text.clone()),
        pending: refreshing.into(),
    }
    .render();
    let navigate = row_href
        .as_ref()
        .map(|_| -> Navigate { Arc::new(use_navigate()) });
    let state = TableState {
        rows,
        columns,
        search_text,
        column_filters,
        sort_state: ArcRwSignal::new(None),
        server_query: query,
        row_class,
        row_href,
        navigate,
        selection,
        row_replacement,
        hides_row_count,
        on_retry,
        sticky_first_column,
        column_choices,
    };
    let contents = move || {
        view! {
            <TableContents
                state
                empty_message
                row_key
                sidebar
                total_rows
                pending=refreshing.into()
                trailing_rows
                layout
                calendar
            />
        }
    };
    let body = Transition(TransitionProps {
        fallback: ViewFnOnce::from(|| view! { <LoadingState class=BODY_HEIGHT_CLASS /> }),
        set_pending: pending_setter_outside_server_rendering(set_refreshing),
        children: TypedChildren::to_children(contents),
    });
    view! { <div>{toolbar_row} {body}</div> }.into_any()
}

type Navigate = Arc<dyn Fn(&str, NavigateOptions) + Send + Sync>;

fn server_facet_row<Row: Send + Sync + 'static>(
    columns: &[Column<Row>],
    query: UrlTableQuery,
    filtering: ServerFiltering,
) -> ServerFacetRow {
    ServerFacetRow {
        columns: columns
            .iter()
            .filter(|column| column.filterable)
            .map(|column| FacetColumn {
                key: column.key,
                header: column.header,
            })
            .collect(),
        query,
        filtering,
    }
}

fn compare_cells(left: &CellValue, right: &CellValue, ascending: bool) -> Ordering {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) if ascending => left.compare(right),
        (false, false) => left.compare(right).reverse(),
    }
}

fn sorted_rows<Row: Send + Sync>(
    columns: &[Column<Row>],
    rows: Vec<Row>,
    sort: Option<(usize, bool)>,
) -> Vec<Row> {
    let Some((column, ascending)) =
        sort.and_then(|(index, ascending)| columns.get(index).map(|column| (column, ascending)))
    else {
        return rows;
    };
    let mut keyed: Vec<(CellValue, Row)> = rows
        .into_iter()
        .map(|row| ((column.value)(&row), row))
        .collect();
    keyed.sort_by(|(left, _), (right, _)| compare_cells(left, right, ascending));
    keyed.into_iter().map(|(_, row)| row).collect()
}

struct TableState<Row: Send + Sync + 'static> {
    rows: TableRows<Row>,
    columns: Arc<[Column<Row>]>,
    search_text: ArcRwSignal<String>,
    column_filters: ArcRwSignal<ClientColumnFilters>,
    sort_state: ArcRwSignal<Option<(usize, bool)>>,
    server_query: Option<UrlTableQuery>,
    row_class: Option<RowAttribute<Row>>,
    row_href: Option<RowAttribute<Row>>,
    navigate: Option<Navigate>,
    selection: Option<RowSelection<Row>>,
    row_replacement: Option<RowReplacement<Row>>,
    hides_row_count: bool,
    on_retry: Option<Callback<()>>,
    sticky_first_column: bool,
    column_choices: Option<ColumnChoices<Row>>,
}

impl<Row: Send + Sync + 'static> Clone for TableState<Row> {
    fn clone(&self) -> Self {
        Self {
            rows: self.rows,
            columns: Arc::clone(&self.columns),
            search_text: self.search_text.clone(),
            column_filters: self.column_filters.clone(),
            sort_state: self.sort_state.clone(),
            server_query: self.server_query,
            row_class: self.row_class.clone(),
            row_href: self.row_href.clone(),
            navigate: self.navigate.clone(),
            selection: self.selection.clone(),
            row_replacement: self.row_replacement.clone(),
            hides_row_count: self.hides_row_count,
            on_retry: self.on_retry,
            sticky_first_column: self.sticky_first_column,
            column_choices: self.column_choices.clone(),
        }
    }
}

impl<Row: Clone + Send + Sync + 'static> TableState<Row> {
    fn current_rows(&self) -> Option<Result<Vec<Row>, String>> {
        self.rows.try_get().flatten()
    }

    fn server_driven(&self) -> bool {
        self.server_query.is_some()
    }

    fn column_key_at(&self, index: usize) -> Option<&'static str> {
        self.columns.get(index).map(|column| column.key)
    }

    fn displayed_sort(&self) -> Option<(usize, bool)> {
        let Some(query) = self.server_query else {
            return self.sort_state.get();
        };
        query.with(|query| {
            let sort_key = query.sort_column_key.as_deref()?;
            let index = self
                .columns
                .iter()
                .position(|column| column.key == sort_key)?;
            Some((index, query.sort_ascending))
        })
    }

    fn filters_are_active(&self) -> bool {
        match self.server_query {
            Some(query) => query.with(|query| query.filtered_column_count() > 0),
            None => self.column_filters.with(|active| !active.is_empty()),
        }
    }

    fn search_is_active(&self) -> bool {
        match self.server_query {
            Some(query) => query.with(|query| !query.search.is_empty()),
            None => self.search_text.with(|search| !search.is_empty()),
        }
    }

    fn reset_filters(&self) {
        self.search_text.set(String::new());
        self.column_filters.set(HashMap::new());
        if let Some(query) = self.server_query {
            query.update(|query| {
                query.set_search(String::new());
                query.clear_all_column_filters();
            });
        }
    }

    fn visible_rows(&self) -> Vec<Row> {
        let Some(Ok(all_rows)) = self.current_rows() else {
            return Vec::new();
        };
        if self.server_driven() {
            return all_rows;
        }
        let search_text = self.search_text.get().to_lowercase();
        let active_filters = self.column_filters.get();
        let matching: Vec<Row> = all_rows
            .into_iter()
            .filter(|row| {
                matches_search(&self.columns, row, &search_text)
                    && passes_column_filters(&self.columns, &active_filters, row, None)
            })
            .collect();
        sorted_rows(&self.columns, matching, self.sort_state.get())
    }

    fn card_body(&self, card_for_row: RowView<Row>, pending: Signal<bool>) -> AnyView {
        let state = self.clone();
        let cards = move || {
            state
                .visible_rows()
                .iter()
                .map(|row| card_for_row(row))
                .collect_view()
        };
        view! {
            <div class="relative">
                <div
                    class=format!("overflow-auto {BODY_MAX_HEIGHT_CLASS}")
                    aria-busy=move || is_pending(pending).to_string()
                >
                    <CardGrid>{cards}</CardGrid>
                </div>
                {refresh_cover(pending, "inset-0 rounded-xl")}
            </div>
        }
        .into_any()
    }

    fn render_row(&self, row: &Row) -> AnyView {
        let Some(replacement) = self.row_replacement.clone() else {
            return self.render_table_row(row);
        };
        let state = self.clone();
        let row = row.clone();
        (move || replacement(&row).unwrap_or_else(|| state.render_table_row(&row))).into_any()
    }

    fn keys_in_view(&self, selection: &RowSelection<Row>) -> Vec<String> {
        self.visible_rows()
            .iter()
            .map(|row| selection.key_of(row))
            .collect()
    }

    fn selection_header_cell(&self, selection: &RowSelection<Row>) -> AnyView {
        let class = format!("{HEADER_ROW_HEIGHT_CLASS} {SELECTION_CELL_CLASS}");
        if !selection.picks_many() {
            return view! { <th class=class></th> }.into_any();
        }
        let reading_state = self.clone();
        let reading_selection = selection.clone();
        let rows_in_view = Signal::derive(move || {
            reading_selection.state_of_rows_in_view(&reading_state.keys_in_view(&reading_selection))
        });
        let state = self.clone();
        let selection = selection.clone();
        let toggle_rows_in_view =
            move |_| selection.toggle_rows_in_view(&state.keys_in_view(&selection));
        view! {
            <th class=class>
                {check_button(rows_in_view, SELECT_EVERY_ROW_LABEL, toggle_rows_in_view)}
            </th>
        }
        .into_any()
    }

    fn shown_columns(&self) -> Vec<(usize, &Column<Row>)> {
        let shown = match &self.column_choices {
            Some(choices) => choices.shown_indices(),
            None => (0..self.columns.len()).collect(),
        };
        shown
            .into_iter()
            .filter_map(|index| self.columns.get(index).map(|column| (index, column)))
            .collect()
    }

    fn row_cells(&self, row: &Row) -> AnyView {
        let state = self.clone();
        let row = row.clone();
        (move || {
            state
                .shown_columns()
                .into_iter()
                .map(|(_, column)| {
                    let class = format!(
                        "{ROW_HEIGHT_CLASS} px-4 py-2.5 text-gray-700 {} truncate",
                        column.align.text_class()
                    );
                    let title = column.plain_text_of(&row);
                    view! {
                        <td class=class title=title>
                            {(column.render)(&row)}
                        </td>
                    }
                })
                .collect_view()
        })
        .into_any()
    }

    fn row_target(&self, row: &Row) -> Option<String> {
        self.row_href
            .as_ref()
            .map(|href_for_row| href_for_row(row))
            .filter(|target| !target.is_empty())
    }

    fn navigation_on_click(&self, target: String) -> impl Fn(leptos::ev::MouseEvent) + 'static {
        let navigate = self.navigate.clone();
        move |event: leptos::ev::MouseEvent| {
            if !click_asks_for_row_navigation(&event) {
                return;
            }
            if let Some(navigate) = navigate.as_ref() {
                navigate(&target, NavigateOptions::default());
            }
        }
    }

    fn render_table_row(&self, row: &Row) -> AnyView {
        let cells = self.row_cells(row);
        let extra = self
            .row_class
            .as_ref()
            .map(|class_for_row| class_for_row(row))
            .unwrap_or_default();
        let target = self.row_target(row);
        if let Some(selection) = self.selection.clone() {
            return self.selectable_row(selection, row, cells, &extra, target);
        }
        let Some(target) = target else {
            return view! { <tr class=extra>{cells}</tr> }.into_any();
        };
        view! {
            <tr
                class=format!("{CLICKABLE_ROW_CLASS} {extra}")
                data-row-link=""
                on:click=self.navigation_on_click(target)
            >
                {cells}
            </tr>
        }
        .into_any()
    }

    fn selectable_row(
        &self,
        selection: RowSelection<Row>,
        row: &Row,
        cells: AnyView,
        extra_class: &str,
        target: Option<String>,
    ) -> AnyView {
        let key = selection.key_of(row);
        let is_selected = {
            let selection = selection.clone();
            let key = key.clone();
            Signal::derive(move || selection.contains(&key))
        };
        let picks_many = selection.picks_many();
        let pick = {
            let state = self.clone();
            move |event: &leptos::ev::MouseEvent| {
                let keys_in_view = state.keys_in_view(&selection);
                selection.pick(&key, event.shift_key(), &keys_in_view);
            }
        };
        let pick_from_checkbox = pick.clone();
        let checkbox = view! {
            <td class=format!(
                "{ROW_HEIGHT_CLASS} {SELECTION_CELL_CLASS}",
            )>
                {check_button(
                    Signal::derive(move || CheckState::of(is_selected.get())),
                    SELECT_ROW_LABEL,
                    move |event| pick_from_checkbox(&event),
                )}
            </td>
        };
        let selected_marker = move || is_selected.get().then_some("");
        let Some(target) = target else {
            return view! {
                <tr
                    class=format!("{PICKABLE_ROW_CLASS} {extra_class}")
                    data-selectable-row=""
                    data-selected=selected_marker
                    on:mousedown=move |event: leptos::ev::MouseEvent| {
                        if picks_many && event.shift_key() {
                            event.prevent_default();
                        }
                    }
                    on:click=move |event: leptos::ev::MouseEvent| {
                        if click_asks_for_row_selection(&event) {
                            pick(&event);
                        }
                    }
                >
                    {checkbox}
                    {cells}
                </tr>
            }
            .into_any();
        };
        view! {
            <tr
                class=format!("{SELECTABLE_LINK_ROW_CLASS} {extra_class}")
                data-row-link=""
                data-selectable-row=""
                data-selected=selected_marker
                on:click=self.navigation_on_click(target)
            >
                {checkbox}
                {cells}
            </tr>
        }
        .into_any()
    }

    fn row_identity(&self, key_for_row: &RowAttribute<Row>, row: &Row) -> String {
        let contents: Vec<String> = self
            .columns
            .iter()
            .map(|column| (column.value)(row).filter_key())
            .collect();
        format!(
            "{}{ROW_IDENTITY_SEPARATOR}{}",
            key_for_row(row),
            contents.join(CELL_SEPARATOR)
        )
    }

    fn data_rows(&self, row_key: Option<RowAttribute<Row>>) -> AnyView {
        let state = self.clone();
        let Some(key_for_row) = row_key else {
            return (move || {
                state
                    .visible_rows()
                    .iter()
                    .map(|row| state.render_row(row))
                    .collect_view()
            })
            .into_any();
        };
        let each_state = self.clone();
        let key_state = self.clone();
        view! {
            <For
                each=move || each_state.visible_rows()
                key=move |row| key_state.row_identity(&key_for_row, row)
                children=move |row: Row| state.render_row(&row)
            />
        }
        .into_any()
    }

    fn sticky_column_width(&self) -> Option<String> {
        let first = self.columns.first().filter(|_| self.sticky_first_column)?;
        let first_column_width = format!(
            "calc({}ch + {CELL_PADDING_WIDTH})",
            first.rendered_width_characters()
        );
        Some(if self.selection.is_some() {
            format!(
                "--selection-column-width: {SELECTION_COLUMN_WIDTH}; --sticky-column-width: calc({first_column_width} + {SELECTION_COLUMN_WIDTH})"
            )
        } else {
            format!("--sticky-column-width: {first_column_width}")
        })
    }

    fn column_widths(&self) -> AnyView {
        let selection_column = self
            .selection
            .is_some()
            .then(|| view! { <col style=format!("width: {SELECTION_COLUMN_WIDTH}") /> });
        let state = self.clone();
        let columns = move || {
            state
                .shown_columns()
                .into_iter()
                .map(|(_, column)| {
                    let style = format!(
                        "width: calc({}ch + {CELL_PADDING_WIDTH})",
                        column.rendered_width_characters()
                    );
                    view! { <col style=style /> }
                })
                .collect_view()
        };
        view! {
            {selection_column}
            {columns}
        }
        .into_any()
    }

    fn selection_summary(&self, follows_row_count: bool) -> Option<AnyView> {
        let selection = self.selection.clone()?;
        let selected_count = selection.selected_count();
        if selected_count == 0 {
            return None;
        }
        let class = if follows_row_count {
            format!("{SELECTION_SUMMARY_CLASS} {SUMMARY_DIVIDER_CLASS}")
        } else {
            SELECTION_SUMMARY_CLASS.to_owned()
        };
        Some(
            view! {
                <span class=class>
                    {format!("{selected_count} selected")}
                    <button
                        type="button"
                        class=format!("text-blue-600 {FOOTER_ACTION_CLASS}")
                        on:click=move |_| selection.clear()
                    >
                        "Clear"
                    </button>
                </span>
            }
            .into_any(),
        )
    }

    fn row_count_summary(&self) -> Option<String> {
        if self.hides_row_count {
            return None;
        }
        let Some(Ok(all_rows)) = self.current_rows() else {
            return None;
        };
        let shown = self.visible_rows().len();
        if shown == 0 {
            return None;
        }
        let total = all_rows.len();
        Some(if shown == total {
            format!("Showing {total}")
        } else {
            format!("Showing {shown} of {total}")
        })
    }

    fn client_pager(&self) -> AnyView {
        let row_count = self.row_count_summary();
        let selection_summary = self.selection_summary(row_count.is_some());
        if row_count.is_none() && selection_summary.is_none() {
            return ().into_any();
        }
        let row_count = row_count.map(
            |summary| view! { <span class="text-sm text-gray-500 tabular-nums">{summary}</span> },
        );
        view! { <div class="mt-4 flex items-center gap-4">{row_count} {selection_summary}</div> }
            .into_any()
    }

    fn server_pager(&self, query: UrlTableQuery, total_rows: Signal<Option<u64>>) -> AnyView {
        let rows_on_page = match self.current_rows() {
            Some(Ok(page)) => page.len(),
            _ => return ().into_any(),
        };
        let current = query.get();
        let total = total_rows.try_get().flatten();
        let first = current.first_row_number(rows_on_page);
        let last = current.last_row_number(rows_on_page);
        let range = match total {
            Some(total) => format!("{first}\u{2013}{last} of {total}"),
            None => format!("{first}\u{2013}{last}"),
        };
        let on_first_page = current.is_on_first_page();
        let on_last_page = match total {
            Some(total) => u64::from(last) >= total,
            None => rows_on_page < usize::try_from(current.rows_per_page).unwrap_or(usize::MAX),
        };
        let previous_page = move || query.update(TableQuery::return_to_previous_page);
        let next_page = move || query.update(TableQuery::advance_to_next_page);
        view! {
            <div class="mt-4 flex items-center justify-between gap-4">
                <div class="flex items-center gap-4">
                    <span class="text-sm text-gray-500 tabular-nums">{range}</span>
                    {self.selection_summary(true)}
                </div>
                <div class="flex items-center gap-2">
                    {page_step_button(PageStep::Previous, on_first_page, previous_page)}
                    {page_step_button(PageStep::Next, on_last_page, next_page)}
                </div>
            </div>
        }
        .into_any()
    }

    fn pager(&self, total_rows: Signal<Option<u64>>) -> AnyView {
        match self.server_query {
            Some(query) => self.server_pager(query, total_rows),
            None => self.client_pager(),
        }
    }

    fn sort_button(&self, index: usize, ascending: bool) -> AnyView {
        let state = self.clone();
        let popover = use_popover();
        view! {
            <button
                class=ITEM_CLASS
                on:mousemove=|event: leptos::ev::MouseEvent| focus_on_hover(&event)
                on:click=move |_| {
                    state.sort_state.set(Some((index, ascending)));
                    let column_key = state.column_key_at(index);
                    if let Some((query, column_key)) = state.server_query.zip(column_key) {
                        query.update(|query| query.set_sort(column_key.to_string(), ascending));
                    }
                    if let Some(popover) = popover {
                        popover.close();
                    }
                }
            >
                {icons::icon(sort_arrow_glyph(ascending), ICON_CLASS)}
                <span class="whitespace-nowrap">{sort_direction_label(ascending)}</span>
            </button>
        }
        .into_any()
    }

    fn sort_menu(&self, index: usize) -> AnyView {
        view! { <div class="p-1">{self.sort_button(index, true)} {self.sort_button(index, false)}</div> }
            .into_any()
    }

    fn header_label(&self, index: usize, column: &Column<Row>) -> AnyView {
        let sort_state = self.clone();
        let sortable = column.sortable;
        view! {
            <span class="inline-flex items-center gap-1 align-middle">
                {column.header}
                {move || match sort_state.displayed_sort() {
                    Some((current, ascending)) if current == index => {
                        icons::labelled_icon(
                            sort_arrow_glyph(ascending),
                            "h-3.5 w-3.5 shrink-0 text-gray-900",
                            sort_direction_label(ascending),
                        )
                    }
                    _ if sortable => {
                        icons::labelled_icon(
                            icons::LucideGlyph::ChevronDown,
                            "h-3.5 w-3.5 shrink-0 text-gray-400",
                            "Sort",
                        )
                    }
                    _ => ().into_any(),
                }}
            </span>
        }
        .into_any()
    }

    fn header_cell(&self, index: usize, column: &Column<Row>) -> AnyView {
        let label = self.header_label(index, column);
        if !column.sortable {
            let class = format!(
                "{HEADER_ROW_HEIGHT_CLASS} truncate px-4 select-none {} {}",
                column.align.text_class(),
                crate::typography::LABEL_CLASS,
            );
            return view! {
                <th class=class title=column.header>
                    {label}
                </th>
            }
            .into_any();
        }
        let menu_state = self.clone();
        let menu = move || menu_state.sort_menu(index);
        view! {
            <th
                class=format!(
                    "{HEADER_ROW_HEIGHT_CLASS} truncate px-4 {}",
                    column.align.text_class(),
                )
                title=column.header
            >
                <Popover align=menu_alignment(column.align)>
                    <PopoverTrigger class=format!(
                        "{HEADER_TRIGGER_CLASS} {}",
                        crate::typography::LABEL_CLASS,
                    )>{label}</PopoverTrigger>
                    <PopoverContent class=SORT_MENU_WIDTH_CLASS unpadded=true>
                        {menu.clone()}
                    </PopoverContent>
                </Popover>
            </th>
        }
        .into_any()
    }

    fn header(&self) -> AnyView {
        let selection_cell = self
            .selection
            .as_ref()
            .map(|selection| self.selection_header_cell(selection));
        let state = self.clone();
        let cells = move || {
            state
                .shown_columns()
                .into_iter()
                .map(|(index, column)| state.header_cell(index, column))
                .collect_view()
        };
        view! {
            {selection_cell}
            {cells}
        }
        .into_any()
    }

    fn empty_state(&self, empty_message: Signal<String>) -> AnyView {
        if !(self.filters_are_active() || self.search_is_active()) {
            let message = empty_message
                .try_get()
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| NO_ROWS_MESSAGE.to_string());
            return view! { <EmptyState title=message /> }.into_any();
        }
        let state = self.clone();
        let reset_button = view! {
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=move |_| state.reset_filters()
            >
                {icons::icon(icons::LucideGlyph::X, "h-4 w-4 shrink-0 opacity-70")}
                "Reset Filters"
            </button>
        };
        view! {
            <EmptyState
                glyph=icons::LucideGlyph::SearchX
                title=NO_MATCHES_TITLE
                description=NO_MATCHES_DESCRIPTION
                action=reset_button.into_any()
            />
        }
        .into_any()
    }

    fn failure_state(&self, pending: Signal<bool>) -> AnyView {
        let reason = match self.current_rows() {
            Some(Err(message)) => message,
            _ => String::new(),
        };
        let retry_button = self.on_retry.map(|on_retry| {
            view! {
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                    prop:disabled=move || is_pending(pending)
                    on:click=move |_| on_retry.run(())
                >
                    {icons::icon(icons::LucideGlyph::RotateCw, "h-4 w-4 shrink-0 opacity-70")}
                    {move || if is_pending(pending) { "Trying Again\u{2026}" } else { "Try Again" }}
                </button>
            }
        });
        let action = view! {
            <div class="flex flex-col items-center gap-3">
                {retry_button} <p class="max-w-prose break-words text-xs text-gray-400">{reason}</p>
            </div>
        };
        view! {
            <div role="alert">
                <EmptyState
                    glyph=icons::LucideGlyph::TriangleAlert
                    title=LOAD_FAILED_TITLE
                    description=LOAD_FAILED_DESCRIPTION
                    action=action.into_any()
                />
            </div>
        }
        .into_any()
    }

    fn footer(&self) -> Option<AnyView> {
        if self.columns.iter().all(|column| column.total.is_none()) {
            return None;
        }
        let state = self.clone();
        let cells = move || {
            let rows = state.visible_rows();
            state
                .shown_columns()
                .into_iter()
                .map(|(index, column)| {
                    let (text, tone) = match &column.total {
                        Some(total) => (total(&rows), FOOTER_CELL_CLASS),
                        None if index == 0 => {
                            (FOOTER_LABEL.to_owned(), crate::typography::LABEL_CLASS)
                        }
                        None => (String::new(), FOOTER_CELL_CLASS),
                    };
                    let class = format!(
                        "{HEADER_ROW_HEIGHT_CLASS} truncate px-4 {tone} {}",
                        column.align.text_class()
                    );
                    let title = text.clone();
                    view! {
                        <td class=class title=title>
                            {text}
                        </td>
                    }
                })
                .collect_view()
        };
        let selection_cell = self.selection.is_some().then(|| view! { <td></td> });
        Some(
            view! {
                <tfoot class=FOOTER_ROW_CLASS>
                    <tr>{selection_cell} {cells}</tr>
                </tfoot>
            }
            .into_any(),
        )
    }

    fn table_body(
        &self,
        row_key: Option<RowAttribute<Row>>,
        pending: Signal<bool>,
        trailing_rows: Option<ViewFn>,
    ) -> AnyView {
        let sticky_column_width = self.sticky_column_width();
        let sticky_marker = sticky_column_width.is_some().then_some("");
        let selectable_marker = self.selection.is_some().then_some("");
        view! {
            <div class="table-frame relative" style=sticky_column_width>
                <div
                    class=format!("{SCROLLER_CLASS} {BODY_MAX_HEIGHT_CLASS}")
                    data-sticky-first-column=sticky_marker
                    data-selectable-rows=selectable_marker
                >
                    <table class=TABLE_CLASS>
                        <colgroup>{self.column_widths()}</colgroup>
                        <thead class=format!("sticky top-0 z-10 bg-gray-50 {HEADER_RULE_CLASS}")>
                            <tr>{self.header()}</tr>
                        </thead>
                        <tbody
                            class="divide-y divide-divider"
                            aria-busy=move || is_pending(pending).to_string()
                        >
                            {self.data_rows(row_key)}
                            {trailing_rows.map(|rows| move || rows.run())}
                        </tbody>
                        {self.footer()}
                    </table>
                </div>
                <div class="table-edges text-sm" aria-hidden="true"></div>
                {refresh_cover(pending, "inset-px rounded-[calc(var(--radius-xl)-1px)]")}
            </div>
        }
        .into_any()
    }
}

pub(super) fn is_pending(pending: Signal<bool>) -> bool {
    pending.try_get().unwrap_or(false)
}

#[component]
fn TableContents<Row>(
    state: TableState<Row>,
    empty_message: Signal<String>,
    row_key: Option<RowAttribute<Row>>,
    sidebar: Option<AnyView>,
    total_rows: Signal<Option<u64>>,
    pending: Signal<bool>,
    layout: Layout<Row>,
    calendar: Option<(&'static str, UrlTableQuery)>,
    trailing_rows: Option<ViewFn>,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
{
    let shows_empty_state = calendar.is_none();
    let body_state_reader = state.clone();
    let body_state = ArcMemo::new(move |_| match body_state_reader.current_rows() {
        None => BodyState::Loading,
        Some(Err(_)) if shows_empty_state => BodyState::Failed,
        Some(Ok(_)) if shows_empty_state && body_state_reader.visible_rows().is_empty() => {
            BodyState::Empty
        }
        Some(_) => BodyState::Rows,
    });
    let placeholder_state = state.clone();
    let placeholder_body_state = body_state.clone();
    let placeholder = move || match placeholder_body_state.get() {
        BodyState::Loading => view! { <LoadingState class=BODY_HEIGHT_CLASS /> }.into_any(),
        BodyState::Failed => placeholder_state.failure_state(pending),
        BodyState::Empty => placeholder_state.empty_state(empty_message),
        BodyState::Rows => ().into_any(),
    };
    let pager_state = state.clone();
    let pager = move || shows_empty_state.then(|| pager_state.pager(total_rows));
    let body = match (calendar, layout) {
        (Some((date_column_key, query)), _) => view! {
            <CalendarView
                rows=state.rows
                columns=state.columns.to_vec()
                date_column_key=date_column_key
                query=query
                total_rows=total_rows
                pending=pending
                row_href=state.row_href.clone()
            />
        }
        .into_any(),
        (None, Layout::Cards(card_for_row)) => state.card_body(card_for_row, pending),
        (None, Layout::Table) => state.table_body(row_key, pending, trailing_rows),
    };
    view! {
        <div>
            <div class="flex items-start gap-6">
                {sidebar}
                <div class="min-w-0 flex-1">
                    {placeholder}
                    <div class:hidden=move || body_state.get() != BodyState::Rows>{body}</div>
                    {pager}
                </div>
            </div>
        </div>
    }
    .into_any()
}
