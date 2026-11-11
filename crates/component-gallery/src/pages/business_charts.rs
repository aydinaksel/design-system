use design_system::{
    AxisLabel, ChartCard, ChartColor, ChartSeries, Currency, HeadingLevel, LineChart, SlopeChart,
    SlopeEntry, SmallMultiples, StackMode, StackedBarChart, ValueFormat,
};
use leptos::prelude::*;

const DOLLARS: ValueFormat = ValueFormat::Money(Currency::UsDollar);
const MONTHS: [(&str, &str); 9] = [
    ("Jan", "January 2026"),
    ("Feb", "February 2026"),
    ("Mar", "March 2026"),
    ("Apr", "April 2026"),
    ("May", "May 2026"),
    ("Jun", "June 2026"),
    ("Jul", "July 2026"),
    ("Aug", "August 2026"),
    ("Sep", "September 2026"),
];
const PRODUCT_CATEGORIES: [(&str, ChartColor); 4] = [
    ("Porcelain Tile", ChartColor::Blue),
    ("Natural Stone", ChartColor::Orange),
    ("Quartz Slabs", ChartColor::Aqua),
    ("Setting Materials", ChartColor::Yellow),
];
const REGION_CATEGORY_MIX: [(&str, [f64; 4]); 5] = [
    ("Central", [612_400.0, 301_220.0, 214_860.0, 113_130.0]),
    ("South", [1_398_200.0, 902_640.0, 571_330.0, 232_390.0]),
    ("North", [1_402_550.0, 1_041_370.0, 609_840.0, 242_380.0]),
    ("West", [701_920.0, 562_310.0, 401_150.0, 134_000.0]),
    ("East", [702_140.0, 708_920.0, 548_230.0, 171_180.0]),
];
const AGEING_BUCKETS: [(&str, ChartColor); 4] = [
    ("0 to 30 Days", ChartColor::Blue),
    ("31 to 60 Days", ChartColor::Aqua),
    ("61 to 90 Days", ChartColor::Yellow),
    ("Over 90 Days", ChartColor::Orange),
];
const RECEIVABLES_BY_EMPLOYEE: [(&str, [f64; 4]); 5] = [
    ("Alex Morgan", [412_300.0, 128_440.0, 41_200.0, 18_950.0]),
    ("Jordan Reyes", [301_870.0, 96_310.0, 57_820.0, 61_430.0]),
    ("Unassigned", [96_220.0, 71_980.0, 48_610.0, 88_340.0]),
    ("Sam Patel", [188_040.0, 44_120.0, 12_300.0, 4_870.0]),
    ("Casey Kim", [64_510.0, 18_200.0, 3_150.0, 0.0]),
];
const CUSTOMER_QUARTERS: [(&str, f64, f64); 8] = [
    ("Coastal Tile & Marble", 371_400.0, 412_880.0),
    ("Brightline Interiors", 402_910.0, 356_120.0),
    ("Dunmore Builders", 214_330.0, 298_440.0),
    ("Acme Stone Supply", 256_800.0, 241_975.0),
    ("Harbor View Kitchens", 121_540.0, 188_300.0),
    ("Northfield Renovation", 173_220.0, 152_610.0),
    ("Summit Countertops", 98_760.0, 131_090.0),
    ("Willow Creek Homes", 142_380.0, 97_455.0),
];
const WAREHOUSE_MONTHS: [(&str, [f64; 9]); 6] = [
    (
        "Atlanta",
        [
            412_000.0, 398_500.0, 455_200.0, 471_800.0, 502_300.0, 516_900.0, 488_100.0, 497_600.0,
            531_400.0,
        ],
    ),
    (
        "Charlotte",
        [
            288_400.0, 301_200.0, 297_800.0, 322_500.0, 318_900.0, 341_700.0, 336_200.0, 352_800.0,
            349_100.0,
        ],
    ),
    (
        "Jacksonville",
        [
            201_300.0, 196_800.0, 214_500.0, 231_900.0, 244_200.0, 239_600.0, 228_400.0, 236_700.0,
            251_300.0,
        ],
    ),
    (
        "Nashville",
        [
            176_900.0, 182_400.0, 219_800.0, 241_300.0, 262_700.0, 284_100.0, 301_500.0, 318_200.0,
            356_900.0,
        ],
    ),
    (
        "Raleigh",
        [
            154_200.0, 149_800.0, 158_300.0, 161_900.0, 157_400.0, 163_800.0, 151_200.0, 149_600.0,
            155_900.0,
        ],
    ),
    (
        "Tampa",
        [
            158_900.0, 173_400.0, 252_800.0, 315_000.0, 327_400.0, 310_100.0, 128_100.0, 135_200.0,
            177_200.0,
        ],
    ),
];

