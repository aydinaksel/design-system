use std::collections::BTreeSet;
use std::sync::Arc;

use leptos::prelude::*;
use table_query::TableQuery;

use crate::icons;
use crate::multi_select_list::{MultiSelectList, MultiSelectOption};
use crate::popover::{Popover, PopoverContent, PopoverTrigger};

use super::facet_row::{clear_all_facets_button, facet_trigger_class, selected_count_badge};
use super::facet_row::{CHEVRON_CLASS, MENU_WIDTH_CLASS};
use super::layout::ServerFiltering;
use super::url_query::UrlTableQuery;

#[derive(Clone, Copy)]
pub(super) struct FacetColumn {
    pub key: &'static str,
    pub header: &'static str,
}

#[derive(Clone)]
pub(super) struct ServerFacetRow {
    pub columns: Arc<[FacetColumn]>,
    pub query: UrlTableQuery,
    pub filtering: ServerFiltering,
}

fn selected_values(query: UrlTableQuery, column_key: &str) -> BTreeSet<String> {
    query.with(|query| {
        query
            .selected_filter_values(column_key)
            .cloned()
            .unwrap_or_default()
    })
}

fn toggle_value(query: UrlTableQuery, column_key: &str, value: String) {
    let mut selected = selected_values(query, column_key);
    if !selected.remove(&value) {
        selected.insert(value);
    }
    query.update(|query| {
        if selected.is_empty() {
            query.clear_column_filter(column_key);
        } else {
            query.set_column_filter(column_key, selected);
        }
    });
}

impl ServerFacetRow {
    pub(super) fn render(&self) -> Option<AnyView> {
        if self.columns.is_empty() {
            return None;
        }
        let buttons = self
            .columns
            .iter()
            .map(|column| self.facet_button(*column))
            .collect_view();
        let query = self.query;
        let any_selected =
            Signal::derive(move || query.with(|query| query.filtered_column_count() > 0));
        let clear_all = clear_all_facets_button(any_selected, move || {
            query.update(TableQuery::clear_all_column_filters);
        });
        Some(
            view! { <div class="flex flex-wrap items-center gap-2">{buttons} {clear_all}</div> }
                .into_any(),
        )
    }

    fn facet_button(&self, column: FacetColumn) -> AnyView {
        let query = self.query;
        let filtering = self.filtering;
        let selected_count = Signal::derive(move || selected_values(query, column.key).len());
        let on_open = Callback::new(move |()| {
            filtering.requested_column.set(Some(column.key.to_owned()));
        });
        let menu = move || facet_menu(query, filtering, column);
        view! {
            <Popover on_open=on_open>
                <PopoverTrigger class=Signal::derive(move || {
                    facet_trigger_class(selected_count.get())
                })>
                    {column.header} {move || selected_count_badge(selected_count.get())}
                    {icons::icon(icons::LucideGlyph::ChevronDown, CHEVRON_CLASS)}
                </PopoverTrigger>
                <PopoverContent class=MENU_WIDTH_CLASS unpadded=true>
                    {menu}
                </PopoverContent>
            </Popover>
        }
        .into_any()
    }
}

fn offered_values(
    filtering: ServerFiltering,
    column: FacetColumn,
) -> Option<Result<Vec<MultiSelectOption>, String>> {
    let is_requested = filtering
        .requested_column
        .with(|requested| requested.as_deref() == Some(column.key));
    if !is_requested {
        return None;
    }
    filtering.values.get().map(|loaded| {
        loaded.map(|values| {
            values
                .into_iter()
                .map(|value| MultiSelectOption {
                    label: value.label().to_owned(),
                    row_count: Some(value.row_count),
                    value: value.value,
                })
                .collect()
        })
    })
}

fn facet_menu(query: UrlTableQuery, filtering: ServerFiltering, column: FacetColumn) -> AnyView {
    let offered = Memo::new(move |_| offered_values(filtering, column));
    let is_loaded = Memo::new(move |_| offered.with(|offered| matches!(offered, Some(Ok(_)))));
    let options = Signal::derive(move || offered.get().and_then(Result::ok).unwrap_or_default());
    let selected = Signal::derive(move || selected_values(query, column.key));
    let on_toggle = Callback::new(move |value: String| toggle_value(query, column.key, value));
    let on_clear = Callback::new(move |()| {
        query.update(|query| query.clear_column_filter(column.key));
    });
    (move || {
        if is_loaded.get() {
            return view! {
                <MultiSelectList
                    options=options
                    selected=selected
                    on_toggle=on_toggle
                    on_clear=on_clear
                    label=column.header
                />
            }
            .into_any();
        }
        match offered.get() {
            Some(Err(message)) => {
                view! { <p class="px-3 py-2 text-xs text-red-600">{message}</p> }.into_any()
            }
            _ => view! {
                <p class="flex items-center gap-2 px-3 py-2 text-xs text-gray-400">
                    {icons::spinner("h-3.5 w-3.5")} "Loading Values\u{2026}"
                </p>
            }
            .into_any(),
        }
    })
    .into_any()
}
