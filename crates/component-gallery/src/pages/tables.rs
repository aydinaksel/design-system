use std::sync::Arc;

use design_system::formatting::format_quantity;
use design_system::icons::LucideGlyph;
use design_system::table::{
    facet_values_loader, rewrite_table_query_parameters, url_synced_table_query, Column,
    FacetValues, Page, RowView, ServerTable, TableQuery,
};
use design_system::{
    Card, ChoiceTransition, ModeSwitch, PageHeader, PageMode, PageView, ResponsiveImage, ViewTabs,
};
use leptos::prelude::*;

use super::{ROWS_PER_PAGE, STONE_FACETS};
use crate::fake_data::{Stone, StoneView};

const TABLE_MODE: &str = "table";
const CARDS_MODE: &str = "cards";

const STONE_MODES: &[PageMode] = &[
    PageMode::new(TABLE_MODE, "Table View", LucideGlyph::Table),
    PageMode::new(CARDS_MODE, "Card View", LucideGlyph::LayoutGrid),
];

const STONE_VIEWS: &[PageView] = &[
    PageView::new(StoneView::All.key(), "All"),
    PageView::new(StoneView::InStock.key(), "In Stock"),
    PageView::new(StoneView::OnOrder.key(), "On Order"),
];

const TABLE_SORT_KEYS: [&str; 6] = [
    "code",
    "name",
    "on_hand",
    "committed",
    "on_order",
    "arrival",
];

const CARD_IMAGE_SIZES: &str =
    "(min-width: 1280px) 18rem, (min-width: 1024px) 25vw, (min-width: 640px) 45vw, 90vw";

#[server]
pub async fn load_stone_page(
    view: StoneView,
    table_query: TableQuery,
) -> Result<Page<Stone>, ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_millis(
        super::SIMULATED_LATENCY_MILLISECONDS,
    ))
    .await;
    Ok(crate::fake_data::generated::page(view, &table_query))
}

#[server]
pub async fn load_stone_facets(
    view: StoneView,
    table_query: TableQuery,
) -> Result<Vec<FacetValues>, ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_millis(
        super::SIMULATED_LATENCY_MILLISECONDS,
    ))
    .await;
    Ok(crate::fake_data::generated::facets(view, &table_query))
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
        Column::text("unit", "Unit", |stone: &Stone| stone.unit.clone()),
        Column::quantity("on_hand", "On Hand", |stone: &Stone| Some(stone.on_hand)).sortable(),
        Column::quantity("committed", "Committed", |stone: &Stone| {
            Some(stone.committed)
        })
        .sortable(),
        Column::quantity("on_order", "On Order", |stone: &Stone| Some(stone.on_order)).sortable(),
        Column::text("arrival", "Arrival", |stone: &Stone| stone.arrival.clone()).sortable(),
    ]
}

fn stone_card() -> RowView<Stone> {
    Arc::new(|stone: &Stone| {
        let image = view! {
            <ResponsiveImage
                source=Some(stone.image.clone())
                alternative_text=stone.name.clone()
                sizes=CARD_IMAGE_SIZES
            />
        }
        .into_any();
        let name = stone.name.clone();
        let code = stone.code.clone();
        let attributes = format!("{} · {} · {}", stone.size, stone.color_family, stone.finish);
        let on_hand = format!("{} {} on hand", format_quantity(stone.on_hand), stone.unit);
        view! {
            <Card href=format!("#{}", stone.code) image=image>
                <p class="text-sm font-medium text-gray-900">{name}</p>
                <p class="mt-0.5 font-mono text-xs text-gray-500">{code}</p>
                <p class="mt-2 text-xs text-gray-600">{attributes}</p>
                <p class="mt-3 text-xs text-gray-500">{on_hand}</p>
            </Card>
        }
        .into_any()
    })
}

fn mode_column_keys(mode: &str) -> Vec<&'static str> {
    let sort_keys: &[&str] = if mode == CARDS_MODE {
        &["code"]
    } else {
        &TABLE_SORT_KEYS
    };
    sort_keys
        .iter()
        .copied()
        .chain(STONE_FACETS.iter().map(|facet| facet.column_key))
        .collect()
}

fn keep_only_the_mode_columns() -> ChoiceTransition {
    Arc::new(|mode, parameters| {
        let column_keys = mode_column_keys(mode);
        rewrite_table_query_parameters(parameters, ROWS_PER_PAGE, |query| {
            query.keep_only_columns(&column_keys);
        });
    })
}

fn stone_list(view: StoneView, mode: &'static str) -> AnyView {
    let table_query = url_synced_table_query(ROWS_PER_PAGE);
    let load = move |query: TableQuery| load_stone_page(view, query);
    let facet_values = facet_values_loader(
        move |query: TableQuery| load_stone_facets(view, query),
        table_query,
    );
    if mode == CARDS_MODE {
        return view! {
            <ServerTable
                columns=Vec::new()
                card=stone_card()
                load=load
                query=table_query
                rows_per_page=ROWS_PER_PAGE
                search_placeholder="Search by Code or Name"
                empty_message="No stones match."
                facets=STONE_FACETS.to_vec()
                facet_values=facet_values
            />
        }
        .into_any();
    }
    view! {
        <ServerTable
            columns=stone_columns()
            load=load
            query=table_query
            rows_per_page=ROWS_PER_PAGE
            search_placeholder="Search by Code or Name"
            empty_message="No stones match."
            facets=STONE_FACETS.to_vec()
            facet_values=facet_values
            toolbar=|| view! { <span class="text-sm text-gray-500">"Toolbar content"</span> }
        />
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn TablesPage() -> impl IntoView {
    let selected_view = design_system::selected_view(STONE_VIEWS);
    let selected_mode = design_system::selected_mode(STONE_MODES);

    view! {
        <main class="mx-auto max-w-screen-2xl px-6 py-8">
            <PageHeader title="Tables">
                <ModeSwitch modes=STONE_MODES transition=keep_only_the_mode_columns() />
            </PageHeader>
            <ViewTabs views=STONE_VIEWS />
            {move || stone_list(StoneView::from_key(selected_view.get()), selected_mode.get())}
        </main>
    }
    .into_any()
}
