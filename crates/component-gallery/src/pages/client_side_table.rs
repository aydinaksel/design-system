use design_system::table::{Column, DataTable, Search};
use design_system::PageHeader;
use leptos::prelude::*;

use crate::fake_data::Stone;

#[cfg(feature = "ssr")]
const CLIENT_SIDE_TABLE_ROW_COUNT: usize = 200;

#[server]
pub async fn load_all_stones() -> Result<Vec<Stone>, ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_millis(
        super::SIMULATED_LATENCY_MILLISECONDS,
    ))
    .await;
    Ok(crate::fake_data::generated::stones()
        .into_iter()
        .take(CLIENT_SIDE_TABLE_ROW_COUNT)
        .collect())
}

fn stone_columns() -> Vec<Column<Stone>> {
    vec![
        Column::text("code", "Code", |stone: &Stone| stone.code.clone())
            .sortable()
            .searchable(),
        Column::text("name", "Name", |stone: &Stone| stone.name.clone())
            .sortable()
            .searchable(),
        Column::text("warehouse", "Warehouse", |stone: &Stone| {
            stone.warehouse.clone()
        }),
        Column::quantity("on_hand", "On Hand", |stone: &Stone| Some(stone.on_hand)).sortable(),
    ]
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ClientSideTablePage() -> impl IntoView {
    let stones = Resource::new(|| (), |()| load_all_stones());
    let rows = Signal::derive(move || {
        stones
            .get()
            .map(|result| result.map_err(|error| error.to_string()))
    });

    view! {
        <main class="mx-auto max-w-screen-2xl px-6 py-8">
            <PageHeader title="Client-Side Table" />
            <DataTable
                rows=rows
                columns=stone_columns()
                search=Search::InMemory
                search_placeholder="Search Stones"
                empty_message="No stones found."
            />
        </main>
    }
    .into_any()
}
