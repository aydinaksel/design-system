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

use super::cell_value::{Align, CellValue};
use super::column::Column;
use super::calendar_view::CalendarView;
use super::filter_option::FilterOption;
use super::layout::Layout;
use super::toolbar::TableToolbar;
use super::url_query::UrlTableQuery;
use table_query::TableQuery;
use table_query::{FilterValue, LoadedFilterValues};

pub type RowAttribute<Row> = Arc<dyn Fn(&Row) -> String + Send + Sync>;
pub type RowView<Row> = Arc<dyn Fn(&Row) -> AnyView + Send + Sync>;

#[derive(Clone, Copy, Default)]
pub enum Search {
    #[default]
    Off,
    /// Filter the in-browser dataset across all `searchable` columns.
    Client,
    /// Publish the search term to the table's `query` for the server to answer.
    Server,
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

/// Generic table: absorbs loading / error / empty states, a sticky
/// header, client-side search, click-to-sort, and Excel-style per-column value
/// filters. Cells stay custom via each `Column`'s `render`.
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
const FOOTER_ACTION_CLASS: &str = "cursor-pointer rounded-sm hover:underline focus-ring";
const CLICKABLE_ROW_CLASS: &str = "cursor-pointer hover:bg-row-hover";

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
pub(crate) const HEADER_RULE_CLASS: &str = "shadow-[inset_0_-1px_0_0_rgb(229,231,235)]";

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
                let visibility = if pending.get() { "flex" } else { "hidden" };
                format!("{REFRESH_COVER_CLASS} {shape_class} {visibility}")
            }
        >
            {icons::spinner("h-6 w-6 text-gray-400")}
        </div>
    }
    .into_any()
}

