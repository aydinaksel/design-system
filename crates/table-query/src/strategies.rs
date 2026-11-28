use proptest::collection::{btree_map, btree_set};
use proptest::option;
use proptest::prelude::*;

use crate::{DateRange, TableQuery, FILTER_VALUE_SEPARATOR};

const MOST_FILTERED_COLUMNS: usize = 4;
const MOST_SELECTED_VALUES: usize = 4;
const MOST_PAGES: u32 = 50;
const MOST_ROWS_PER_PAGE: u32 = 200;

pub fn column_key() -> impl Strategy<Value = String> {
    "[a-z][a-z_.]{0,8}"
}

pub fn filter_value() -> impl Strategy<Value = String> {
    any::<String>().prop_filter("a value never holds the separator", |value| {
        !value.contains(FILTER_VALUE_SEPARATOR)
    })
}

fn date_range() -> impl Strategy<Value = DateRange> {
    (column_key(), any::<String>(), any::<String>()).prop_map(|(column_key, from, to)| DateRange {
        column_key,
        from,
        to,
    })
}

pub fn table_query() -> impl Strategy<Value = TableQuery> {
    let filters = btree_map(
        column_key(),
        btree_set(filter_value(), 0..=MOST_SELECTED_VALUES),
        0..=MOST_FILTERED_COLUMNS,
    );
    (
        any::<String>(),
        option::of(column_key()),
        any::<bool>(),
        filters,
        option::of(date_range()),
        0..MOST_PAGES,
        1..=MOST_ROWS_PER_PAGE,
    )
        .prop_map(
            |(
                search,
                sort_column_key,
                descending,
                column_filters,
                date_range,
                page,
                rows_per_page,
            )| {
                let sort_ascending = sort_column_key.is_none() || !descending;
                TableQuery {
                    search,
                    sort_column_key,
                    sort_ascending,
                    column_filters,
                    date_range,
                    skipped_rows: page.saturating_mul(rows_per_page),
                    rows_per_page,
                }
            },
        )
}
