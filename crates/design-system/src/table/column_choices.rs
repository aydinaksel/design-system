use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use leptos::ev::MouseEvent;
use leptos::prelude::*;

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::icons;
use crate::menu::{focus_on_hover, ICON_CLASS, ITEM_CLASS};
use crate::multi_select_list::{MultiSelectList, MultiSelectOption};
use crate::popover::{Popover, PopoverAlign, PopoverContent, PopoverTrigger};

use super::column::Column;
use super::facet_row::CHEVRON_CLASS;

const MENU_WIDTH_CLASS: &str = "w-64";
const MENU_LABEL: &str = "Columns";
const SEARCH_PLACEHOLDER: &str = "Search Columns";
const TRIGGER_ICON_CLASS: &str = "h-4 w-4 shrink-0 opacity-70";

pub type ColumnOverrides = BTreeMap<String, bool>;
pub type SavedColumnChoices = BTreeMap<String, ColumnOverrides>;
pub type SaveColumnChoices = Callback<(String, ColumnOverrides)>;

#[derive(Clone, Copy)]
pub struct ColumnChoiceStore {
    saved: RwSignal<SavedColumnChoices>,
    on_save: Option<SaveColumnChoices>,
}

impl ColumnChoiceStore {
    #[must_use]
    pub fn new(saved: SavedColumnChoices, on_save: SaveColumnChoices) -> Self {
        Self {
            saved: RwSignal::new(saved),
            on_save: Some(on_save),
        }
    }

    fn in_memory() -> Self {
        Self {
            saved: RwSignal::new(BTreeMap::new()),
            on_save: None,
        }
    }

    #[must_use]
    pub fn shown_columns<'columns, Row: Send + Sync + 'static>(
        &self,
        table_key: &str,
        columns: &'columns [Column<Row>],
    ) -> Vec<&'columns Column<Row>> {
        let overrides = self.overrides(table_key);
        columns
            .iter()
            .enumerate()
            .filter(|(index, column)| is_shown(*index, column, &overrides))
            .map(|(_, column)| column)
            .collect()
    }

    fn overrides(&self, table_key: &str) -> ColumnOverrides {
        self.saved
            .with(|saved| saved.get(table_key).cloned().unwrap_or_default())
    }

    fn choose(&self, table_key: &str, overrides: ColumnOverrides) {
        self.saved.update(|saved| {
            if overrides.is_empty() {
                saved.remove(table_key);
            } else {
                saved.insert(table_key.to_owned(), overrides.clone());
            }
        });
        if let Some(on_save) = self.on_save {
            on_save.run((table_key.to_owned(), overrides));
        }
    }
}

fn can_hide<Row: Send + Sync + 'static>(index: usize, column: &Column<Row>) -> bool {
    index > 0 && !column.header.is_empty()
}

fn is_shown<Row: Send + Sync + 'static>(
    index: usize,
    column: &Column<Row>,
    overrides: &ColumnOverrides,
) -> bool {
    !can_hide(index, column)
        || overrides
            .get(column.key)
            .copied()
            .unwrap_or(column.default_visibility.is_shown())
}

fn with_choice<Row: Send + Sync + 'static>(
    mut overrides: ColumnOverrides,
    column: &Column<Row>,
    shown: bool,
) -> ColumnOverrides {
    if shown == column.default_visibility.is_shown() {
        overrides.remove(column.key);
    } else {
        overrides.insert(column.key.to_owned(), shown);
    }
    overrides
}

pub(super) struct ColumnChoices<Row: Send + Sync + 'static> {
    table_key: Arc<str>,
    store: ColumnChoiceStore,
    columns: Arc<[Column<Row>]>,
}

impl<Row: Send + Sync + 'static> Clone for ColumnChoices<Row> {
    fn clone(&self) -> Self {
        Self {
            table_key: Arc::clone(&self.table_key),
            store: self.store,
            columns: Arc::clone(&self.columns),
        }
    }
}

