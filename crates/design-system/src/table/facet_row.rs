use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use leptos::prelude::*;

use crate::button::{button_class, button_icon_class, ButtonSize, ButtonTone, SHARED_CLASS};
use crate::filter_chip::{SELECTED_CLASS, SELECTED_COUNT_CLASS};
use crate::icons;
use crate::multi_select_list::{MultiSelectList, MultiSelectOption};
use crate::popover::{Popover, PopoverContent, PopoverTrigger};

use super::column::Column;
use super::data_table::{matches_search, passes_column_filters, ClientColumnFilters, TableRows};

const MENU_WIDTH_CLASS: &str = "w-64";
const CHEVRON_CLASS: &str = "h-3.5 w-3.5 shrink-0 opacity-70";
const FACET_BUTTON_SIZE: ButtonSize = ButtonSize::Standard;

pub(super) struct FacetRow<Row: Send + Sync + 'static> {
    pub rows: TableRows<Row>,
    pub columns: StoredValue<Vec<Column<Row>>>,
    pub column_filters: RwSignal<ClientColumnFilters>,
    pub search_text: RwSignal<String>,
}

impl<Row: Send + Sync + 'static> Clone for FacetRow<Row> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Row: Send + Sync + 'static> Copy for FacetRow<Row> {}

impl<Row: Clone + Send + Sync + 'static> FacetRow<Row> {
    pub(super) fn render(self) -> Option<AnyView> {
        let facet_indexes: Vec<usize> = self.columns.with_value(|all| {
            all.iter()
                .enumerate()
                .filter(|(_, column)| column.filterable)
                .map(|(index, _)| index)
                .collect()
        });
        if facet_indexes.is_empty() {
            return None;
        }
        let buttons = facet_indexes
            .into_iter()
            .map(|index| self.facet_button(index))
            .collect_view();
        Some(
            view! {
                <div class="flex flex-wrap items-center gap-2">
                    {buttons} {self.clear_all_button()}
                </div>
            }
            .into_any(),
        )
    }

    fn selected_count(self, index: usize) -> usize {
        self.column_filters
            .with(|filters| filters.get(&index).map_or(0, HashSet::len))
    }

    fn selected_values(self, index: usize) -> BTreeSet<String> {
        self.column_filters.with(|filters| {
            filters
                .get(&index)
                .map(|selected| selected.iter().cloned().collect())
                .unwrap_or_default()
        })
    }

    fn toggle(self, index: usize, key: &str) {
        self.column_filters.update(|filters| {
            let selected = filters.entry(index).or_default();
            if !selected.remove(key) {
                selected.insert(key.to_string());
            }
            if selected.is_empty() {
                filters.remove(&index);
            }
        });
    }

    fn counted_values(self, index: usize) -> Vec<MultiSelectOption> {
        let Some(Ok(all_rows)) = self.rows.get() else {
            return Vec::new();
        };
        let search_lower = self.search_text.get().to_lowercase();
        let filters = self.column_filters.get();
        let all_columns = self.columns.get_value();
        let Some(column) = all_columns.get(index) else {
            return Vec::new();
        };
        let mut counted: BTreeMap<String, MultiSelectOption> = BTreeMap::new();
        for row in &all_rows {
            let cell = (column.value)(row);
            let matching = matches_search(&all_columns, row, &search_lower)
                && passes_column_filters(&all_columns, &filters, row, Some(index));
            let entry = counted
                .entry(cell.filter_key())
                .or_insert_with(|| MultiSelectOption {
                    value: cell.filter_key(),
                    label: cell.filter_label(),
                    row_count: Some(0),
                });
            entry.row_count = entry
                .row_count
                .map(|count| count.saturating_add(u64::from(matching)));
        }
        counted.into_values().collect()
    }

    fn trigger_class(self, index: usize) -> String {
        if self.selected_count(index) == 0 {
            button_class(ButtonTone::Secondary, FACET_BUTTON_SIZE)
        } else {
            format!(
                "{SHARED_CLASS} {} {SELECTED_CLASS}",
                FACET_BUTTON_SIZE.class()
            )
        }
    }

    fn facet_button(self, index: usize) -> AnyView {
        let header = self
            .columns
            .with_value(|all| all.get(index).map_or("", |column| column.header));
        let values = Memo::new(move |_| self.counted_values(index));
        let selected_badge = move || match self.selected_count(index) {
            0 => ().into_any(),
            count => view! { <span class=SELECTED_COUNT_CLASS>{count}</span> }.into_any(),
        };
        view! {
            <Popover>
                <PopoverTrigger class=Signal::derive(move || self.trigger_class(index))>
                    {header}
                    {selected_badge}
                    {icons::icon(icons::LucideGlyph::ChevronDown, CHEVRON_CLASS)}
                </PopoverTrigger>
                <PopoverContent class=MENU_WIDTH_CLASS unpadded=true>
                    {move || self.menu(index, header, values)}
                </PopoverContent>
            </Popover>
        }
        .into_any()
    }

    fn menu(
        self,
        index: usize,
        header: &'static str,
        values: Memo<Vec<MultiSelectOption>>,
    ) -> AnyView {
        let selected = Signal::derive(move || self.selected_values(index));
        let on_toggle = Callback::new(move |value: String| self.toggle(index, &value));
        let column_filters = self.column_filters;
        let on_clear = Callback::new(move |()| {
            column_filters.update(|filters| {
                filters.remove(&index);
            });
        });
        view! {
            <MultiSelectList
                options=values
                selected=selected
                on_toggle=on_toggle
                on_clear=on_clear
                label=header
            />
        }
        .into_any()
    }

    fn clear_all_button(self) -> AnyView {
        let column_filters = self.column_filters;
        let button = move || {
            view! {
                <button
                    type="button"
                    class=button_class(ButtonTone::Subtle, FACET_BUTTON_SIZE)
                    on:click=move |_| column_filters.set(HashMap::new())
                >
                    {icons::icon(icons::LucideGlyph::X, button_icon_class(FACET_BUTTON_SIZE))}
                    "Clear All"
                </button>
            }
        };
        (move || (!column_filters.with(HashMap::is_empty)).then(button)).into_any()
    }
}
