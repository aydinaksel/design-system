use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::catalog::CardGrid;
use crate::icons;
use crate::search_input::{SearchInput, SearchSize};

use super::cell_value::CellValue;
use super::column::Column;
use super::filter_option::FilterOption;
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

impl Search {
    fn is_shown(self) -> bool {
        !matches!(self, Search::Off)
    }
}

const LOADING_ROW_COUNT: usize = 20;
const LOADING_CARD_COUNT: usize = 8;
const NO_ROWS_MESSAGE: &str = "No results.";
const NO_MATCHES_MESSAGE: &str = "No matching results.";

/// Generic table: absorbs loading skeleton / error / empty states, a sticky
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

const POPOVER_VIEWPORT_MARGIN_PIXELS: u32 = 8;
const FILTER_POPOVER_WIDTH_PIXELS: u32 = 256;
const SORT_POPOVER_WIDTH_PIXELS: u32 = 176;

#[derive(Clone, Copy, Default)]
struct PopoverAnchor {
    left: f64,
    top: f64,
}

impl PopoverAnchor {
    fn position_style(self, width_pixels: u32) -> String {
        let Self { left, top } = self;
        let margin = POPOVER_VIEWPORT_MARGIN_PIXELS;
        format!(
            "left:clamp({margin}px, {left}px, max({margin}px, 100vw - {width_pixels}px - {margin}px));\
             top:{top}px;width:{width_pixels}px"
        )
    }
}

#[cfg(feature = "hydrate")]
fn capture_anchor(event: &leptos::ev::MouseEvent, anchor: RwSignal<PopoverAnchor>) {
    use leptos::wasm_bindgen::JsCast;
    const POPOVER_GAP_PIXELS: f64 = 6.0;
    let Some(target) = event.current_target() else {
        return;
    };
    let element: web_sys::Element = target.unchecked_into();
    let rect = element.get_bounding_client_rect();
    anchor.set(PopoverAnchor {
        left: rect.left(),
        top: rect.bottom() + POPOVER_GAP_PIXELS,
    });
}

#[cfg(not(feature = "hydrate"))]
fn capture_anchor(_event: &leptos::ev::MouseEvent, _anchor: RwSignal<PopoverAnchor>) {}

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

pub(crate) const BODY_HEIGHT_CLASS: &str = "h-[calc(100vh-18rem)]";
pub(crate) const BODY_MAX_HEIGHT_CLASS: &str = "max-h-[calc(100vh-18rem)]";

fn pending_spinner(pending: Signal<bool>, class: &'static str) -> AnyView {
    view! {
        <span class=move || {
            if pending.get() { "inline-flex" } else { "hidden" }
        }>{icons::spinner(class)}</span>
    }
    .into_any()
}

