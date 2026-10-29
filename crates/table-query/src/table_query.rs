use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Separates selected values in one parameter; names here contain commas.
pub const FILTER_VALUE_SEPARATOR: char = '\u{1f}';

/// Ticked values per column key: OR within a column, AND across columns. An
/// absent key is unfiltered; an empty set matches nothing.
pub type ColumnFilters = BTreeMap<String, BTreeSet<String>>;

/// What one column contributes to a server query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnSelection {
    Unfiltered,
    /// Every value was unticked, so the column can match no row at all.
    MatchesNothing,
    /// The selected values, [`FILTER_VALUE_SEPARATOR`] delimited. One empty
    /// value means the blank bucket, not an absent selection.
    AnyOf(String),
}

/// One window onto a result set the browser never holds in full.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TableQuery {
    pub search: String,
    pub sort_column_key: Option<String>,
    pub sort_ascending: bool,
    /// Defaulted, because the encodings a server function argument travels
    /// through drop an empty map rather than writing one.
    #[serde(default)]
    pub column_filters: ColumnFilters,
    /// The window a calendar shows, in place of a page of rows.
    #[serde(default)]
    pub date_range: Option<DateRange>,
    pub skipped_rows: u32,
    pub rows_per_page: u32,
}

/// An inclusive span of `yyyy-mm-dd` dates on one column.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    pub column_key: String,
    pub from: String,
    pub to: String,
}

impl TableQuery {
    #[must_use]
    pub fn with_rows_per_page(rows_per_page: u32) -> Self {
        Self {
            rows_per_page,
            sort_ascending: true,
            ..Self::default()
        }
    }

    pub fn return_to_first_page(&mut self) {
        self.skipped_rows = 0;
    }

    pub fn set_search(&mut self, search: String) {
        self.search = search;
        self.return_to_first_page();
    }

    pub fn set_sort(&mut self, column_key: String, ascending: bool) {
        self.sort_column_key = Some(column_key);
        self.sort_ascending = ascending;
        self.return_to_first_page();
    }

    #[must_use]
    pub fn filters_only(&self) -> Self {
        Self {
            search: self.search.clone(),
            sort_column_key: None,
            sort_ascending: true,
            column_filters: self.column_filters.clone(),
            date_range: self.date_range.clone(),
            skipped_rows: 0,
            rows_per_page: 0,
        }
    }

    pub fn show_month(&mut self, column_key: &str, from: String, to: String) {
        self.date_range = Some(DateRange {
            column_key: column_key.to_owned(),
            from,
            to,
        });
        self.return_to_first_page();
    }

    pub fn clear_date_range(&mut self) {
        self.date_range = None;
        self.return_to_first_page();
    }

    #[must_use]
    pub fn selected_filter_values(&self, column_key: &str) -> Option<&BTreeSet<String>> {
        self.column_filters.get(column_key)
    }

    #[must_use]
    pub fn is_column_filtered(&self, column_key: &str) -> bool {
        self.column_filters.contains_key(column_key)
    }

    #[must_use]
    pub fn filtered_column_count(&self) -> usize {
        self.column_filters.len()
    }

    pub fn set_column_filter(&mut self, column_key: &str, values: BTreeSet<String>) {
        self.column_filters.insert(column_key.to_string(), values);
        self.return_to_first_page();
    }

    pub fn deselect_every_value(&mut self, column_key: &str) {
        self.set_column_filter(column_key, BTreeSet::new());
    }

    pub fn clear_column_filter(&mut self, column_key: &str) {
        self.column_filters.remove(column_key);
        self.return_to_first_page();
    }

    pub fn clear_all_column_filters(&mut self) {
        self.column_filters.clear();
        self.return_to_first_page();
    }

    /// Drops the sort, filters and date range on columns outside
    /// `column_keys`, so no hidden column silently shapes the rows.
    pub fn keep_only_columns(&mut self, column_keys: &[&str]) {
        let is_kept = |column_key: &str| column_keys.contains(&column_key);
        let before = self.clone();
        if self
            .sort_column_key
            .as_deref()
            .is_some_and(|key| !is_kept(key))
        {
            self.sort_column_key = None;
            self.sort_ascending = true;
        }
        self.column_filters
            .retain(|column_key, _| is_kept(column_key));
        if self
            .date_range
            .as_ref()
            .is_some_and(|range| !is_kept(&range.column_key))
        {
            self.date_range = None;
        }
        if *self != before {
            self.return_to_first_page();
        }
    }

