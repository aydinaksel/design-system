use design_system::table::{
    filter_values_loader, url_synced_table_query, Column, FilterValue, Page, ServerTable,
    TableQuery,
};
use design_system::{DatePicker, PageHeader, Switch};
use leptos::prelude::*;

use super::ROWS_PER_PAGE;
use crate::fake_data::Stone;

const ARRIVAL_COLUMN: &str = "arrival";
const LAST_DATE: &str = "9999-12-31";

#[server]
pub async fn load_filtered_stone_page(
    table_query: TableQuery,
    fails: bool,
) -> Result<Page<Stone>, ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_millis(
        super::SIMULATED_LATENCY_MILLISECONDS,
    ))
    .await;
    if fails {
        crate::deliberate_failure::mark_response_as_deliberate_failure();
        return Err(ServerFnError::new("the stone database did not answer"));
    }
    Ok(crate::fake_data::generated::page(
        crate::fake_data::StoneView::All,
        &table_query,
    ))
}

#[server]
pub async fn load_stone_filter_values(
    column_key: String,
    table_query: TableQuery,
) -> Result<Vec<FilterValue>, ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_millis(
        super::SIMULATED_LATENCY_MILLISECONDS,
    ))
    .await;
    Ok(
        crate::fake_data::generated::facets(crate::fake_data::StoneView::All, &table_query)
            .into_iter()
            .find(|facet| facet.column_key == column_key)
            .map(|facet| facet.values)
            .unwrap_or_default(),
    )
}

fn stone_columns() -> Vec<Column<Stone>> {
    vec![
        Column::text("code", "Code", |stone: &Stone| stone.code.clone()).sortable(),
        Column::text("name", "Name", |stone: &Stone| stone.name.clone()).sortable(),
        Column::text("material", "Material", |stone: &Stone| {
            stone.material.clone()
        })
        .filterable(),
        Column::text("finish", "Finish", |stone: &Stone| stone.finish.clone()).filterable(),
        Column::text("warehouse", "Warehouse", |stone: &Stone| {
            stone.warehouse.clone()
        })
        .filterable(),
        Column::text(ARRIVAL_COLUMN, "Arrival", |stone: &Stone| {
            stone.arrival.clone()
        })
        .sortable(),
        Column::quantity("on_hand", "On Hand", |stone: &Stone| Some(stone.on_hand)).sortable(),
    ]
}

#[component]
pub fn QuickFiltersPage() -> impl IntoView {
    let table_query = url_synced_table_query(ROWS_PER_PAGE);
    let loads_fail = RwSignal::new(false);
    let reload = RwSignal::new(0_u32);
    let arriving_from = RwSignal::new(String::new());
    Effect::new(move |_| {
        let first_date = arriving_from.get();
        table_query.update(|query| {
            if first_date.is_empty() {
                query.clear_date_range();
            } else {
                query.show_month(ARRIVAL_COLUMN, first_date, LAST_DATE.to_owned());
            }
        });
    });
    let load = move |query: TableQuery| load_filtered_stone_page(query, loads_fail.get_untracked());
    let arrival_filter = move || {
        view! { <DatePicker value=arriving_from placeholder="Arriving From" clearable=true /> }
    };
    view! {
        <main class="mx-auto max-w-screen-2xl px-6 py-8">
            <PageHeader title="Quick Filters" />
            <div class="mb-4 flex items-center gap-3 text-sm text-gray-700">
                <Switch
                    checked=loads_fail
                    on_toggle=Callback::new(move |()| {
                        loads_fail.update(|fails| *fails = !*fails);
                        reload.update(|count| *count = count.wrapping_add(1));
                    })
                    label="Make Loading Fail"
                />
                "Make Loading Fail"
            </div>
            <ServerTable
                columns=stone_columns()
                load=load
                query=table_query
                reload=reload
                rows_per_page=ROWS_PER_PAGE
                search_placeholder="Search Stones"
                empty_message="No stones match."
                load_filter_values=filter_values_loader(load_stone_filter_values)
                filters=arrival_filter
            />
        </main>
    }
    .into_any()
}