fn card_body<Row>(
    rows: Signal<Option<Result<Vec<Row>, String>>>,
    visible_rows: Signal<Vec<Row>>,
    card_for_row: RowView<Row>,
    pending: Signal<bool>,
) -> AnyView
where
    Row: Clone + Send + Sync + 'static,
{
    let cards = move || match rows.get() {
        Some(Err(message)) => {
            view! { <li class="col-span-full">{error_alert(message)}</li> }.into_any()
        }
        _ => visible_rows
            .get()
            .iter()
            .map(|row| card_for_row(row))
            .collect_view()
            .into_any(),
    };
    view! {
        <div class="relative">
            <div
                class=format!("overflow-auto {BODY_MAX_HEIGHT_CLASS}")
                aria-busy=move || pending.get().to_string()
            >
                <CardGrid>{cards}</CardGrid>
            </div>
            {refresh_cover(pending, "inset-0 rounded-surface")}
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
        <div class="border-b border-gray-100 p-2.5">
            <SearchInput value=value_search placeholder="Search Values" size=SearchSize::Compact />
        </div>
        <div class="max-h-56 overflow-auto p-1.5 [scrollbar-gutter:stable] [scrollbar-width:thin]">
            {value_list}
        </div>
        <div class="flex items-center justify-between border-t border-gray-100 px-3 py-2.5 text-xs text-blue-600">
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
    #[prop(optional, into)] empty_message: String,
    #[prop(optional)] row_class: Option<RowAttribute<Row>>,
    #[prop(optional)] row_href: Option<RowAttribute<Row>>,
    /// Stable row identity; sorts and filters then reorder DOM rows instead of rebuilding them.
    #[prop(optional)]
    row_key: Option<RowAttribute<Row>>,
    /// Controls that change how the table is drawn, such as a mode switch,
    /// rendered after the search. Page actions belong in the page header.
    #[prop(optional_no_strip)]
    toolbar: Option<AnyView>,
    /// Opens the filters, rendered before the search.
    #[prop(optional_no_strip)]
    filter_toggle: Option<AnyView>,
    /// Tabs that pick a subset of the rows, sharing the search row.
    #[prop(optional_no_strip)]
    views: Option<&'static [PageView]>,
    /// Rendered beside the table body, below the toolbar, such as a facet rail.
    #[prop(optional_no_strip)]
    sidebar: Option<AnyView>,
    /// Switches the table to server-driven mode, reading `rows` as one page.
    #[prop(optional)]
    query: Option<UrlTableQuery>,
    /// Rows matching the query in full. Server-driven mode only.
    #[prop(optional, into)]
    total_rows: Signal<Option<u64>>,
    /// The column key whose values the page should load, written when a filter
    /// popover opens. Enables `filterable` columns in server-driven mode.
    #[prop(optional_no_strip)]
    filter_column_request: Option<RwSignal<Option<String>>>,
    /// The answer to `filter_column_request`. Server-driven mode only.
    #[prop(optional, into)]
    filter_values: Signal<LoadedFilterValues>,
    #[prop(optional)] layout: Layout<Row>,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
{
    let (refreshing, set_refreshing) = signal(false);
    let search_text = RwSignal::new(
        query
            .map(|query| query.with_untracked(|query| query.search.clone()))
            .unwrap_or_default(),
    );
    let toolbar_row = TableToolbar {
        views,
        filter_toggle,
        controls: toolbar,
        search,
        search_placeholder,
        query,
        search_text,
        pending: refreshing.into(),
    }
    .render();
    let contents = move || {
        view! {
            <TableContents
                rows
                columns
                filter=search_text
                empty_message
                row_class
                row_href
                row_key
                sidebar
                query
                total_rows
                pending=refreshing.into()
                filter_column_request
                filter_values
                layout
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

#[component]
fn TableContents<Row>(
    rows: Signal<Option<Result<Vec<Row>, String>>>,
    columns: Vec<Column<Row>>,
    filter: RwSignal<String>,
    empty_message: String,
    row_class: Option<RowAttribute<Row>>,
    row_href: Option<RowAttribute<Row>>,
    row_key: Option<RowAttribute<Row>>,
    sidebar: Option<AnyView>,
    query: Option<UrlTableQuery>,
    total_rows: Signal<Option<u64>>,
    pending: Signal<bool>,
    filter_column_request: Option<RwSignal<Option<String>>>,
    filter_values: Signal<LoadedFilterValues>,
    layout: Layout<Row>,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
{
    let navigate = row_href.as_ref().map(|_| use_navigate());
    let calendar_row_href = row_href.clone();
    let column_count = columns.len();
    let columns = StoredValue::new(columns);
    let sort_state: RwSignal<Option<(usize, bool)>> = RwSignal::new(None);
    let column_filters: RwSignal<HashMap<usize, HashSet<String>>> = RwSignal::new(HashMap::new());
    let value_search = RwSignal::new(String::new());
    let server_query = query;
    let server_driven = server_query.is_some();
    let server_filtering = server_query.zip(filter_column_request);

    let column_key_at = move |index: usize| -> Option<&'static str> {
        columns.with_value(|all| all.get(index).map(|column| column.key))
    };

    let displayed_sort: Signal<Option<(usize, bool)>> = match server_query {
        Some(query) => Signal::derive(move || {
            query.with(|query| {
                let sort_key = query.sort_column_key.clone()?;
                let index = columns
                    .with_value(|all| all.iter().position(|column| column.key == sort_key))?;
                Some((index, query.sort_ascending))
            })
        }),
        None => sort_state.into(),
    };

    let is_filterable = move |column: &Column<Row>| {
        if server_driven {
            column.filterable && server_filtering.is_some()
        } else {
            column.filterable
        }
    };

    let is_filtered = move |index: usize| match server_filtering {
        Some((query, _)) => column_key_at(index)
            .is_some_and(|key| query.with(|query| query.is_column_filtered(key))),
        None => column_filters.with(|filters| filters.contains_key(&index)),
    };

    let distinct_values = move |index: usize| -> Vec<(String, String)> {
        let Some(Ok(all_rows)) = rows.get() else {
            return Vec::new();
        };
        columns.with_value(|all_columns| {
            let Some(column) = all_columns.get(index) else {
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
        })
    };
    let empty_message = if empty_message.is_empty() {
        NO_ROWS_MESSAGE.to_string()
    } else {
        empty_message
    };
    let empty_message = StoredValue::new(empty_message);

    let visible_rows: Signal<Vec<Row>> = Signal::derive(move || {
        let Some(Ok(all_rows)) = rows.get() else {
            return Vec::new();
        };
        columns.with_value(|all_columns| {
            let search_text = if server_driven {
                String::new()
            } else {
                filter.get().to_lowercase()
            };
            let mut visible: Vec<Row> = if search_text.is_empty() {
                all_rows
            } else {
                all_rows
                    .into_iter()
                    .filter(|row| {
                        all_columns
                            .iter()
                            .filter(|column| column.searchable)
                            .any(|column| (column.value)(row).search_text().contains(&search_text))
                    })
                    .collect()
            };

            let active_filters = if server_driven {
                HashMap::new()
            } else {
                column_filters.get()
            };
            if !active_filters.is_empty() {
                visible.retain(|row| {
                    active_filters.iter().all(|(column_index, allowed)| {
                        match all_columns.get(*column_index) {
                            Some(column) => allowed.contains(&(column.value)(row).filter_key()),
                            None => true,
                        }
                    })
                });
            }

            if let Some((index, ascending)) = sort_state.get().filter(|_| !server_driven) {
                if let Some(column) = all_columns.get(index) {
                    let mut keyed: Vec<(CellValue, Row)> = visible
                        .into_iter()
                        .map(|row| ((column.value)(&row), row))
                        .collect();
                    keyed.sort_by(|(left_value, _), (right_value, _)| {
                        match (left_value.is_empty(), right_value.is_empty()) {
                            (true, true) => Ordering::Equal,
                            (true, false) => Ordering::Greater,
                            (false, true) => Ordering::Less,
                            (false, false) => {
                                let ordering = left_value.compare(right_value);
                                if ascending {
                                    ordering
                                } else {
                                    ordering.reverse()
                                }
                            }
                        }
                    });
                    visible = keyed.into_iter().map(|(_, row)| row).collect();
                }
            }
            visible
        })
    });

    let render_row = {
        move |row: Row| {
            columns.with_value(|all_columns| {
                let cells = all_columns
                    .iter()
                    .map(|column| {
                        let class = format!(
                            "px-4 py-3 text-gray-700 {} truncate",
                            column.align.text_class()
                        );
                        let content = (column.render)(&row);
                        view! { <td class=class>{content}</td> }
                    })
                    .collect_view();
                let extra = row_class
                    .as_ref()
                    .map(|class_for_row| class_for_row(&row))
                    .unwrap_or_default();
                match row_href
                    .as_ref()
                    .map(|href_for_row| href_for_row(&row))
                    .filter(|target| !target.is_empty())
                {
                    Some(target) => {
                        let class = format!("{CLICKABLE_ROW_CLASS} {extra}");
                        let navigate = navigate.clone();
                        view! {
                            <tr
                                class=class
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
                    None => view! { <tr class=extra>{cells}</tr> }.into_any(),
                }
            })
        }
    };

    let status_rows = move || match rows.get() {
        Some(Err(message)) => view! {
            <tr>
                <td colspan=column_count.to_string() class="px-4 py-6">
                    {error_alert(message)}
                </td>
            </tr>
        }
        .into_any(),
        _ => ().into_any(),
    };

    let data_rows = match row_key {
        Some(key_for_row) => {
            let render_row = render_row.clone();
            view! {
                <For
                    each=move || visible_rows.get()
                    key=move |row| key_for_row(row)
                    children=render_row
                />
            }
            .into_any()
        }
        None => (move || {
            visible_rows
                .get()
                .into_iter()
                .map(&render_row)
                .collect_view()
        })
        .into_any(),
    };

    let column_widths = columns.with_value(|all_columns| {
        all_columns
            .iter()
            .map(|column| {
                let style = format!(
                    "width: calc({}ch + {CELL_PADDING_WIDTH})",
                    column.rendered_width_characters()
                );
                view! { <col style=style /> }
            })
            .collect_view()
    });

    let pager = move || {
        let Some(query) = server_query else {
            let Some(Ok(all_rows)) = rows.get() else {
                return ().into_any();
            };
            let shown = visible_rows.get().len();
            if shown == 0 {
                return ().into_any();
            }
            let total = all_rows.len();
            let summary = if shown == total {
                format!("Showing {total}")
            } else {
                format!("Showing {shown} of {total}")
            };
            return view! {
                <div class="mt-4 flex">
                    <span class="text-sm text-gray-500 tabular-nums">{summary}</span>
                </div>
            }
            .into_any();
        };
        let rows_on_page = match rows.get() {
            Some(Ok(page)) => page.len(),
            _ => return ().into_any(),
        };
        let current = query.get();
        let total = total_rows.get();
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
    };

    let header_for_key = move |column_key: &str| -> &'static str {
        columns.with_value(|all| {
            all.iter()
                .find(|column| column.key == column_key)
                .map_or("", |column| column.header)
        })
    };

    let chip = move |label: String, summary: String, clear: Arc<dyn Fn()>| {
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
    };

    let server_filter_chips = move |query: UrlTableQuery| {
        query.with(|current| {
            current
                .column_filters
                .iter()
                .map(|(column_key, values)| {
                    let owned_key = column_key.clone();
                    chip(
                        header_for_key(column_key).to_string(),
                        format!("{} selected", values.len()),
                        Arc::new(move || {
                            let owned_key = owned_key.clone();
                            query.update(|query| query.clear_column_filter(&owned_key));
                        }),
                    )
                })
                .collect_view()
        })
    };

    let client_filter_chips = move || {
        let mut entries: Vec<(usize, usize)> = column_filters.with(|active| {
            active
                .iter()
                .map(|(index, allowed)| (*index, allowed.len()))
                .collect()
        });
        entries.sort_by_key(|(index, _)| *index);
        entries
            .into_iter()
            .map(|(index, selected_count)| {
                let header =
                    columns.with_value(|all| all.get(index).map_or("", |column| column.header));
                let total = distinct_values(index).len();
                chip(
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
    };

    let filters_are_active = move || match server_filtering {
        Some((query, _)) => query.with(|query| query.filtered_column_count() > 0),
        None => column_filters.with(|active| !active.is_empty()),
    };

    let filter_chips = move || {
        if !filters_are_active() {
            return ().into_any();
        }
        let chips = match server_filtering {
            Some((query, _)) => server_filter_chips(query).into_any(),
            None => client_filter_chips().into_any(),
        };
        view! {
            <div class="mb-4 flex flex-wrap items-center gap-2">
                {chips}
                <button
                    class="cursor-pointer text-xs text-gray-500 underline hover:text-gray-700"
                    on:click=move |_| match server_filtering {
                        Some((query, _)) => query.update(TableQuery::clear_all_column_filters),
                        None => column_filters.set(HashMap::new()),
                    }
                >
                    "Clear All"
                </button>
            </div>
        }
        .into_any()
    };

    let column_menu = move |index: usize| {
        let (sortable, filterable) = columns.with_value(|all| {
            all.get(index).map_or((false, false), |column| {
                (column.sortable, is_filterable(column))
            })
        });
        let popover = use_popover();

        let sort_section = if sortable {
            let sort_button = move |ascending: bool| {
                view! {
                    <button
                        class=ITEM_CLASS
                        on:mousemove=|event: leptos::ev::MouseEvent| focus_on_hover(&event)
                        on:click=move |_| {
                            sort_state.set(Some((index, ascending)));
                            if let Some(query) = server_query {
                                let column_key = columns
                                    .with_value(|all| all.get(index).map(|column| column.key));
                                if let Some(column_key) = column_key {
                                    query
                                        .update(|query| {
                                            query.set_sort(column_key.to_string(), ascending);
                                        });
                                }
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
            };
            view! { <div class="border-b border-gray-100 p-1">{sort_button(true)} {sort_button(false)}</div> }
            .into_any()
        } else {
            ().into_any()
        };

        let filter_section = if !filterable {
            ().into_any()
        } else if let Some((query, _)) = server_filtering {
            let Some(column_key) = column_key_at(index) else {
                return ().into_any();
            };
            server_filter_section(ServerFilterSection {
                query,
                column_key,
                filter_values,
                value_search,
            })
        } else {
            let all_values = distinct_values(index);
            let all_keys: HashSet<String> = all_values.iter().map(|(key, _)| key.clone()).collect();
            let value_list = {
                let all_values = all_values.clone();
                let all_keys = all_keys.clone();
                move || {
                    let search_lower = value_search.get().to_lowercase();
                    all_values
                        .iter()
                        .filter(|(_, label)| {
                            search_lower.is_empty()
                                || label.to_lowercase().contains(&search_lower)
                        })
                        .cloned()
                        .map(|(key, label)| {
                            let checked = column_filters.with(|filters| {
                                filters
                                    .get(&index)
                                    .is_none_or(|set| set.contains(&key))
                            });
                            let all_keys = all_keys.clone();
                            let on_toggle = move |()| {
                                let all_keys = all_keys.clone();
                                column_filters.update(|filters| {
                                    let set = filters
                                        .entry(index)
                                        .or_insert_with(|| all_keys.clone());
                                    if set.contains(&key) {
                                        set.remove(&key);
                                    } else {
                                        set.insert(key.clone());
                                    }
                                    if *set == all_keys {
                                        filters.remove(&index);
                                    }
                                });
                            };
                            view! { <FilterOption label=label checked=checked on_toggle=on_toggle /> }
                        })
                        .collect_view()
                }
            };
            view! {
                <div class="border-b border-gray-100 p-2.5">
                    <SearchInput
                        value=value_search
                        placeholder="Search Values"
                        size=SearchSize::Compact
                    />
                </div>
                <div class="max-h-56 overflow-auto p-1.5 [scrollbar-gutter:stable] [scrollbar-width:thin]">
                    {value_list}
                </div>
                <div class="flex items-center justify-between border-t border-gray-100 px-3 py-2.5 text-xs text-blue-600">
                    <button
                        class=FOOTER_ACTION_CLASS
                        on:click=move |_| {
                            column_filters
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
                            column_filters
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
        };

        view! {
            {sort_section}
            {filter_section}
        }
        .into_any()
    };

    let header = move || {
        columns.with_value(|all_columns| {
            all_columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    let filterable = is_filterable(column);
                    let interactive = column.sortable || filterable;
                    let label_class =
                        format!("{} {}", column.align.text_class(), crate::typography::LABEL_CLASS);
                    let label = view! {
                        <span class="inline-flex items-center gap-1 align-middle">
                            {column.header}
                            {move || match displayed_sort.get() {
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
                                let filtered = is_filtered(index);
                                let sorted = matches!(
                                    displayed_sort.get(),
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
                    };
                    if !interactive {
                        return view! { <th class=format!("truncate px-4 py-3 select-none {label_class}")>{label}</th> }
                        .into_any();
                    }
                    let on_open = Callback::new(move |()| {
                        value_search.set(String::new());
                        if let Some((_, requested_column)) = server_filtering {
                            requested_column
                                .set(
                                    filterable
                                        .then(|| column_key_at(index))
                                        .flatten()
                                        .map(str::to_string),
                                );
                        }
                    });
                    let menu_width_class = if filterable {
                        FILTER_MENU_WIDTH_CLASS
                    } else {
                        SORT_MENU_WIDTH_CLASS
                    };
                    view! {
                        <th class=format!("truncate px-4 py-3 {}", column.align.text_class())>
                            <Popover align=menu_alignment(column.align) on_open=on_open>
                                <PopoverTrigger class=format!(
                                    "{HEADER_TRIGGER_CLASS} {}",
                                    crate::typography::LABEL_CLASS,
                                )>{label}</PopoverTrigger>
                                <PopoverContent class=menu_width_class unpadded=true>
                                    {move || column_menu(index)}
                                </PopoverContent>
                            </Popover>
                        </th>
                    }
                    .into_any()
                })
                .collect_view()
                .into_any()
        })
    };

    let search_is_active = move || match server_query {
        Some(query) => query.with(|query| !query.search.is_empty()),
        None => filter.with(|search| !search.is_empty()),
    };
    let reset_filters = move || {
        filter.set(String::new());
        column_filters.set(HashMap::new());
        if let Some(query) = server_query {
            query.update(|query| {
                query.set_search(String::new());
                query.clear_all_column_filters();
            });
        }
    };
    let shows_empty_state = !layout.is_calendar();
    let body_state = Memo::new(move |_| match rows.read().as_ref() {
        None => BodyState::Loading,
        Some(Ok(_)) if shows_empty_state && visible_rows.with(Vec::is_empty) => BodyState::Empty,
        Some(_) => BodyState::Rows,
    });
    let empty_state = move || {
        if !(filters_are_active() || search_is_active()) {
            return view! { <EmptyState title=empty_message.get_value() /> }.into_any();
        }
        let reset_button = view! {
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=move |_| reset_filters()
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
    };
    let placeholder = move || match body_state.get() {
        BodyState::Loading => view! { <LoadingState class=BODY_HEIGHT_CLASS /> }.into_any(),
        BodyState::Empty => empty_state(),
        BodyState::Rows => ().into_any(),
    };
    let pager = move || shows_empty_state.then(&pager);
    let body = match (layout, server_query) {
        (Layout::Cards(card_for_row), _) => card_body(rows, visible_rows, card_for_row, pending),
        (Layout::Calendar { date_column_key }, Some(query)) => view! {
            <CalendarView
                rows=rows
                columns=columns.get_value()
                date_column_key=date_column_key
                query=query
                total_rows=total_rows
                pending=pending
                row_href=calendar_row_href
            />
        }
        .into_any(),
        _ => view! {
            <div class="relative">
                <div class=format!(
                    "w-full max-w-full overflow-auto rounded-surface border border-gray-200 bg-white {BODY_MAX_HEIGHT_CLASS}",
                )>
                    <table class="w-full table-fixed text-sm tabular-nums">
                        <colgroup>{column_widths}</colgroup>
                        <thead class=format!("sticky top-0 z-10 bg-gray-50 {HEADER_RULE_CLASS}")>
                            <tr>{header}</tr>
                        </thead>
                        <tbody
                            class="divide-y divide-gray-100"
                            aria-busy=move || pending.get().to_string()
                        >
                            {status_rows}
                            {data_rows}
                        </tbody>
                    </table>
                </div>
                {refresh_cover(pending, "inset-px rounded-surface-inner")}
            </div>
        }
        .into_any(),
    };
    view! {
        <div>
            {filter_chips}
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
