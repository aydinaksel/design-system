use design_system::ResponsiveSource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stone {
    pub code: String,
    pub name: String,
    pub material: String,
    pub color_family: String,
    pub finish: String,
    pub size: String,
    pub warehouse: String,
    pub unit: String,
    pub on_hand: f64,
    pub committed: f64,
    pub on_order: f64,
    pub arrival: String,
    pub image: ResponsiveSource,
}

/// Which stones a list holds, offered as a tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoneView {
    All,
    InStock,
    OnOrder,
}

impl StoneView {
    pub const ALL: [Self; 3] = [Self::All, Self::InStock, Self::OnOrder];

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::InStock => "in-stock",
            Self::OnOrder => "on-order",
        }
    }

    #[must_use]
    pub fn from_key(key: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|view| view.key() == key)
            .unwrap_or(Self::All)
    }
}

pub const FACET_KEYS: [&str; 5] = ["material", "color_family", "finish", "size", "warehouse"];

#[must_use]
pub fn swatch(color_hex: &str) -> ResponsiveSource {
    ResponsiveSource {
        display_url: format!(
            "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 8'>\
             <rect width='8' height='8' fill='%23{color_hex}'/>\
             <path d='M0 6 L8 1 M0 8 L8 4' stroke='%23ffffff' stroke-opacity='0.3' stroke-width='0.25'/>\
             </svg>"
        ),
        source_set: String::new(),
    }
}

#[cfg(feature = "ssr")]
pub mod generated {
    use std::collections::BTreeMap;

    use chrono::{Local, TimeDelta};
    use design_system::table::{FacetValues, FilterValue, Page, TableQuery};

    use super::{swatch, Stone, StoneView, FACET_KEYS};

    const MATERIALS: [(&str, &str); 5] = [
        ("MRBL", "Marble"),
        ("TRVT", "Travertine"),
        ("LMST", "Limestone"),
        ("GRNT", "Granite"),
        ("SLTE", "Slate"),
    ];
    const COLORS: [(&str, &str); 6] = [
        ("Ivory", "e9e2d0"),
        ("Grey", "a7a9ac"),
        ("Black", "3b3b3b"),
        ("Beige", "d8c3a5"),
        ("Green", "8a9a7b"),
        ("Brown", "8b6b4f"),
    ];
    const FINISHES: [&str; 4] = ["Polished", "Honed", "Tumbled", "Brushed"];
    const SIZES: [&str; 5] = ["12x12", "18x18", "24x24", "12x24", "Versailles"];
    const WAREHOUSES: [&str; 3] = ["Paramount, CA", "Houston, TX", "Savannah, GA"];
    const UNITS: [&str; 2] = ["SF", "EA"];
    const ON_HAND: [f64; 7] = [0.0, 12.5, 240.0, 1830.25, 96.0, 0.75, 4410.0];
    const COMMITTED: [f64; 5] = [0.0, 10.0, 120.5, 0.0, 600.0];
    const ON_ORDER: [f64; 4] = [0.0, 0.0, 480.0, 2200.0];
    const ARRIVAL_OFFSET_DAYS: [i64; 9] = [-24, -9, -2, 0, 3, 6, 11, 17, 33];

    fn arrival_date(offset_days: i64) -> String {
        Local::now()
            .date_naive()
            .checked_add_signed(TimeDelta::days(offset_days))
            .map(|date| date.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    }

    fn cycled<Value: Copy + Default>(values: &[Value], position: usize) -> Value {
        values
            .iter()
            .cycle()
            .nth(position)
            .copied()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn stones() -> Vec<Stone> {
        let combinations = MATERIALS.iter().flat_map(|material| {
            COLORS.iter().flat_map(move |color| {
                FINISHES
                    .iter()
                    .map(move |finish| (*material, *color, *finish))
            })
        });
        combinations
            .enumerate()
            .map(
                |(number, ((material_code, material_name), (color_name, color_hex), finish))| {
                    let size = cycled(&SIZES, number);
                    Stone {
                        code: format!("{material_code}-{number:03}"),
                        name: format!("{color_name} {finish} {material_name} {size}"),
                        material: material_name.to_string(),
                        color_family: color_name.to_string(),
                        finish: finish.to_string(),
                        size: size.to_string(),
                        warehouse: cycled(&WAREHOUSES, number).to_string(),
                        unit: cycled(&UNITS, number).to_string(),
                        on_hand: cycled(&ON_HAND, number),
                        committed: cycled(&COMMITTED, number),
                        on_order: cycled(&ON_ORDER, number),
                        arrival: arrival_date(cycled(&ARRIVAL_OFFSET_DAYS, number)),
                        image: swatch(color_hex),
                    }
                },
            )
            .collect()
    }

    fn column_value<'stone>(stone: &'stone Stone, column_key: &str) -> Option<&'stone str> {
        match column_key {
            "material" => Some(&stone.material),
            "color_family" => Some(&stone.color_family),
            "finish" => Some(&stone.finish),
            "size" => Some(&stone.size),
            "warehouse" => Some(&stone.warehouse),
            _ => None,
        }
    }