fn month_labels() -> Vec<AxisLabel> {
    MONTHS
        .iter()
        .map(|(short, long)| AxisLabel {
            short: (*short).to_owned(),
            long: (*long).to_owned(),
        })
        .collect()
}

fn row_labels<const PARTS: usize>(rows: &[(&str, [f64; PARTS])]) -> Vec<AxisLabel> {
    rows.iter()
        .map(|(name, _)| AxisLabel {
            short: (*name).to_owned(),
            long: (*name).to_owned(),
        })
        .collect()
}

fn part_series<const PARTS: usize>(
    parts: &[(&str, ChartColor); PARTS],
    rows: &[(&str, [f64; PARTS])],
) -> Vec<ChartSeries> {
    parts
        .iter()
        .enumerate()
        .map(|(part_index, (label, color))| ChartSeries {
            label: (*label).to_owned(),
            color: *color,
            values: rows
                .iter()
                .map(|(_, values)| values.get(part_index).copied())
                .collect(),
        })
        .collect()
}

fn region_mix() -> AnyView {
    view! {
        <ChartCard
            title="Porcelain Tile Leads Every Region but the East"
            subtitle="Share of AR invoice totals by product category and region, January to September 2026"
            heading_level=HeadingLevel::Three
        >
            <StackedBarChart
                labels=row_labels(&REGION_CATEGORY_MIX)
                series=part_series(&PRODUCT_CATEGORIES, &REGION_CATEGORY_MIX)
                format=DOLLARS
                mode=StackMode::Share
            />
        </ChartCard>
    }
    .into_any()
}

fn receivables_ageing() -> AnyView {
    view! {
        <ChartCard
            title="Unassigned Accounts Hold the Most Overdue Debt"
            subtitle="Open AR invoice balances by age and sales employee, September 30, 2026"
            heading_level=HeadingLevel::Three
            source="Open AR invoices from the company database"
        >
            <StackedBarChart
                labels=row_labels(&RECEIVABLES_BY_EMPLOYEE)
                series=part_series(&AGEING_BUCKETS, &RECEIVABLES_BY_EMPLOYEE)
                format=DOLLARS
            />
        </ChartCard>
    }
    .into_any()
}

fn customer_movement() -> AnyView {
    let entries: Vec<SlopeEntry> = CUSTOMER_QUARTERS
        .iter()
        .map(|(label, from, to)| SlopeEntry {
            label: (*label).to_owned(),
            from: *from,
            to: *to,
        })
        .collect();
    view! {
        <ChartCard
            title="Dunmore and Harbor View Grew Fastest"
            subtitle="AR invoice totals for the top eight customers, Q2 against Q3 2026"
            heading_level=HeadingLevel::Three
        >
            <SlopeChart entries=entries from_label="Q2 2026" to_label="Q3 2026" format=DOLLARS />
        </ChartCard>
    }
    .into_any()
}

fn warehouse_series(highlighted: Option<&str>) -> Vec<ChartSeries> {
    WAREHOUSE_MONTHS
        .iter()
        .map(|(name, values)| ChartSeries {
            label: (*name).to_owned(),
            color: match highlighted {
                Some(focus) if focus != *name => ChartColor::Muted,
                _ => ChartColor::Blue,
            },
            values: values.iter().copied().map(Some).collect(),
        })
        .collect()
}

fn warehouse_panels() -> AnyView {
    view! {
        <ChartCard
            title="Nashville Has Doubled While Tampa Fell Back"
            subtitle="AR invoice totals by warehouse and month, January to September 2026"
            heading_level=HeadingLevel::Three
            notes=vec!["Every panel shares the same scale.".to_owned()]
        >
            <SmallMultiples labels=month_labels() series=warehouse_series(None) format=DOLLARS />
        </ChartCard>
    }
    .into_any()
}

fn focus_line() -> AnyView {
    let series = warehouse_series(Some("Nashville"));
    view! {
        <ChartCard
            title="Nashville Has Grown Every Month"
            subtitle="AR invoice totals by warehouse and month, January to September 2026"
            heading_level=HeadingLevel::Three
        >
            <LineChart labels=month_labels() series=series format=DOLLARS />
        </ChartCard>
    }
    .into_any()
}

pub(crate) fn business_chart_sections() -> [(&'static str, AnyView); 5] {
    [
        ("Stacked Bar, Share of Whole", region_mix()),
        ("Stacked Bar, Totals", receivables_ageing()),
        ("Slope, Period Against Period", customer_movement()),
        ("Small Multiples", warehouse_panels()),
        ("Line, Focus", focus_line()),
    ]
}
