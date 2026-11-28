use std::collections::BTreeSet;

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};
use leptos_router::location::Url;
use leptos_router::params::ParamsMap;
use leptos_router::NavigateOptions;

use table_query::{DateRange, TableQuery, FILTER_VALUE_SEPARATOR};

const SEARCH_PARAMETER: &str = "search";
const SORT_PARAMETER: &str = "sort";
const ORDER_PARAMETER: &str = "order";
const PAGE_PARAMETER: &str = "page";
const DATE_COLUMN_PARAMETER: &str = "date_column";
const DATE_FROM_PARAMETER: &str = "date_from";
const DATE_TO_PARAMETER: &str = "date_to";
const FILTER_PARAMETER_PREFIX: &str = "filter.";
const DESCENDING: &str = "descending";

const PLAIN_PARAMETERS: [&str; 7] = [
    SEARCH_PARAMETER,
    SORT_PARAMETER,
    ORDER_PARAMETER,
    PAGE_PARAMETER,
    DATE_COLUMN_PARAMETER,
    DATE_FROM_PARAMETER,
    DATE_TO_PARAMETER,
];

#[derive(Clone, Copy)]
struct ParameterNames {
    namespace: &'static str,
}

impl ParameterNames {
    fn name(self, parameter: &str) -> String {
        if self.namespace.is_empty() {
            parameter.to_string()
        } else {
            format!("{}.{parameter}", self.namespace)
        }
    }

    fn filter_name(self, column_key: &str) -> String {
        self.name(&format!("{FILTER_PARAMETER_PREFIX}{column_key}"))
    }

    fn filter_column_key(self, parameter: &str) -> Option<&str> {
        let unprefixed = if self.namespace.is_empty() {
            parameter
        } else {
            parameter.strip_prefix(self.namespace)?.strip_prefix('.')?
        };
        unprefixed.strip_prefix(FILTER_PARAMETER_PREFIX)
    }

    fn owns(self, parameter: &str) -> bool {
        PLAIN_PARAMETERS
            .iter()
            .any(|name| self.name(name) == parameter)
            || self.filter_column_key(parameter).is_some()
    }
}

fn empty_selection_sentinel() -> String {
    FILTER_VALUE_SEPARATOR.to_string()
}

fn decode_filter_values(values: Vec<String>) -> BTreeSet<String> {
    if values.len() == 1 && values.contains(&empty_selection_sentinel()) {
        return BTreeSet::new();
    }
    values.into_iter().collect()
}

fn page_number(query: &TableQuery) -> u32 {
    query
        .skipped_rows
        .checked_div(query.rows_per_page)
        .unwrap_or(0)
        .saturating_add(1)
}

fn parse_query(parameters: &ParamsMap, names: ParameterNames, rows_per_page: u32) -> TableQuery {
    let mut query = TableQuery::with_rows_per_page(rows_per_page);
    query.search = parameters
        .get(&names.name(SEARCH_PARAMETER))
        .unwrap_or_default();
    query.sort_column_key = parameters.get(&names.name(SORT_PARAMETER));
    query.sort_ascending = parameters
        .get_str(&names.name(ORDER_PARAMETER))
        .is_none_or(|order| order != DESCENDING);
    let requested_page = parameters
        .get_str(&names.name(PAGE_PARAMETER))
        .and_then(|page| page.parse::<u32>().ok())
        .unwrap_or(1);
    query.skipped_rows = requested_page
        .saturating_sub(1)
        .checked_mul(rows_per_page)
        .unwrap_or(0);
    let filter_parameters: Vec<(String, String)> = parameters
        .latest_values()
        .filter_map(|(parameter, _)| {
            names
                .filter_column_key(parameter)
                .map(|column_key| (parameter.to_string(), column_key.to_string()))
        })
        .collect();
    for (parameter, column_key) in filter_parameters {
        let values = parameters.get_all(&parameter).unwrap_or_default();
        query
            .column_filters
            .insert(column_key, decode_filter_values(values));
    }
    if let (Some(column_key), Some(from), Some(to)) = (
        parameters.get(&names.name(DATE_COLUMN_PARAMETER)),
        parameters.get(&names.name(DATE_FROM_PARAMETER)),
        parameters.get(&names.name(DATE_TO_PARAMETER)),
    ) {
        query.date_range = Some(DateRange {
            column_key,
            from,
            to,
        });
    }
    query
}

