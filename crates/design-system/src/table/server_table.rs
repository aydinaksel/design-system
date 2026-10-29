use std::future::Future;

use leptos::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use table_query::Page;

use super::column::Column;
use super::data_table::{DataTable, RowAttribute, RowView, Search};
use super::server_facet_panel::{FacetControls, ServerFacet};
use super::url_query::UrlTableQuery;
use table_query::TableQuery;
use table_query::{FilterValue, LoadedFacetValues, LoadedFilterValues};

type LoadedRows<Row> = Signal<Option<Result<Vec<Row>, String>>>;

type FilterValuesFuture =
    std::pin::Pin<Box<dyn Future<Output = Result<Vec<FilterValue>, String>> + Send>>;

/// Loads the values a column offers, under everything in the query except that
/// column's own selection. Build one with [`filter_values_loader`].
pub type FilterValuesLoader =
    std::sync::Arc<dyn Fn(String, TableQuery) -> FilterValuesFuture + Send + Sync>;

pub fn filter_values_loader<LoadFn, LoadFuture, LoadError>(load: LoadFn) -> FilterValuesLoader
where
    LoadFn: Fn(String, TableQuery) -> LoadFuture + Clone + Send + Sync + 'static,
    LoadFuture: Future<Output = Result<Vec<FilterValue>, LoadError>> + Send + 'static,
    LoadError: ToString,
{
    std::sync::Arc::new(move |column_key, query| {
        let load = load.clone();
        Box::pin(async move {
            load(column_key, query)
                .await
                .map_err(|error| error.to_string())
        })
    })
}

fn requested_filter_values(
    load_filter_values: Option<FilterValuesLoader>,
    table_query: UrlTableQuery,
    filter_column_request: RwSignal<Option<String>>,
) -> Signal<LoadedFilterValues> {
    let Some(loader) = load_filter_values else {
        return Signal::derive(|| None);
    };
    let requested = Resource::new(
        move || {
            let column_key = filter_column_request.get();
            let mut query = table_query.with(TableQuery::filters_only);
            if let Some(open_column) = column_key.as_deref() {
                query.column_filters.remove(open_column);
            }
            (column_key, query)
        },
        move |(column_key, query)| {
            let loader = loader.clone();
            async move {
                match column_key {
                    Some(column_key) => Some(loader(column_key, query).await),
                    None => None,
                }
            }
        },
    );
    Signal::derive(move || requested.try_get().flatten().flatten())
}

/// A [`DataTable`] fed one [`Page`] at a time by a server function.
#[component]
pub fn ServerTable<Row, LoadFn, LoadFuture, LoadError>(
    columns: Vec<Column<Row>>,
    load: LoadFn,
    rows_per_page: u32,
    #[prop(optional, into)] search_placeholder: String,
    #[prop(optional, into)] empty_message: String,
    #[prop(optional)] row_href: Option<RowAttribute<Row>>,
    #[prop(optional)] row_class: Option<RowAttribute<Row>>,
    #[prop(optional, into)] toolbar: ViewFn,
    /// Enables the header filter popover on `filterable` columns.
    #[prop(optional)]
    load_filter_values: Option<FilterValuesLoader>,
    #[prop(optional)] facets: Option<Vec<ServerFacet>>,
    #[prop(optional, into)] facet_values: Option<Signal<LoadedFacetValues>>,
    /// Share the query with a facet rail. Omit to let the table own it.
    #[prop(optional)]
    query: Option<UrlTableQuery>,
    /// Bump to re-read the current page, after an edit saves.
    #[prop(optional)]
    reload: Option<RwSignal<u32>>,
    /// Draws each row as a card in a grid in place of the table.
    #[prop(optional)]
    card: Option<RowView<Row>>,
) -> impl IntoView
where
    Row: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
    LoadFn: Fn(TableQuery) -> LoadFuture + Clone + Send + Sync + 'static,
    LoadFuture: Future<Output = Result<Page<Row>, LoadError>> + Send + 'static,
    LoadError: ToString,
{
    let table_query =
        query.unwrap_or_else(|| super::url_query::url_synced_table_query(rows_per_page));

    let current_page = Resource::new_blocking(
        move || (table_query.get(), reload.map(|reload| reload.get())),
        move |(query, _)| {
            let load = load.clone();
            async move { load(query).await.map_err(|error| error.to_string()) }
        },
    );

    let page_rows: LoadedRows<Row> = Signal::derive(move || {
        current_page
            .try_get()
            .flatten()
            .map(|result| result.map(|page| page.rows))
    });

    let total_rows = Signal::derive(move || {
        current_page
            .try_get()
            .flatten()
            .and_then(Result::ok)
            .map(|page| page.total_rows)
    });

    let (refreshing, set_refreshing) = signal(false);

    let row_href = row_href.unwrap_or_else(|| std::sync::Arc::new(|_| String::new()));
    let row_class = row_class.unwrap_or_else(|| std::sync::Arc::new(|_| String::new()));

    let filter_column_request: RwSignal<Option<String>> = RwSignal::new(None);
    let filtering_is_enabled = load_filter_values.is_some();
    let filter_values =
        requested_filter_values(load_filter_values, table_query, filter_column_request);
    let filter_column_request = filtering_is_enabled.then_some(filter_column_request);

    let facet_controls = facets
        .filter(|facets| !facets.is_empty())
        .zip(facet_values)
        .map(|(facets, facet_values)| {
            FacetControls::new(facets, table_query, facet_values, Some(total_rows))
        });
    let toolbar_with_filters = move || match facet_controls {
        Some(controls) => view! {
            {controls.toggle_button("")}
            {toolbar.run()}
        }
        .into_any(),
        None => toolbar.run(),
    };

    let table =
        move |rows: LoadedRows<Row>, total_rows: Signal<Option<u64>>, pending: Signal<bool>| {
            view! {
                <DataTable
                    rows=rows
                    columns=columns.clone()
                    search=Search::Server
                    search_placeholder=search_placeholder.clone()
                    empty_message=empty_message.clone()
                    row_href=row_href.clone()
                    row_class=row_class.clone()
                    toolbar=Some(toolbar_with_filters())
                    sidebar=facet_controls.map(FacetControls::rail)
                    query=table_query
                    total_rows=total_rows
                    pending=pending
                    filter_column_request=filter_column_request
                    filter_values=filter_values
                    card=card.clone()
                />
            }
            .into_any()
        };

    let skeleton = table.clone();
    view! {
        <Transition
            fallback=move || {
                skeleton(Signal::derive(|| None), Signal::derive(|| None), Signal::derive(|| false))
            }
            set_pending=set_refreshing
        >
            {move || table(page_rows, total_rows, refreshing.into())}
        </Transition>
    }
    .into_any()
}