    fn is_in_view(stone: &Stone, view: StoneView) -> bool {
        match view {
            StoneView::All => true,
            StoneView::InStock => stone.on_hand > 0.0,
            StoneView::OnOrder => stone.on_order > 0.0,
        }
    }

    fn matches_search(stone: &Stone, search: &str) -> bool {
        let search = search.to_lowercase();
        search.is_empty()
            || stone.code.to_lowercase().contains(&search)
            || stone.name.to_lowercase().contains(&search)
    }

    fn matches_filters(stone: &Stone, query: &TableQuery, ignored_column: Option<&str>) -> bool {
        query
            .column_filters
            .iter()
            .filter(|(column_key, _)| Some(column_key.as_str()) != ignored_column)
            .all(|(column_key, selected)| {
                column_value(stone, column_key).is_some_and(|value| selected.contains(value))
            })
    }

    fn matches_date_range(stone: &Stone, query: &TableQuery) -> bool {
        query.date_range.as_ref().is_none_or(|range| {
            range.column_key != "arrival"
                || (range.from.as_str() <= stone.arrival.as_str()
                    && stone.arrival.as_str() <= range.to.as_str())
        })
    }

    fn matching(view: StoneView, query: &TableQuery, ignored_column: Option<&str>) -> Vec<Stone> {
        stones()
            .into_iter()
            .filter(|stone| {
                is_in_view(stone, view)
                    && matches_search(stone, &query.search)
                    && matches_filters(stone, query, ignored_column)
                    && matches_date_range(stone, query)
            })
            .collect()
    }

    fn compare(left: &Stone, right: &Stone, column_key: &str) -> std::cmp::Ordering {
        match column_key {
            "name" => left.name.cmp(&right.name),
            "on_hand" => left.on_hand.total_cmp(&right.on_hand),
            "committed" => left.committed.total_cmp(&right.committed),
            "on_order" => left.on_order.total_cmp(&right.on_order),
            "arrival" => left.arrival.cmp(&right.arrival),
            _ => left.code.cmp(&right.code),
        }
    }

    #[must_use]
    pub fn page(view: StoneView, query: &TableQuery) -> Page<Stone> {
        let mut rows = matching(view, query, None);
        let sort_key = query.sort_column_key.as_deref().unwrap_or("code");
        rows.sort_by(|left, right| {
            let ordering = compare(left, right, sort_key).then_with(|| left.code.cmp(&right.code));
            if query.sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
        let total_rows = u64::try_from(rows.len()).unwrap_or_default();
        let skipped = usize::try_from(query.skipped_rows).unwrap_or_default();
        let taken = usize::try_from(query.rows_per_page).unwrap_or(usize::MAX);
        Page {
            rows: rows.into_iter().skip(skipped).take(taken).collect(),
            total_rows,
        }
    }

    #[must_use]
    pub fn facets(view: StoneView, query: &TableQuery) -> Vec<FacetValues> {
        FACET_KEYS
            .iter()
            .map(|column_key| {
                let mut counts: BTreeMap<String, u64> = BTreeMap::new();
                for stone in matching(view, query, Some(column_key)) {
                    if let Some(value) = column_value(&stone, column_key) {
                        let count = counts.entry(value.to_string()).or_default();
                        *count = count.saturating_add(1);
                    }
                }
                FacetValues {
                    column_key: (*column_key).to_string(),
                    values: counts
                        .into_iter()
                        .map(|(value, row_count)| FilterValue {
                            value,
                            row_count,
                            label: None,
                        })
                        .collect(),
                }
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn every_combination_becomes_one_stone_with_a_unique_code() {
            let all = stones();
            let mut codes: Vec<&str> = all.iter().map(|stone| stone.code.as_str()).collect();
            codes.sort_unstable();
            codes.dedup();

            assert_eq!(all.len(), 120);
            assert_eq!(codes.len(), 120);
        }

        #[test]
        fn a_page_skips_to_the_requested_rows() {
            let mut query = TableQuery::with_rows_per_page(48);
            query.advance_to_next_page();

            let second_page = page(StoneView::All, &query);

            assert_eq!(second_page.total_rows, 120);
            assert_eq!(second_page.rows.len(), 48);
            assert_eq!(second_page.rows[0].code, "MRBL-000");
        }

        #[test]
        fn a_facet_counts_under_every_other_filter_but_its_own() {
            let mut query = TableQuery::with_rows_per_page(48);
            query.set_column_filter("material", ["Marble".to_string()].into());

            let counted = facets(StoneView::All, &query);
            let materials = counted
                .iter()
                .find(|facet| facet.column_key == "material")
                .map(|facet| facet.values.len());
            let finish_total: u64 = counted
                .iter()
                .find(|facet| facet.column_key == "finish")
                .map(|facet| facet.values.iter().map(|value| value.row_count).sum())
                .unwrap_or_default();

            assert_eq!(materials, Some(5));
            assert_eq!(finish_total, 24);
        }
    }
}