fn insert_exactly(parameters: &mut ParamsMap, name: String, value: &str) {
    parameters.insert(name, Url::escape(value));
}

fn write_query(parameters: &mut ParamsMap, query: &TableQuery, names: ParameterNames) {
    let owned: Vec<String> = parameters
        .latest_values()
        .map(|(parameter, _)| parameter.to_string())
        .filter(|parameter| names.owns(parameter))
        .collect();
    for parameter in owned {
        parameters.remove(&parameter);
    }
    if !query.search.is_empty() {
        insert_exactly(parameters, names.name(SEARCH_PARAMETER), &query.search);
    }
    if let Some(sort_column_key) = &query.sort_column_key {
        insert_exactly(parameters, names.name(SORT_PARAMETER), sort_column_key);
        if !query.sort_ascending {
            insert_exactly(parameters, names.name(ORDER_PARAMETER), DESCENDING);
        }
    }
    if page_number(query) > 1 {
        insert_exactly(
            parameters,
            names.name(PAGE_PARAMETER),
            &page_number(query).to_string(),
        );
    }
    for (column_key, values) in &query.column_filters {
        let parameter = names.filter_name(column_key);
        if values.is_empty() {
            insert_exactly(parameters, parameter, &empty_selection_sentinel());
        } else {
            for value in values {
                insert_exactly(parameters, parameter.clone(), value);
            }
        }
    }
    if let Some(range) = &query.date_range {
        insert_exactly(
            parameters,
            names.name(DATE_COLUMN_PARAMETER),
            &range.column_key,
        );
        insert_exactly(parameters, names.name(DATE_FROM_PARAMETER), &range.from);
        insert_exactly(parameters, names.name(DATE_TO_PARAMETER), &range.to);
    }
}

pub fn rewrite_table_query_parameters(
    parameters: &mut ParamsMap,
    rows_per_page: u32,
    rewrite: impl FnOnce(&mut TableQuery),
) {
    let names = ParameterNames { namespace: "" };
    let mut query = parse_query(parameters, names, rows_per_page);
    rewrite(&mut query);
    write_query(parameters, &query, names);
}

fn is_table_parameter(parameter: &str) -> bool {
    let is_plain = PLAIN_PARAMETERS.iter().any(|plain| {
        parameter == *plain
            || parameter
                .strip_suffix(plain)
                .is_some_and(|namespace| namespace.ends_with('.'))
    });
    let is_filter = parameter.starts_with(FILTER_PARAMETER_PREFIX)
        || parameter.contains(&format!(".{FILTER_PARAMETER_PREFIX}"));
    is_plain || is_filter
}

pub fn clear_table_query_parameters(parameters: &mut ParamsMap) {
    let table_parameters: Vec<String> = parameters
        .latest_values()
        .map(|(parameter, _)| parameter.to_string())
        .filter(|parameter| is_table_parameter(parameter))
        .collect();
    for parameter in table_parameters {
        parameters.remove(&parameter);
    }
}

type Navigate = std::sync::Arc<dyn Fn(&str, NavigateOptions) + Send + Sync>;

#[derive(Clone, Copy)]
pub struct UrlTableQuery {
    current: Memo<TableQuery>,
    parameters: Memo<ParamsMap>,
    pathname: Memo<String>,
    names: ParameterNames,
    navigate: StoredValue<Navigate>,
}

impl UrlTableQuery {
    fn is_gone(self) -> bool {
        self.current.try_with_untracked(|_| ()).is_none()
    }

