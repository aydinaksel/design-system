use design_system::table::{
    facet_values_loader, url_synced_table_query, CalendarView, Column, FacetControls, TableQuery,
};
use design_system::PageHeader;
use leptos::prelude::*;

use super::tables::{load_stone_facets, load_stone_page};
use super::STONE_FACETS;
use crate::fake_data::{Stone, StoneView};

const MONTH_ROW_CAP: u32 = 500;

fn day_columns() -> Vec<Column<Stone>> {
    vec![
        Column::text("code", "Code", |stone: &Stone| stone.code.clone()),
        Column::text("name", "Name", |stone: &Stone| stone.name.clone()),
        Column::text("warehouse", "Warehouse", |stone: &Stone| {
            stone.warehouse.clone()
        }),
    ]
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn CalendarPage() -> impl IntoView {
    let table_query = url_synced_table_query(super::ROWS_PER_PAGE);
    let facet_values = facet_values_loader(
        move |query: TableQuery| load_stone_facets(StoneView::All, query),
        table_query,
    );
    let month_rows = LocalResource::new(move || {
        let mut month = table_query.get();
        month.skipped_rows = 0;
        month.rows_per_page = MONTH_ROW_CAP;
        async move {
            load_stone_page(StoneView::All, month)
                .await
                .map(|page| page.rows)
                .map_err(|error| error.to_string())
        }
    });
    let rows = Signal::derive(move || month_rows.get());
    let facet_controls = FacetControls::new(STONE_FACETS.to_vec(), table_query, facet_values, None);

    view! {
        <main class="mx-auto max-w-screen-2xl px-6 py-8">
            <PageHeader title="Calendar" />
            {facet_controls.toggle_button("mb-4")}
            <div class="flex items-start gap-6">
                {facet_controls.rail()} <div class="min-w-0 flex-1">
                    <CalendarView
                        rows=rows
                        columns=day_columns()
                        date_of_row=std::sync::Arc::new(|stone: &Stone| {
                            Some(stone.arrival.clone())
                        })
                        query=table_query
                        date_column_key="arrival"
                        empty_message="Nothing arrives that day."
                    />
                </div>
            </div>
        </main>
    }
    .into_any()
}