impl<Row: Send + Sync + 'static> ColumnChoices<Row> {
    pub(super) fn new(table_key: &str, columns: Arc<[Column<Row>]>) -> Self {
        Self {
            table_key: table_key.into(),
            store: use_context::<ColumnChoiceStore>().unwrap_or_else(ColumnChoiceStore::in_memory),
            columns,
        }
    }

    pub(super) fn shown_indices(&self) -> Vec<usize> {
        let overrides = self.store.overrides(&self.table_key);
        self.columns
            .iter()
            .enumerate()
            .filter(|(index, column)| is_shown(*index, column, &overrides))
            .map(|(index, _)| index)
            .collect()
    }

    fn hideable(&self) -> impl Iterator<Item = &Column<Row>> {
        self.columns
            .iter()
            .enumerate()
            .filter(|(index, column)| can_hide(*index, column))
            .map(|(_, column)| column)
    }

    fn options(&self) -> Vec<MultiSelectOption> {
        self.hideable()
            .map(|column| MultiSelectOption {
                value: column.key.to_owned(),
                label: column.header.to_owned(),
                row_count: None,
            })
            .collect()
    }

    fn shown_keys(&self) -> BTreeSet<String> {
        let overrides = self.store.overrides(&self.table_key);
        self.columns
            .iter()
            .enumerate()
            .filter(|(index, column)| {
                can_hide(*index, column) && is_shown(*index, column, &overrides)
            })
            .map(|(_, column)| column.key.to_owned())
            .collect()
    }

    fn toggle(&self, key: &str) {
        let Some(column) = self.hideable().find(|column| column.key == key) else {
            return;
        };
        let shown = self.shown_keys().contains(key);
        let overrides = with_choice(self.store.overrides(&self.table_key), column, !shown);
        self.store.choose(&self.table_key, overrides);
    }

    fn show_every_column(&self, shown: bool) {
        let overrides = self
            .hideable()
            .fold(ColumnOverrides::new(), |overrides, column| {
                with_choice(overrides, column, shown)
            });
        self.store.choose(&self.table_key, overrides);
    }

    fn reset(&self) {
        self.store.choose(&self.table_key, ColumnOverrides::new());
    }

    pub(super) fn menu(&self) -> AnyView {
        let listing = self.clone();
        let options = Signal::derive(move || listing.options());
        let reading = self.clone();
        let selected = Signal::derive(move || reading.shown_keys());
        let toggling = self.clone();
        let on_toggle = Callback::new(move |key: String| toggling.toggle(&key));
        let resetting = self.clone();
        let showing = self.clone();
        let hiding = self.clone();
        let content = move || {
            let resetting = resetting.clone();
            let showing = showing.clone();
            let hiding = hiding.clone();
            view! {
                <MultiSelectList
                    options=options
                    selected=selected
                    on_toggle=on_toggle
                    label=MENU_LABEL
                    search_placeholder=SEARCH_PLACEHOLDER
                />
                <div class="border-t border-divider p-1">
                    {menu_action(
                        icons::LucideGlyph::RotateCcw,
                        "Reset Columns",
                        move || resetting.reset(),
                    )}
                    {menu_action(
                        icons::LucideGlyph::Eye,
                        "Select All",
                        move || showing.show_every_column(true),
                    )}
                    {menu_action(
                        icons::LucideGlyph::EyeOff,
                        "Deselect All",
                        move || hiding.show_every_column(false),
                    )}
                </div>
            }
        };
        view! {
            <Popover align=PopoverAlign::End>
                <PopoverTrigger class=button_class(
                    ButtonTone::Secondary,
                    ButtonSize::Standard,
                )>
                    {icons::icon(icons::LucideGlyph::Columns3, TRIGGER_ICON_CLASS)} {MENU_LABEL}
                    {icons::icon(icons::LucideGlyph::ChevronDown, CHEVRON_CLASS)}
                </PopoverTrigger>
                <PopoverContent class=MENU_WIDTH_CLASS unpadded=true>
                    {content.clone()}
                </PopoverContent>
            </Popover>
        }
        .into_any()
    }
}

fn menu_action(
    glyph: icons::LucideGlyph,
    label: &'static str,
    action: impl Fn() + Send + Sync + 'static,
) -> AnyView {
    view! {
        <button
            type="button"
            class=ITEM_CLASS
            on:mousemove=|event: MouseEvent| focus_on_hover(&event)
            on:click=move |_| action()
        >
            {icons::icon(glyph, ICON_CLASS)}
            {label}
        </button>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns() -> Vec<Column<String>> {
        vec![
            Column::text("code", "Code", |row: &String| Some(row.clone())),
            Column::text("name", "Name", |row: &String| Some(row.clone())),
            Column::text("note", "Note", |row: &String| Some(row.clone())).hidden_by_default(),
            Column::actions(|_| ().into_any()),
        ]
    }

    fn shown_keys(columns: &[Column<String>], overrides: &ColumnOverrides) -> Vec<&'static str> {
        columns
            .iter()
            .enumerate()
            .filter(|(index, column)| is_shown(*index, column, overrides))
            .map(|(_, column)| column.key)
            .collect()
    }

    #[test]
    fn without_choices_the_defaults_decide() {
        assert_eq!(
            shown_keys(&columns(), &ColumnOverrides::new()),
            ["code", "name", "actions"]
        );
    }

    #[test]
    fn the_first_column_and_unlabelled_columns_stay_when_everything_is_hidden() {
        let columns = columns();
        let overrides = columns
            .iter()
            .fold(ColumnOverrides::new(), |overrides, column| {
                with_choice(overrides, column, false)
            });

        assert_eq!(shown_keys(&columns, &overrides), ["code", "actions"]);
    }

    #[test]
    fn choosing_a_column_its_default_already_gives_records_nothing() {
        let columns = columns();
        let overrides = with_choice(ColumnOverrides::new(), &columns[1], true);

        assert!(overrides.is_empty());
    }

    #[test]
    fn a_choice_against_the_default_is_recorded() {
        let columns = columns();
        let overrides = with_choice(ColumnOverrides::new(), &columns[2], true);

        assert_eq!(overrides.get("note"), Some(&true));
        assert_eq!(
            shown_keys(&columns, &overrides),
            ["code", "name", "note", "actions"]
        );
    }
}