    #[must_use]
    pub fn get(self) -> TableQuery {
        self.current.try_get().unwrap_or_default()
    }

    #[must_use]
    pub fn get_untracked(self) -> TableQuery {
        self.current.try_get_untracked().unwrap_or_default()
    }

    pub fn with<Output>(self, read: impl FnOnce(&TableQuery) -> Output) -> Output {
        if self.is_gone() {
            return read(&TableQuery::default());
        }
        self.current.with(read)
    }

    pub fn with_untracked<Output>(self, read: impl FnOnce(&TableQuery) -> Output) -> Output {
        if self.is_gone() {
            return read(&TableQuery::default());
        }
        self.current.with_untracked(read)
    }

    pub fn set(self, query: &TableQuery) {
        let Some(current) = self.parameters.try_get_untracked() else {
            return;
        };
        let Some(pathname) = self.pathname.try_get_untracked() else {
            return;
        };
        let mut updated = current.clone();
        write_query(&mut updated, query, self.names);
        let updated_string = updated.to_query_string();
        if updated_string == current.to_query_string() {
            return;
        }
        let target = format!("{pathname}{updated_string}");
        self.navigate.try_with_value(|navigate| {
            navigate(
                &target,
                NavigateOptions {
                    replace: true,
                    scroll: false,
                    ..Default::default()
                },
            );
        });
    }

    pub fn update(self, change: impl FnOnce(&mut TableQuery)) {
        let mut query = self.get_untracked();
        change(&mut query);
        self.set(&query);
    }
}

#[must_use]
pub fn url_synced_table_query(rows_per_page: u32) -> UrlTableQuery {
    url_synced_table_query_in_namespace(rows_per_page, "")
}

