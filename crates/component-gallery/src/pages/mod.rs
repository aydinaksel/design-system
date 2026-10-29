pub mod calendar;
pub mod overview;
pub mod sidebar;
pub mod tables;

use leptos::prelude::*;

pub(crate) const ROWS_PER_PAGE: u32 = 48;
#[cfg(feature = "ssr")]
pub(crate) const SIMULATED_LATENCY_MILLISECONDS: u64 = 300;

pub(crate) const STONE_FACETS: [design_system::table::ServerFacet; 5] = [
    design_system::table::ServerFacet::new("material", "Material"),
    design_system::table::ServerFacet::new("color_family", "Color Family"),
    design_system::table::ServerFacet::new("finish", "Finish"),
    design_system::table::ServerFacet::new("size", "Size"),
    design_system::table::ServerFacet::new("warehouse", "Warehouse"),
];

pub(crate) fn section(title: &'static str, content: AnyView) -> AnyView {
    view! {
        <section class="space-y-3">
            <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-500">{title}</h2>
            <div class="rounded-lg border border-gray-200 p-6">{content}</div>
        </section>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_rail_offers_every_facet_the_fake_data_counts() {
        let offered: Vec<&str> = super::STONE_FACETS
            .iter()
            .map(|facet| facet.column_key)
            .collect();

        assert_eq!(offered, crate::fake_data::FACET_KEYS);
    }
}
