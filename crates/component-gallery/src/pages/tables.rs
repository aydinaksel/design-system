use std::sync::Arc;

use design_system::formatting::format_quantity;
use design_system::icons::LucideGlyph;
use design_system::table::{
    facet_values_loader, rewrite_table_query_parameters, url_synced_table_query, Column,
    FacetValues, Layout, Page, RowView, ServerTable, TableQuery,
};
use design_system::{
    Card, ChoiceTransition, ListPage, MenuEntry, MenuItem, ModeSwitch, PageMode, PageView,
    ResponsiveImage,
};
use leptos::prelude::*;

use super::{ROWS_PER_PAGE, STONE_FACETS};
use crate::fake_data::{Stone, StoneView};

const TABLE_MODE: &str = "table";
const CARDS_MODE: &str = "cards";
const CALENDAR_MODE: &str = "calendar";
const ARRIVAL_COLUMN: &str = "arrival";

const STONE_MODES: &[PageMode] = &[
    PageMode::new(TABLE_MODE, "Table View", LucideGlyph::Table),
    PageMode::new(CARDS_MODE, "Card View", LucideGlyph::LayoutGrid),
    PageMode::new(CALENDAR_MODE, "Calendar View", LucideGlyph::CalendarDays),
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
            .width_characters(28)
            .sortable()
            .searchable(),
        Column::text("warehouse", "Warehouse", |stone: &Stone| {
            stone.warehouse.clone()
        }),
        Column::text("unit", "Unit", |stone: &Stone| stone.unit.clone()).width_characters(4),
        Column::quantity("on_hand", "On Hand", |stone: &Stone| Some(stone.on_hand)).sortable(),
        Column::quantity("committed", "Committed", |stone: &Stone| {
            Some(stone.committed)
        })
        .sortable(),
        Column::quantity("on_order", "On Order", |stone: &Stone| Some(stone.on_order)).sortable(),
        Column::text("arrival", "Arrival", |stone: &Stone| stone.arrival.clone())
            .width_characters(10)
            .sortable(),
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

fn calendar_columns() -> Vec<Column<Stone>> {
    vec![
        Column::text("code", "Code", |stone: &Stone| stone.code.clone()),
        Column::text("name", "Name", |stone: &Stone| stone.name.clone()).width_characters(28),
        Column::text("warehouse", "Warehouse", |stone: &Stone| {
            stone.warehouse.clone()
        }),
        Column::quantity("on_order", "On Order", |stone: &Stone| Some(stone.on_order)),
        Column::text(ARRIVAL_COLUMN, "Arrival", |stone: &Stone| {
            stone.arrival.clone()
        }),
    ]
}

fn mode_column_keys(mode: &str) -> Vec<&'static str> {
    let kept_keys: &[&str] = match mode {
        CARDS_MODE => &["code"],
        CALENDAR_MODE => &[ARRIVAL_COLUMN],
        _ => &TABLE_SORT_KEYS,
    };
    kept_keys
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
            if mode != CALENDAR_MODE && query.date_range.is_some() {
                query.clear_date_range();
            }
        });
    })
}

fn stone_layout(mode: &str) -> (Vec<Column<Stone>>, Layout<Stone>) {
    match mode {
        CARDS_MODE => (Vec::new(), Layout::Cards(stone_card())),
        CALENDAR_MODE => (
            calendar_columns(),
            Layout::Calendar {
                date_column_key: ARRIVAL_COLUMN,
            },
        ),
        _ => (stone_columns(), Layout::Table),
    }
}

fn mode_switch() -> AnyView {
    view! { <ModeSwitch modes=STONE_MODES transition=keep_only_the_mode_columns() /> }.into_any()
}

fn page_actions() -> Vec<MenuEntry> {
    vec![
        MenuItem::link("New Stone", LucideGlyph::Plus, "#new-stone").into(),
        MenuItem::action("Export", LucideGlyph::Download, || {}).into(),
    ]
}

fn stone_list(view: StoneView, mode: &'static str) -> AnyView {
    let table_query = url_synced_table_query(ROWS_PER_PAGE);
    let load = move |query: TableQuery| load_stone_page(view, query);
    let facet_values = facet_values_loader(
        move |query: TableQuery| load_stone_facets(view, query),
        table_query,
    );
    let (columns, layout) = stone_layout(mode);
    view! {
        <ServerTable
            columns=columns
            layout=layout
            row_href=Arc::new(|stone: &Stone| format!("#{}", stone.code))
            load=load
            query=table_query
            rows_per_page=ROWS_PER_PAGE
            search_placeholder="Search Stones"
            empty_message="No stones match."
            facets=STONE_FACETS.to_vec()
            facet_values=facet_values
            views=STONE_VIEWS
            toolbar=mode_switch
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
        <ListPage title="Tables" actions=page_actions()>
            {move || stone_list(StoneView::from_key(selected_view.get()), selected_mode.get())}
        </ListPage>
    }
    .into_any()
}