#[must_use]
pub fn url_synced_table_query_in_namespace(
    rows_per_page: u32,
    namespace: &'static str,
) -> UrlTableQuery {
    let names = ParameterNames { namespace };
    let parameters = use_query_map();
    let navigate = use_navigate();
    UrlTableQuery {
        current: Memo::new(move |_| {
            parameters.with(|parameters| parse_query(parameters, names, rows_per_page))
        }),
        parameters,
        pathname: use_location().pathname,
        names,
        navigate: StoredValue::new(std::sync::Arc::new(navigate)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use table_query::strategies::table_query;

    const NAMESPACES: [&str; 2] = ["", "lines"];
    const UNRELATED_PARAMETER: &str = "tab";

    fn write_then_read(
        query: &TableQuery,
        namespace: &'static str,
        parameters: &mut ParamsMap,
    ) -> TableQuery {
        let names = ParameterNames { namespace };
        write_query(parameters, query, names);
        parse_query(parameters, names, query.rows_per_page)
    }

    proptest! {
        #[test]
        fn a_query_reads_back_from_the_address_unchanged(query in table_query()) {
            for namespace in NAMESPACES {
                prop_assert_eq!(&write_then_read(&query, namespace, &mut ParamsMap::new()), &query);
            }
        }

        #[test]
        fn writing_a_query_replaces_the_last_and_keeps_other_parameters(
            first in table_query(),
            mut second in table_query(),
        ) {
            second.rows_per_page = first.rows_per_page;
            second.skipped_rows = second
                .skipped_rows
                .checked_div(second.rows_per_page)
                .unwrap_or(0)
                .saturating_mul(first.rows_per_page);
            for namespace in NAMESPACES {
                let mut parameters = ParamsMap::new();
                parameters.insert(UNRELATED_PARAMETER, "lines".to_owned());
                write_then_read(&first, namespace, &mut parameters);
                prop_assert_eq!(&write_then_read(&second, namespace, &mut parameters), &second);
                prop_assert_eq!(parameters.get(UNRELATED_PARAMETER), Some("lines".to_owned()));
            }
        }
    }

    const ROWS_PER_PAGE: u32 = 50;

    fn round_trip(query: &TableQuery, names: ParameterNames) -> TableQuery {
        let mut parameters = ParamsMap::new();
        write_query(&mut parameters, query, names);
        parse_query(&parameters, names, ROWS_PER_PAGE)
    }

    fn unprefixed() -> ParameterNames {
        ParameterNames { namespace: "" }
    }

    #[test]
    fn a_default_query_writes_no_parameters() {
        let mut parameters = ParamsMap::new();
        write_query(
            &mut parameters,
            &TableQuery::with_rows_per_page(ROWS_PER_PAGE),
            unprefixed(),
        );
        assert_eq!(parameters.to_query_string(), "");
    }

    #[test]
    fn search_sort_filters_and_page_survive_the_round_trip() {
        let mut query = TableQuery::with_rows_per_page(ROWS_PER_PAGE);
        query.set_search("carrara".to_string());
        query.set_sort("vendor_name".to_string(), false);
        query.set_column_filter(
            "warehouse",
            ["Houston, TX".to_string(), String::new()].into(),
        );
        query.advance_to_next_page();

        assert_eq!(round_trip(&query, unprefixed()), query);
    }

    #[test]
    fn a_selection_of_no_values_survives_the_round_trip() {
        let mut query = TableQuery::with_rows_per_page(ROWS_PER_PAGE);
        query.deselect_every_value("port");

        assert_eq!(round_trip(&query, unprefixed()), query);
    }

    #[test]
    fn a_namespace_keeps_two_tables_apart() {
        let names = ParameterNames {
            namespace: "invoices",
        };
        let mut query = TableQuery::with_rows_per_page(ROWS_PER_PAGE);
        query.set_search("acme".to_string());
        query.set_column_filter("vendor", ["ACME".to_string()].into());

        let mut parameters = ParamsMap::new();
        write_query(&mut parameters, &query, names);

        assert_eq!(round_trip(&query, names), query);
        assert_eq!(
            parse_query(&parameters, unprefixed(), ROWS_PER_PAGE),
            TableQuery::with_rows_per_page(ROWS_PER_PAGE)
        );
    }

    #[test]
    fn foreign_parameters_survive_a_write() {
        let mut parameters = ParamsMap::new();
        parameters.insert("type", "C".to_string());
        let mut query = TableQuery::with_rows_per_page(ROWS_PER_PAGE);
        query.set_search("marble".to_string());

        write_query(&mut parameters, &query, unprefixed());

        assert_eq!(parameters.get_str("type"), Some("C"));
    }

    #[test]
    fn clearing_removes_every_table_state_and_keeps_the_rest() {
        let mut query = TableQuery::with_rows_per_page(ROWS_PER_PAGE);
        query.set_search("marble".to_string());
        query.set_sort("on_hand".to_string(), false);
        query.set_column_filter("item_material", ["MRBL".to_string()].into());
        query.show_month("eta", "2026-09-01".to_string(), "2026-09-30".to_string());
        query.advance_to_next_page();
        let mut parameters = ParamsMap::new();
        parameters.insert("view", "in-stock".to_string());
        parameters.insert("mode", "inventory".to_string());
        write_query(&mut parameters, &query, unprefixed());
        write_query(
            &mut parameters,
            &query,
            ParameterNames {
                namespace: "open-lines",
            },
        );

        clear_table_query_parameters(&mut parameters);

        assert_eq!(parameters.get_str("view"), Some("in-stock"));
        assert_eq!(parameters.get_str("mode"), Some("inventory"));
        assert_eq!(parameters.latest_values().count(), 2);
    }

    #[test]
    fn the_date_range_survives_the_round_trip() {
        let mut query = TableQuery::with_rows_per_page(ROWS_PER_PAGE);
        query.show_month("eta", "2026-09-01".to_string(), "2026-09-30".to_string());

        assert_eq!(round_trip(&query, unprefixed()), query);
    }
}