    /// What one column's selection asks of the server.
    #[must_use]
    pub fn column_selection(&self, column_key: &str) -> ColumnSelection {
        match self.column_filters.get(column_key) {
            None => ColumnSelection::Unfiltered,
            Some(values) if values.is_empty() => ColumnSelection::MatchesNothing,
            Some(values) => ColumnSelection::AnyOf(
                values
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<&str>>()
                    .join(&FILTER_VALUE_SEPARATOR.to_string()),
            ),
        }
    }

    pub fn advance_to_next_page(&mut self) {
        self.skipped_rows = self.skipped_rows.saturating_add(self.rows_per_page);
    }

    pub fn return_to_previous_page(&mut self) {
        self.skipped_rows = self.skipped_rows.saturating_sub(self.rows_per_page);
    }

    #[must_use]
    pub fn is_on_first_page(&self) -> bool {
        self.skipped_rows == 0
    }

    #[must_use]
    pub fn first_row_number(&self, rows_on_page: usize) -> u32 {
        if rows_on_page == 0 {
            0
        } else {
            self.skipped_rows.saturating_add(1)
        }
    }

    #[must_use]
    pub fn last_row_number(&self, rows_on_page: usize) -> u32 {
        self.skipped_rows
            .saturating_add(u32::try_from(rows_on_page).unwrap_or(u32::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query_filtered_by(column_key: &str, values: &[&str]) -> TableQuery {
        let mut query = TableQuery::with_rows_per_page(50);
        query.set_column_filter(
            column_key,
            values.iter().map(|value| (*value).to_string()).collect(),
        );
        query
    }

    #[test]
    fn survives_an_encoding_that_dropped_the_empty_filter_map() {
        let without_filters = r#"{
            "search": "",
            "sort_column_key": null,
            "sort_ascending": true,
            "skipped_rows": 0,
            "rows_per_page": 50
        }"#;

        let query: TableQuery = serde_json::from_str(without_filters).unwrap();

        assert_eq!(query, TableQuery::with_rows_per_page(50));
    }

    #[test]
    fn joins_selected_values_with_the_separator() {
        let query = query_filtered_by("warehouse", &["Houston, TX", "Paramount, CA"]);

        assert_eq!(
            query.column_selection("warehouse"),
            ColumnSelection::AnyOf("Houston, TX\u{1f}Paramount, CA".to_string())
        );
    }

    #[test]
    fn asks_for_nothing_for_an_unfiltered_column() {
        let query = TableQuery::with_rows_per_page(50);

        assert_eq!(
            query.column_selection("warehouse"),
            ColumnSelection::Unfiltered
        );
    }

    #[test]
    fn deselecting_every_value_is_not_the_same_as_being_unfiltered() {
        let mut query = TableQuery::with_rows_per_page(50);
        query.deselect_every_value("port");

        assert!(query.is_column_filtered("port"));
        assert_eq!(
            query.column_selection("port"),
            ColumnSelection::MatchesNothing
        );
    }

    #[test]
    fn selecting_only_the_blank_value_is_not_deselecting_everything() {
        let query = query_filtered_by("freight_forwarder", &[""]);

        assert_eq!(
            query.column_selection("freight_forwarder"),
            ColumnSelection::AnyOf(String::new())
        );
    }

    #[test]
    fn filtering_returns_to_the_first_page() {
        let mut query = TableQuery::with_rows_per_page(50);
        query.advance_to_next_page();

        query.set_column_filter("port", BTreeSet::from(["LB: PCT".to_string()]));

        assert!(query.is_on_first_page());
    }

    #[test]
    fn keeping_only_shown_columns_drops_hidden_sorts_and_filters() {
        let mut query = query_filtered_by("inventory_uom", &["SF"]);
        query.set_column_filter("item_size", BTreeSet::from(["12x12".to_string()]));
        query.set_sort("on_hand".to_string(), false);
        query.advance_to_next_page();

        query.keep_only_columns(&["item_code", "item_size"]);

        assert_eq!(query.sort_column_key, None);
        assert!(query.sort_ascending);
        assert!(!query.is_column_filtered("inventory_uom"));
        assert!(query.is_column_filtered("item_size"));
        assert!(query.is_on_first_page());
    }

    #[test]
    fn keeping_every_shown_column_stays_on_the_same_page() {
        let mut query = query_filtered_by("item_size", &["12x12"]);
        query.set_sort("item_code".to_string(), true);
        query.advance_to_next_page();
        let before = query.clone();

        query.keep_only_columns(&["item_code", "item_size"]);

        assert_eq!(query, before);
    }

    #[test]
    fn selecting_the_same_values_in_a_different_order_compares_equal() {
        assert_eq!(
            query_filtered_by("port", &["Houston", "Savannah"]),
            query_filtered_by("port", &["Savannah", "Houston"])
        );
    }
}