fn error_alert(message: String) -> AnyView {
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

fn empty_rows_message(all_rows_are_empty: bool, empty_message: StoredValue<String>) -> String {
    if all_rows_are_empty {
        empty_message.get_value()
    } else {
        NO_MATCHES_MESSAGE.to_string()
    }
}

fn card_body<Row>(
    rows: Signal<Option<Result<Vec<Row>, String>>>,
    visible_rows: Signal<Vec<Row>>,
    card_for_row: RowView<Row>,
    empty_message: StoredValue<String>,
    pending: Signal<bool>,
) -> AnyView
where
    Row: Clone + Send + Sync + 'static,
{
    let cards = move || match rows.get() {
        None => (0..LOADING_CARD_COUNT)
            .map(|_| view! { <li class="aspect-square animate-pulse rounded-lg bg-gray-100"></li> })
            .collect_view()
            .into_any(),
        Some(Err(message)) => {
            view! { <li class="col-span-full">{error_alert(message)}</li> }.into_any()
        }
        Some(Ok(all_rows)) if visible_rows.with(Vec::is_empty) => {
            let message = empty_rows_message(all_rows.is_empty(), empty_message);
            view! { <li class="col-span-full py-8 text-center text-sm text-gray-400">{message}</li> }
                .into_any()
        }
        Some(Ok(_)) => visible_rows
            .get()
            .iter()
            .map(|row| card_for_row(row))
            .collect_view()
            .into_any(),
    };
    view! {
        <div
            class=move || {
                let fade = if pending.get() { "opacity-50" } else { "" };
                format!("overflow-auto transition-opacity {BODY_MAX_HEIGHT_CLASS} {fade}")
            }
            aria-busy=move || pending.get().to_string()
        >
            <CardGrid>{cards}</CardGrid>
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
                class="cursor-pointer hover:underline"
                on:click=move |_| query.update(|query| query.clear_column_filter(column_key))
            >
                "Select All"
            </button>
            <button
                class="cursor-pointer hover:underline"
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

#[expect(
    clippy::too_many_lines,
    reason = "the component wires search, sort, and filter state into one view! tree"
)]
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
    /// Rendered against the search row's left edge.
    #[prop(optional_no_strip)]
    toolbar: Option<AnyView>,
    /// Rendered beside the table body, below the toolbar, such as a facet rail.
    #[prop(optional_no_strip)]
    sidebar: Option<AnyView>,
    /// Switches the table to server-driven mode, reading `rows` as one page.
    #[prop(optional)]
    query: Option<UrlTableQuery>,
    /// Rows matching the query in full. Server-driven mode only.
    #[prop(optional, into)]
    total_rows: Signal<Option<u64>>,
    /// True while stale rows await the server's answer to a new query.
    #[prop(optional, into)]
    pending: Signal<bool>,
    /// The column key whose values the page should load, written when a filter
    /// popover opens. Enables `filterable` columns in server-driven mode.
    #[prop(optional_no_strip)]
    filter_column_request: Option<RwSignal<Option<String>>>,
    /// The answer to `filter_column_request`. Server-driven mode only.
    #[prop(optional, into)]
    filter_values: Signal<LoadedFilterValues>,
    /// Draws each row as a card in a grid in place of the table.
    #[prop(optional_no_strip)]
    card: Option<RowView<Row>>,
) -> impl IntoView
where
    Row: Clone + Send + Sync + 'static,
{
    let navigate = row_href.as_ref().map(|_| use_navigate());
    let column_count = columns.len();
    let columns = StoredValue::new(columns);
    let filter = RwSignal::new(
        query
            .map(|query| query.with_untracked(|query| query.search.clone()))
            .unwrap_or_default(),
    );
    let sort_state: RwSignal<Option<(usize, bool)>> = RwSignal::new(None);
    let column_filters: RwSignal<HashMap<usize, HashSet<String>>> = RwSignal::new(HashMap::new());
    let open_filter: RwSignal<Option<usize>> = RwSignal::new(None);
    let value_search = RwSignal::new(String::new());
    let anchor: RwSignal<PopoverAnchor> = RwSignal::new(PopoverAnchor::default());
    let server_query = query;
    let server_driven = server_query.is_some();
    let server_filtering = server_query.zip(filter_column_request);
    let search_enabled = search.is_shown();

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
    let placeholder = if search_placeholder.is_empty() {
        "Search".to_string()
    } else {
        search_placeholder
    };
    let empty_message = if empty_message.is_empty() {
        NO_ROWS_MESSAGE.to_string()
    } else {
        empty_message
    };
    let empty_message = StoredValue::new(empty_message);

    let header = move || {
        if rows.read().is_none() {
            return view! {
                <th colspan=column_count.to_string() class="px-4 py-3">
                    <div class="h-4 w-full rounded bg-gray-100 animate-pulse"></div>
                </th>
            }
            .into_any();
        }
        columns.with_value(|all_columns| {
            all_columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    let filterable = is_filterable(column);
                    let interactive = column.sortable || filterable;
                    let base = format!(
                        "px-4 py-3 {} {} select-none",
                        column.align.text_class(),
                        crate::typography::LABEL_CLASS,
                    );
                    let class = if interactive {
                        format!("{base} cursor-pointer hover:bg-gray-100")
                    } else {
                        base
                    };
                    let header_label = column.header;
                    let caret = if interactive {
                        view! {
                            {move || {
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
                        }
                        .into_any()
                    } else {
                        ().into_any()
                    };
                    view! {
                        <th
                            class=class
                            on:click=move |event: leptos::ev::MouseEvent| {
                                if !interactive {
                                    return;
                                }
                                if open_filter.get_untracked() == Some(index) {
                                    open_filter.set(None);
                                    return;
                                }
                                capture_anchor(&event, anchor);
                                value_search.set(String::new());
                                open_filter.set(Some(index));
                                if let Some((_, requested_column)) = server_filtering {
                                    requested_column
                                        .set(
                                            filterable
                                                .then(|| column_key_at(index))
                                                .flatten()
                                                .map(str::to_string),
                                        );
                                }
                            }
                        >
                            <span class="inline-flex items-center gap-1 align-middle">
                                {header_label}
                                {move || match displayed_sort.get() {
                                    Some((current, ascending)) if current == index => {
                                        icons::labelled_icon(
                                            sort_arrow_glyph(ascending),
                                            "h-3.5 w-3.5 shrink-0 text-gray-900",
                                            sort_direction_label(ascending),
                                        )
                                    }
                                    _ => ().into_any(),
                                }} {caret}
                            </span>
                        </th>
                    }
                })
                .collect_view()
                .into_any()
        })
    };

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
                            "px-4 py-3 text-gray-700 {} whitespace-nowrap",
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
                let mut class = if extra.is_empty() {
                    "hover:bg-gray-50".to_string()
                } else {
                    format!("hover:bg-gray-50 {extra}")
                };
                match row_href
                    .as_ref()
                    .map(|href_for_row| href_for_row(&row))
                    .filter(|target| !target.is_empty())
                {
                    Some(target) => {
                        class.push_str(" cursor-pointer");
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
                    None => view! { <tr class=class>{cells}</tr> }.into_any(),
                }
            })
        }
    };

    let status_rows = move || match rows.get() {
        None => (0..LOADING_ROW_COUNT)
            .map(|_| {
                view! {
                    <tr>
                        <td colspan=column_count.to_string() class="px-4 py-3">
                            <div class="h-4 w-full rounded bg-gray-100 animate-pulse"></div>
                        </td>
                    </tr>
                }
            })
            .collect_view()
            .into_any(),
        Some(Err(message)) => view! {
            <tr>
                <td colspan=column_count.to_string() class="px-4 py-6">
                    {error_alert(message)}
                </td>
            </tr>
        }
        .into_any(),
        Some(Ok(all_rows)) => {
            if visible_rows.with(Vec::is_empty) {
                let message = empty_rows_message(all_rows.is_empty(), empty_message);
                view! {
                    <tr>
                        <td
                            colspan=column_count.to_string()
                            class="px-4 py-8 text-center text-gray-400"
                        >
                            {message}
                        </td>
                    </tr>
                }
                .into_any()
            } else {
                ().into_any()
            }
        }
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

    let width_holding_row = move || {
        if server_driven {
            return ().into_any();
        }
        let Some(Ok(all_rows)) = rows.get() else {
            return ().into_any();
        };
        columns.with_value(|all_columns| {
            let cells = all_columns
                .iter()
                .map(|column| {
                    let class =
                        format!("px-4 py-0 {} whitespace-nowrap", column.align.text_class());
                    let widest = all_rows
                        .iter()
                        .max_by_key(|row| (column.value)(row).search_text().chars().count());
                    let content = widest.map(|row| (column.render)(row));
                    view! {
                        <td class=class>
                            <div class="invisible h-0 overflow-hidden">{content}</div>
                        </td>
                    }
                })
                .collect_view();
            view! {
                <tr aria-hidden="true" class="h-0 border-0">
                    {cells}
                </tr>
            }
            .into_any()
        })
    };

    let search_input = match (search_enabled, server_query) {
        (false, _) => None,
        (true, None) => Some(
            view! {
                <div class="w-full lg:max-w-sm">
                    <SearchInput value=filter placeholder=placeholder />
                </div>
            }
            .into_any(),
        ),
        (true, Some(query)) => {
            let submit = move || query.update(|query| query.set_search(filter.get_untracked()));
            Some(
                view! {
                    <div class="flex w-full max-w-lg items-center gap-2">
                        <SearchInput
                            value=filter
                            placeholder=placeholder
                            on_submit=Callback::new(move |()| submit())
                            on_clear=Callback::new(move |()| {
                                query.update(|query| query.set_search(String::new()));
                            })
                            applied=Signal::derive(move || {
                                query.with(|query| !query.search.is_empty())
                            })
                        />
                        <button
                            type="button"
                            class=format!(
                                "shrink-0 {}",
                                button_class(ButtonTone::Secondary, ButtonSize::Standard),
                            )
                            on:click=move |_| submit()
                        >
                            {pending_spinner(pending, "h-4 w-4")}
                            "Search"
                        </button>
                    </div>
                }
                .into_any(),
            )
        }
    };

    let toolbar_row_is_shown = toolbar.is_some() || search_input.is_some();
    let toolbar_row = if toolbar_row_is_shown {
        view! {
            <div class="flex flex-wrap items-center justify-between gap-3">
                <div class="flex flex-wrap items-center gap-2">{toolbar}</div>
                {search_input}
            </div>
        }
        .into_any()
    } else {
        ().into_any()
    };

    let pager = move || {
        let Some(query) = server_query else {
            let Some(Ok(all_rows)) = rows.get() else {
                return ().into_any();
            };
            let shown = visible_rows.get().len();
            let total = all_rows.len();
            let summary = if shown == total {
                format!("Showing {total}")
            } else {
                format!("Showing {shown} of {total}")
            };
            return view! {
                <div class="mt-4 flex justify-center">
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
            Some(total) => format!("Showing {first} - {last} of {total}"),
            None => format!("Showing {first} - {last}"),
        };
        let on_first_page = current.is_on_first_page();
        let on_last_page = match total {
            Some(total) => u64::from(last) >= total,
            None => rows_on_page < usize::try_from(current.rows_per_page).unwrap_or(usize::MAX),
        };
        view! {
            <div class="mt-4 flex items-center justify-between">
                <button
                    class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                    disabled=on_first_page
                    on:click=move |_| {
                        query.update(TableQuery::return_to_previous_page);
                    }
                >
                    "Previous"
                </button>
                <span class="flex items-center gap-2 text-sm text-gray-500 tabular-nums">
                    {pending_spinner(pending, "h-4 w-4 text-gray-400")} {range}
                </span>
                <button
                    class=button_class(ButtonTone::Secondary, ButtonSize::Compact)
                    disabled=on_last_page
                    on:click=move |_| {
                        query.update(TableQuery::advance_to_next_page);
                    }
                >
                    "Next"
                </button>
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
            <div class="flex flex-wrap items-center gap-2 border-t border-gray-100 pt-2">
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

    let filter_popover = move || {
        let Some(index) = open_filter.get() else {
            return ().into_any();
        };
        let (sortable, filterable) = columns.with_value(|all| {
            all.get(index).map_or((false, false), |column| {
                (column.sortable, is_filterable(column))
            })
        });
        let position_style = anchor.get().position_style(if filterable {
            FILTER_POPOVER_WIDTH_PIXELS
        } else {
            SORT_POPOVER_WIDTH_PIXELS
        });

        let sort_section = if sortable {
            let sort_button = move |ascending: bool| {
                view! {
                    <button
                        class="flex w-full cursor-pointer items-center gap-2 px-3 py-2 hover:bg-gray-50"
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
                            open_filter.set(None);
                        }
                    >
                        {icons::icon(
                            sort_arrow_glyph(ascending),
                            "h-3.5 w-3.5 shrink-0 text-gray-500",
                        )}
                        <span class="whitespace-nowrap">{sort_direction_label(ascending)}</span>
                    </button>
                }
            };
            view! {
                <div class="border-b border-gray-100 py-1.5">
                    {sort_button(true)} {sort_button(false)}
                </div>
            }
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
                        class="cursor-pointer hover:underline"
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
                        class="cursor-pointer hover:underline"
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
            <div class="fixed inset-0 z-30" on:click=move |_| open_filter.set(None)></div>
            <div
                class="fixed z-40 overflow-hidden rounded-lg bg-white text-sm text-gray-700 shadow-xl ring-1 ring-gray-900/10"
                style=position_style
            >
                {sort_section}
                {filter_section}
            </div>
        }
        .into_any()
    };

    let loading = move || rows.get().is_none();
    let container_class = move || {
        let base = "overflow-auto rounded-lg border border-gray-200 bg-white max-w-full";
        if loading() {
            format!("{base} w-full {BODY_HEIGHT_CLASS}")
        } else {
            format!("{base} w-full {BODY_MAX_HEIGHT_CLASS}")
        }
    };
    let body = match card {
        Some(card_for_row) => card_body(rows, visible_rows, card_for_row, empty_message, pending),
        None => view! {
            <div class=container_class>
                <table class="min-w-full text-sm whitespace-nowrap">
                    <thead class="bg-gray-50 sticky top-0 z-10 shadow-[inset_0_-1px_0_0_rgb(229,231,235)]">
                        <tr>{header}</tr>
                        {width_holding_row}
                    </thead>
                    <tbody
                        class=move || {
                            if pending.get() {
                                "divide-y divide-gray-100 transition-opacity opacity-50"
                            } else {
                                "divide-y divide-gray-100 transition-opacity"
                            }
                        }
                        aria-busy=move || pending.get().to_string()
                    >
                        {status_rows}
                        {data_rows}
                    </tbody>
                </table>
            </div>
        }
        .into_any(),
    };
    view! {
        <div>
            <div class=move || {
                if toolbar_row_is_shown || filters_are_active() { "mb-4 space-y-2" } else { "" }
            }>{toolbar_row} {filter_chips}</div>
            <div class="flex items-start gap-6">
                {sidebar} <div class="min-w-0 flex-1">{body} {pager}</div>
            </div>
            {filter_popover}
        </div>
    }
    .into_any()
}
