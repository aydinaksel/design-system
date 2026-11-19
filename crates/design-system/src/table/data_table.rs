use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::catalog::CardGrid;
use crate::empty_state::EmptyState;
use crate::icons;
use crate::loading_state::LoadingState;
use crate::menu::{focus_on_hover, ICON_CLASS, ITEM_CLASS};
use crate::popover::{use_popover, Popover, PopoverAlign, PopoverContent, PopoverTrigger};
use crate::search_input::{SearchInput, SearchSize};
use crate::tabs::PageView;
use crate::tooltip::Tooltip;

use super::calendar_view::CalendarView;
use super::cell_value::{Align, CellValue};
use super::column::Column;
use super::facet_row::FacetRow;
use super::filter_option::FilterOption;
use super::layout::{Layout, ServerRows};
use super::toolbar::TableToolbar;
use super::url_query::UrlTableQuery;
use table_query::TableQuery;
use table_query::{FilterValue, LoadedFilterValues};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyState {
    Loading,
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

const FILTER_MENU_WIDTH_CLASS: &str = "w-64";
const SORT_MENU_WIDTH_CLASS: &str = "w-44";
const HEADER_TRIGGER_CLASS: &str = "-mx-1.5 -my-0.5 inline-flex cursor-pointer select-none items-center rounded-md px-1.5 py-0.5 hover:bg-gray-100 aria-expanded:bg-gray-100 focus-ring";
pub(super) const FOOTER_ACTION_CLASS: &str = "cursor-pointer rounded-sm hover:underline focus-ring";
const CLICKABLE_ROW_CLASS: &str = "cursor-pointer hover:bg-row-hover";
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
fn click_asks_for_row_navigation(event: &leptos::ev::MouseEvent) -> bool {
    use leptos::wasm_bindgen::JsCast;
    if event.ctrl_key() || event.meta_key() || event.shift_key() || event.alt_key() {
        return false;
    }
    let selecting_text = web_sys::window()
        .and_then(|window| window.get_selection().ok().flatten())
        .is_some_and(|selection| !selection.is_collapsed());
    if selecting_text {
        return false;
    }
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
fn click_asks_for_row_navigation(_event: &leptos::ev::MouseEvent) -> bool {
    true
}

#[derive(Clone, Copy)]
struct ServerFilterSection {
    query: UrlTableQuery,
    column_key: &'static str,
    filter_values: Signal<LoadedFilterValues>,
    value_search: RwSignal<String>,
}

fn selected_values(query: UrlTableQuery, column_key: &str) -> Option<BTreeSet<String>> {
    query.with(|query| query.selected_filter_values(column_key).cloned())
}

fn every_offered_value(filter_values: Signal<LoadedFilterValues>) -> BTreeSet<String> {
    match filter_values.get() {
        Some(Ok(values)) => values.into_iter().map(|value| value.value).collect(),
        _ => BTreeSet::new(),
    }
}

const CELL_PADDING_WIDTH: &str = "2rem";
const REFRESH_COVER_CLASS: &str = "absolute z-20 items-center justify-center bg-gray-50";

pub(crate) const BODY_HEIGHT_CLASS: &str = "h-[calc(100vh-18rem)]";
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

fn server_value_row(
    section: ServerFilterSection,
    offered: &FilterValue,
    every_value: &BTreeSet<String>,
) -> AnyView {
    let ServerFilterSection {
        query, column_key, ..
    } = section;
    let value = offered.value.clone();
    let label = offered.label().to_string();
    let row_count = offered.row_count;
    let checked = selected_values(query, column_key).is_none_or(|set| set.contains(&value));
    let every_value = every_value.clone();
    let on_toggle = move |()| {
        let mut selected =
            selected_values(query, column_key).unwrap_or_else(|| every_value.clone());
        if !selected.remove(&value) {
            selected.insert(value.clone());
        }
        query.update(|query| {
            if selected == every_value {
                query.clear_column_filter(column_key);
            } else {
                query.set_column_filter(column_key, selected);
            }
        });
    };
    view! { <FilterOption label=label row_count=row_count checked=checked on_toggle=on_toggle /> }
        .into_any()
}

fn server_filter_section(section: ServerFilterSection) -> AnyView {
    let ServerFilterSection {
        query,
        column_key,
        filter_values,
        value_search,
    } = section;

    let value_list = move || match filter_values.get() {
        None => view! {
            <p class="flex items-center gap-2 px-3 py-2 text-xs text-gray-400">
                {icons::spinner("h-3.5 w-3.5")} "Loading Values\u{2026}"
            </p>
        }
        .into_any(),
        Some(Err(message)) => {
            view! { <p class="px-3 py-2 text-xs text-red-600">{message}</p> }.into_any()
        }
        Some(Ok(offered)) => {
            let every_value = every_offered_value(filter_values);
            let search_lower = value_search.get().to_lowercase();
            offered
                .iter()
                .filter(|value| {
                    search_lower.is_empty() || value.label().to_lowercase().contains(&search_lower)
                })
                .map(|value| server_value_row(section, value, &every_value))
                .collect_view()
                .into_any()
        }
    };

    view! {
        <div class="border-b border-divider p-2.5">
            <SearchInput value=value_search placeholder="Search Values" size=SearchSize::Compact />
        </div>
        <div class="max-h-56 overflow-auto p-1.5 [scrollbar-gutter:stable] [scrollbar-width:thin]">
            {value_list}
        </div>
        <div class="flex items-center justify-between border-t border-divider px-3 py-2.5 text-xs text-blue-600">
            <button
                class=FOOTER_ACTION_CLASS
                on:click=move |_| query.update(|query| query.clear_column_filter(column_key))
            >
                "Select All"
            </button>
            <button
                class=FOOTER_ACTION_CLASS
                on:click=move |_| {
                    query.update(|query| query.deselect_every_value(column_key));
                }
            >
                "Clear"
            </button>
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
    #[prop(optional)] row_replacement: Option<RowReplacement<Row>>,
    #[prop(optional)] trailing_rows: Option<ViewFn>,
    #[prop(optional)] hides_row_count: bool,
    #[prop(optional_no_strip)] toolbar: Option<AnyView>,
    #[prop(optional_no_strip)] filter_toggle: Option<AnyView>,
    #[prop(optional_no_strip)] views: Option<&'static [PageView]>,
    #[prop(optional_no_strip)] sidebar: Option<AnyView>,
    #[prop(optional)] server: Option<ServerRows>,
    #[prop(optional)] layout: Layout<Row>,
    #[prop(optional)] facet_row: bool,
    #[prop(optional)] sticky_first_column: bool,
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
    let (refreshing, set_refreshing) = signal(false);
    let search_text = ArcRwSignal::new(
        query
            .map(|query| query.with_untracked(|query| query.search.clone()))
            .unwrap_or_default(),
    );
    let columns: Arc<[Column<Row>]> = columns.into();
    let column_filters: ArcRwSignal<ClientColumnFilters> = ArcRwSignal::new(HashMap::new());
    let facet_row = facet_row && query.is_none();
    let facets = facet_row
        .then(|| {
            FacetRow {
                rows,
                columns: Arc::clone(&columns),
                column_filters: column_filters.clone(),
                search_text: search_text.clone(),
            }
            .render()
        })
        .flatten();
    let toolbar_row = TableToolbar {
        views,
        filter_toggle,
        facets,
        controls: toolbar,
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
        value_search: ArcRwSignal::new(String::new()),
        server_query: query,
        server_filtering: query.zip(filtering.map(|filtering| filtering.requested_column)),
        filter_values: filtering.map_or_else(Signal::default, |filtering| filtering.values),
        facet_row,
        row_class,
        row_href,
        navigate,
        row_replacement,
        hides_row_count,
        sticky_first_column,
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
    value_search: ArcRwSignal<String>,
    server_query: Option<UrlTableQuery>,
    server_filtering: Option<(UrlTableQuery, RwSignal<Option<String>>)>,
    filter_values: Signal<LoadedFilterValues>,
    facet_row: bool,
    row_class: Option<RowAttribute<Row>>,
    row_href: Option<RowAttribute<Row>>,
    navigate: Option<Navigate>,
    row_replacement: Option<RowReplacement<Row>>,
    hides_row_count: bool,
    sticky_first_column: bool,
}

impl<Row: Send + Sync + 'static> Clone for TableState<Row> {
    fn clone(&self) -> Self {
        Self {
            rows: self.rows,
            columns: Arc::clone(&self.columns),
            search_text: self.search_text.clone(),
            column_filters: self.column_filters.clone(),
            sort_state: self.sort_state.clone(),
            value_search: self.value_search.clone(),
            server_query: self.server_query,
            server_filtering: self.server_filtering,
            filter_values: self.filter_values,
            facet_row: self.facet_row,
            row_class: self.row_class.clone(),
            row_href: self.row_href.clone(),
            navigate: self.navigate.clone(),
            row_replacement: self.row_replacement.clone(),
            hides_row_count: self.hides_row_count,
            sticky_first_column: self.sticky_first_column,
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

    fn header_for_key(&self, column_key: &str) -> &'static str {
        self.columns
            .iter()
            .find(|column| column.key == column_key)
            .map_or("", |column| column.header)
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

    fn is_filterable(&self, column: &Column<Row>) -> bool {
        if self.server_driven() {
            column.filterable && self.server_filtering.is_some()
        } else {
            column.filterable && !self.facet_row
        }
    }

    fn is_filtered(&self, index: usize) -> bool {
        match self.server_filtering {
            Some((query, _)) => self
                .column_key_at(index)
                .is_some_and(|key| query.with(|query| query.is_column_filtered(key))),
            None => self
                .column_filters
                .with(|filters| filters.contains_key(&index)),
        }
    }

    fn filters_are_active(&self) -> bool {
        match self.server_filtering {
            Some((query, _)) => query.with(|query| query.filtered_column_count() > 0),
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

    fn distinct_values(&self, index: usize) -> Vec<(String, String)> {
        let Some(Ok(all_rows)) = self.current_rows() else {
            return Vec::new();
        };
        let Some(column) = self.columns.get(index) else {
            return Vec::new();
        };
        let mut distinct: BTreeMap<String, String> = BTreeMap::new();
        for row in &all_rows {
            let cell = (column.value)(row);
            distinct
                .entry(cell.filter_key())
                .or_insert_with(|| cell.filter_label());
        }
        distinct.into_iter().collect()
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
        let cards = move || match state.current_rows() {
            Some(Err(message)) => {
                view! { <li class="col-span-full">{error_alert(message)}</li> }.into_any()
            }
            _ => state
                .visible_rows()
                .iter()
                .map(|row| card_for_row(row))
                .collect_view()
                .into_any(),
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

    fn render_table_row(&self, row: &Row) -> AnyView {
        let cells = self
            .columns
            .iter()
            .map(|column| {
                let class = format!(
                    "{ROW_HEIGHT_CLASS} px-4 py-2.5 text-gray-700 {} truncate",
                    column.align.text_class()
                );
                view! { <td class=class>{(column.render)(row)}</td> }
            })
            .collect_view();
        let extra = self
            .row_class
            .as_ref()
            .map(|class_for_row| class_for_row(row))
            .unwrap_or_default();
        let target = self
            .row_href
            .as_ref()
            .map(|href_for_row| href_for_row(row))
            .filter(|target| !target.is_empty());
        let Some(target) = target else {
            return view! { <tr class=extra>{cells}</tr> }.into_any();
        };
        let navigate = self.navigate.clone();
        view! {
            <tr
                class=format!("{CLICKABLE_ROW_CLASS} {extra}")
                data-row-link=""
                on:click=move |event: leptos::ev::MouseEvent| {
                    if !click_asks_for_row_navigation(&event) {
                        return;
                    }
                    if let Some(navigate) = navigate.as_ref() {
                        navigate(&target, NavigateOptions::default());
                    }
                }
            >
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

    fn status_rows(&self) -> AnyView {
        let state = self.clone();
        let column_count = self.columns.len().to_string();
        (move || match state.current_rows() {
            Some(Err(message)) => view! {
                <tr>
                    <td colspan=column_count.clone() class="px-4 py-6">
                        {error_alert(message)}
                    </td>
                </tr>
            }
            .into_any(),
            _ => ().into_any(),
        })
        .into_any()
    }

    fn sticky_column_width(&self) -> Option<String> {
        let first = self.columns.first().filter(|_| self.sticky_first_column)?;
        Some(format!(
            "--sticky-column-width: calc({}ch + {CELL_PADDING_WIDTH})",
            first.rendered_width_characters()
        ))
    }

    fn column_widths(&self) -> AnyView {
        self.columns
            .iter()
            .map(|column| {
                let style = format!(
                    "width: calc({}ch + {CELL_PADDING_WIDTH})",
                    column.rendered_width_characters()
                );
                view! { <col style=style /> }
            })
            .collect_view()
            .into_any()
    }

    fn client_pager(&self) -> AnyView {
        if self.hides_row_count {
            return ().into_any();
        }
        let Some(Ok(all_rows)) = self.current_rows() else {
            return ().into_any();
        };
        let shown = self.visible_rows().len();
        if shown == 0 {
            return ().into_any();
        }
        let total = all_rows.len();
        let summary = if shown == total {
            format!("Showing {total}")
        } else {
            format!("Showing {shown} of {total}")
        };
        view! {
            <div class="mt-4 flex">
                <span class="text-sm text-gray-500 tabular-nums">{summary}</span>
            </div>
        }
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
                <span class="text-sm text-gray-500 tabular-nums">{range}</span>
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

    fn server_filter_chips(&self, query: UrlTableQuery) -> AnyView {
        query.with(|current| {
            current
                .column_filters
                .iter()
                .map(|(column_key, values)| {
                    let owned_key = column_key.clone();
                    filter_chip(
                        self.header_for_key(column_key).to_string(),
                        format!("{} selected", values.len()),
                        Arc::new(move || {
                            query.update(|query| query.clear_column_filter(&owned_key));
                        }),
                    )
                })
                .collect_view()
                .into_any()
        })
    }

    fn client_filter_chips(&self) -> AnyView {
        let mut entries: Vec<(usize, usize)> = self.column_filters.with(|active| {
            active
                .iter()
                .map(|(index, allowed)| (*index, allowed.len()))
                .collect()
        });
        entries.sort_by_key(|(index, _)| *index);
        entries
            .into_iter()
            .map(|(index, selected_count)| {
                let header = self.columns.get(index).map_or("", |column| column.header);
                let total = self.distinct_values(index).len();
                let column_filters = self.column_filters.clone();
                filter_chip(
                    header.to_string(),
                    format!("{selected_count} of {total}"),
                    Arc::new(move || {
                        column_filters.update(|filters| {
                            filters.remove(&index);
                        });
                    }),
                )
            })
            .collect_view()
            .into_any()
    }

    fn filter_chips(&self) -> AnyView {
        if self.facet_row || !self.filters_are_active() {
            return ().into_any();
        }
        let chips = match self.server_filtering {
            Some((query, _)) => self.server_filter_chips(query),
            None => self.client_filter_chips(),
        };
        let state = self.clone();
        view! {
            <div class="mb-4 flex flex-wrap items-center gap-2">
                {chips}
                <button
                    class="cursor-pointer text-xs text-gray-500 underline hover:text-gray-700"
                    on:click=move |_| match state.server_filtering {
                        Some((query, _)) => query.update(TableQuery::clear_all_column_filters),
                        None => state.column_filters.set(HashMap::new()),
                    }
                >
                    "Clear All"
                </button>
            </div>
        }
        .into_any()
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

    fn toggle_client_filter_value(&self, index: usize, key: &str, every_key: &HashSet<String>) {
        self.column_filters.update(|filters| {
            let allowed = filters.entry(index).or_insert_with(|| every_key.clone());
            if !allowed.remove(key) {
                allowed.insert(key.to_string());
            }
            if allowed == every_key {
                filters.remove(&index);
            }
        });
    }

    fn client_filter_section(&self, index: usize) -> AnyView {
        let all_values = self.distinct_values(index);
        let every_key: Arc<HashSet<String>> =
            Arc::new(all_values.iter().map(|(key, _)| key.clone()).collect());
        let value_search = self.value_search.clone();
        let state = self.clone();
        let value_list = move || {
            let search_lower = value_search.get().to_lowercase();
            all_values
                .iter()
                .filter(|(_, label)| {
                    search_lower.is_empty() || label.to_lowercase().contains(&search_lower)
                })
                .cloned()
                .map(|(key, label)| {
                    let checked = state.column_filters.with(|filters| {
                        filters
                            .get(&index)
                            .is_none_or(|allowed| allowed.contains(&key))
                    });
                    let toggling_state = state.clone();
                    let every_key = Arc::clone(&every_key);
                    let on_toggle = move |()| {
                        toggling_state.toggle_client_filter_value(index, &key, &every_key);
                    };
                    view! { <FilterOption label=label checked=checked on_toggle=on_toggle /> }
                })
                .collect_view()
        };
        let select_all = self.column_filters.clone();
        let clear = self.column_filters.clone();
        view! {
            <div class="border-b border-divider p-2.5">
                <SearchInput
                    value=RwSignal::from(self.value_search.clone())
                    placeholder="Search Values"
                    size=SearchSize::Compact
                />
            </div>
            <div class="max-h-56 overflow-auto p-1.5 [scrollbar-gutter:stable] [scrollbar-width:thin]">
                {value_list}
            </div>
            <div class="flex items-center justify-between border-t border-divider px-3 py-2.5 text-xs text-blue-600">
                <button
                    class=FOOTER_ACTION_CLASS
                    on:click=move |_| {
                        select_all
                            .update(|filters| {
                                filters.remove(&index);
                            });
                    }
                >
                    "Select All"
                </button>
                <button
                    class=FOOTER_ACTION_CLASS
                    on:click=move |_| {
                        clear
                            .update(|filters| {
                                filters.insert(index, HashSet::new());
                            });
                    }
                >
                    "Clear"
                </button>
            </div>
        }
        .into_any()
    }

    fn column_menu(&self, index: usize) -> AnyView {
        let (sortable, filterable) = self.columns.get(index).map_or((false, false), |column| {
            (column.sortable, self.is_filterable(column))
        });
        let sort_section = sortable.then(|| {
            view! {
                <div class="border-b border-divider p-1">
                    {self.sort_button(index, true)} {self.sort_button(index, false)}
                </div>
            }
        });
        let filter_section = match (filterable, self.server_filtering) {
            (false, _) => ().into_any(),
            (true, Some((query, _))) => self.column_key_at(index).map_or_else(
                || ().into_any(),
                |column_key| {
                    server_filter_section(ServerFilterSection {
                        query,
                        column_key,
                        filter_values: self.filter_values,
                        value_search: RwSignal::from(self.value_search.clone()),
                    })
                },
            ),
            (true, None) => self.client_filter_section(index),
        };
        view! {
            {sort_section}
            {filter_section}
        }
        .into_any()
    }

    fn header_label(&self, index: usize, column: &Column<Row>, interactive: bool) -> AnyView {
        let sort_state = self.clone();
        let indicator_state = self.clone();
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
                    _ => ().into_any(),
                }}
                {move || {
                    if !interactive {
                        return ().into_any();
                    }
                    let filtered = indicator_state.is_filtered(index);
                    let sorted = matches!(
                        indicator_state.displayed_sort(),
                        Some((current, _))
                        if current == index
                    );
                    if sorted && !filtered {
                        return ().into_any();
                    }
                    let (class, title) = if filtered {
                        ("h-3.5 w-3.5 shrink-0 text-blue-600", "Filtered")
                    } else {
                        ("h-3.5 w-3.5 shrink-0 text-gray-400", "Sort and Filter")
                    };
                    icons::labelled_icon(icons::LucideGlyph::ChevronDown, class, title)
                }}
            </span>
        }
        .into_any()
    }

    fn header_cell(&self, index: usize, column: &Column<Row>) -> AnyView {
        let filterable = self.is_filterable(column);
        let interactive = column.sortable || filterable;
        let label = self.header_label(index, column, interactive);
        if !interactive {
            let class = format!(
                "{HEADER_ROW_HEIGHT_CLASS} truncate px-4 select-none {} {}",
                column.align.text_class(),
                crate::typography::LABEL_CLASS,
            );
            return view! { <th class=class>{label}</th> }.into_any();
        }
        let opening_state = self.clone();
        let on_open = Callback::new(move |()| {
            opening_state.value_search.set(String::new());
            if let Some((_, requested_column)) = opening_state.server_filtering {
                let requested = filterable
                    .then(|| opening_state.column_key_at(index))
                    .flatten()
                    .map(str::to_string);
                requested_column.try_set(requested);
            }
        });
        let menu_width_class = if filterable {
            FILTER_MENU_WIDTH_CLASS
        } else {
            SORT_MENU_WIDTH_CLASS
        };
        let menu_state = self.clone();
        let menu = move || menu_state.column_menu(index);
        view! {
            <th class=format!(
                "{HEADER_ROW_HEIGHT_CLASS} truncate px-4 {}",
                column.align.text_class(),
            )>
                <Popover align=menu_alignment(column.align) on_open=on_open>
                    <PopoverTrigger class=format!(
                        "{HEADER_TRIGGER_CLASS} {}",
                        crate::typography::LABEL_CLASS,
                    )>{label}</PopoverTrigger>
                    <PopoverContent class=menu_width_class unpadded=true>
                        {menu.clone()}
                    </PopoverContent>
                </Popover>
            </th>
        }
        .into_any()
    }

    fn header(&self) -> AnyView {
        self.columns
            .iter()
            .enumerate()
            .map(|(index, column)| self.header_cell(index, column))
            .collect_view()
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

    fn footer(&self) -> Option<AnyView> {
        if self.columns.iter().all(|column| column.total.is_none()) {
            return None;
        }
        let state = self.clone();
        let cells = move || {
            let rows = state.visible_rows();
            state
                .columns
                .iter()
                .enumerate()
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
                    view! { <td class=class>{text}</td> }
                })
                .collect_view()
        };
        Some(
            view! {
                <tfoot class=FOOTER_ROW_CLASS>
                    <tr>{cells}</tr>
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
        view! {
            <div class="relative">
                <div
                    class=format!("{SCROLLER_CLASS} {BODY_MAX_HEIGHT_CLASS}")
                    data-sticky-first-column=sticky_marker
                    style=sticky_column_width
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
                            {self.status_rows()}
                            {self.data_rows(row_key)}
                            {trailing_rows.map(|rows| move || rows.run())}
                        </tbody>
                        {self.footer()}
                    </table>
                    <div class="table-edges" aria-hidden="true"></div>
                </div>
                {refresh_cover(pending, "inset-px rounded-[calc(var(--radius-xl)-1px)]")}
            </div>
        }
        .into_any()
    }
}

fn filter_chip(label: String, summary: String, clear: Arc<dyn Fn() + Send + Sync>) -> AnyView {
    view! {
        <span class="inline-flex items-center gap-1 rounded-full border border-blue-200 bg-blue-50 px-2.5 py-1 text-xs text-blue-700">
            <span class="font-medium">{label}": "</span>
            <span>{summary}</span>
            <button
                class="ml-0.5 cursor-pointer text-blue-500 hover:text-blue-800"
                on:click=move |_| clear()
            >
                "\u{00D7}"
            </button>
        </span>
    }
    .into_any()
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
        Some(Ok(_)) if shows_empty_state && body_state_reader.visible_rows().is_empty() => {
            BodyState::Empty
        }
        Some(_) => BodyState::Rows,
    });
    let placeholder_state = state.clone();
    let placeholder_body_state = body_state.clone();
    let placeholder = move || match placeholder_body_state.get() {
        BodyState::Loading => view! { <LoadingState class=BODY_HEIGHT_CLASS /> }.into_any(),
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
    let chips_state = state.clone();
    view! {
        <div>
            {move || chips_state.filter_chips()}
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
