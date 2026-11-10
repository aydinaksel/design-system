use design_system::formatting::format_with_thousands;
use design_system::{
    AxisLabel, BarEntry, ChartCard, ChartColor, ChartMetric, ChartSeries, ColumnChart, Currency,
    HeadingLevel, HorizontalBarChart, LineChart, PageHeader, ReferenceLine, StatChange, StatGrid,
    StatTile, ValueFormat,
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use super::business_charts::business_chart_sections;

const YEAR: u32 = 2026;
const QUARTER_MONTHS: [(&str, u32); 3] = [("Jul", 31), ("Aug", 31), ("Sep", 30)];
const FIRST_DAY_WEEKDAY_FROM_MONDAY: usize = 2;
const DAYS_PER_WEEK: usize = 7;
const FIRST_WEEKEND_DAY_FROM_MONDAY: usize = 5;
const SALES_ORDER_SEED: u64 = 0x5EED_0001;
const ACCOUNTS_RECEIVABLE_INVOICE_SEED: u64 = 0x5EED_0002;
const NOISE_RESOLUTION: f64 = 16_777_216.0;
const TOP_CUSTOMERS: [(&str, f64); 10] = [
    ("Coastal Tile & Marble Distributors", 412_880.0),
    ("Brightline Interiors", 356_120.0),
    ("Dunmore Builders", 298_440.0),
    ("Acme Stone Supply", 241_975.0),
    ("Harbor View Kitchens", 188_300.0),
    ("Northfield Renovation Group", 152_610.0),
    ("Summit Countertops", 131_090.0),
    ("Willow Creek Homes", 97_455.0),
    ("Granite State Fabricators", 74_200.0),
    ("Lakeside Design Studio", 52_830.0),
];
const OTHER_CUSTOMERS_TOTAL: f64 = 611_240.0;
const MONTHLY_INVOICED: [(&str, &str, f64); 9] = [
    ("Jan", "January 2026", 1_391_850.0),
    ("Feb", "February 2026", 1_402_110.0),
    ("Mar", "March 2026", 1_598_470.0),
    ("Apr", "April 2026", 1_744_320.0),
    ("May", "May 2026", 1_812_900.0),
    ("Jun", "June 2026", 1_856_210.0),
    ("Jul", "July 2026", 1_633_480.0),
    ("Aug", "August 2026", 1_690_050.0),
    ("Sep", "September 2026", 1_801_760.0),
];
const REGION_TOTALS: [(&str, &str, f64, f64); 5] = [
    ("North", "North region", 3_412_880.0, 3_296_140.0),
    ("South", "South region", 2_987_310.0, 3_104_560.0),
    ("East", "East region", 2_256_090.0, 2_130_470.0),
    ("West", "West region", 1_874_520.0, 1_799_380.0),
    ("Central", "Central region", 1_203_840.0, 1_241_610.0),
];
const GUIDANCE_HREF: &str =
    "https://analysisfunction.civilservice.gov.uk/policy-store/data-visualisation-charts/";
const PRINCIPLES: [&str; 9] = [
    "The title states the message, and the subtitle says what the data is, where and when.",
    "Gaps between single bars are narrower than a bar, and gaps between clusters are wider than one.",
    "More than one time series is a line chart, never clustered columns.",
    "Lines are labelled at their ends instead of through a legend that relies on colour.",
    "A focus chart colours the series that matters and greys the rest.",
    "Small multiples share one scale, so panels compare honestly.",
    "Bars start at zero, ranked bars are sorted by value, and a stack holds at most four parts.",
    "Every series chart carries its data as a table for screen readers.",
    "Gridlines are few and light, and every label reads horizontally.",
];
const MONTHLY_INVOICE_TARGET: f64 = 1_600_000.0;
const DOLLARS: ValueFormat = ValueFormat::Money(Currency::UsDollar);
const SELECTED_EMPLOYEE_PARAMETER: &str = "sales-employee";
const SALES_EMPLOYEES: [(&str, f64); 6] = [
    ("Alex Morgan", 2_104_550.0),
    ("Unassigned", 1_986_300.0),
    ("Jordan Reyes", 1_412_870.0),
    ("Sam Patel", 764_090.0),
    ("Casey Kim", 188_420.0),
    ("Riley Chen", 31_650.0),
];

fn quarter_labels() -> Vec<AxisLabel> {
    QUARTER_MONTHS
        .iter()
        .flat_map(|(month, day_count)| {
            (1..=*day_count).map(move |day| AxisLabel {
                short: format!("{month} {day}"),
                long: format!("{month} {day}, {YEAR}"),
            })
        })
        .collect()
}

fn noise(seed: u64, index: usize) -> f64 {
    let mut mixed = seed
        .wrapping_add(u64::try_from(index).unwrap_or(u64::MAX))
        .wrapping_mul(0x9E37_79B9_7F4A_7C15);
    mixed = (mixed ^ mixed.wrapping_shr(30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ mixed.wrapping_shr(27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    let top_bits = u32::try_from(mixed.wrapping_shr(40)).unwrap_or(0);
    f64::from(top_bits) / NOISE_RESOLUTION
}

fn is_weekend(index: usize) -> bool {
    index
        .wrapping_add(FIRST_DAY_WEEKDAY_FROM_MONDAY)
        .checked_rem(DAYS_PER_WEEK)
        .is_some_and(|weekday| weekday >= FIRST_WEEKEND_DAY_FROM_MONDAY)
}

fn daily_totals(seed: u64, weekday_floor: f64, weekday_spread: f64, day_count: usize) -> Vec<f64> {
    (0..day_count)
        .map(|index| {
            let jitter = noise(seed, index);
            if is_weekend(index) {
                1_500.0 + jitter * 4_000.0
            } else {
                weekday_floor + jitter * weekday_spread
            }
        })
        .collect()
}

fn whole_dollars(amount: f64) -> String {
    format!("${}", format_with_thousands(&format!("{amount:.0}")))
}

fn quarter_series(day_count: usize) -> [ChartSeries; 2] {
    [
        ChartSeries {
            label: "Sales Orders".to_owned(),
            color: ChartColor::Blue,
            values: daily_totals(SALES_ORDER_SEED, 24_000.0, 36_000.0, day_count),
        },
        ChartSeries {
            label: "AR Invoices".to_owned(),
            color: ChartColor::Orange,
            values: daily_totals(
                ACCOUNTS_RECEIVABLE_INVOICE_SEED,
                20_000.0,
                30_000.0,
                day_count,
            ),
        },
    ]
}

fn chart_section(title: &'static str, content: AnyView) -> AnyView {
    view! {
        <section class="space-y-3">
            <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-500">{title}</h2>
            {content}
        </section>
    }
    .into_any()
}

fn interactive_line(labels: Vec<AxisLabel>, series: [ChartSeries; 2]) -> AnyView {
    let metrics: Vec<ChartMetric> = series
        .iter()
        .map(|line| ChartMetric {
            label: line.label.clone(),
            value: whole_dollars(line.values.iter().sum()),
            color: line.color,
        })
        .collect();
    let selected = RwSignal::new(0_usize);
    let shown = Signal::derive(move || {
        series
            .get(selected.get())
            .cloned()
            .into_iter()
            .collect::<Vec<_>>()
    });
    view! {
        <ChartCard
            title="Sales Drop Every Weekend"
            subtitle="Daily totals, Jul 1 to Sep 30, 2026"
            heading_level=HeadingLevel::Three
            metrics=metrics
            selected_metric=selected
        >
            <LineChart labels=labels series=shown format=DOLLARS />
        </ChartCard>
    }
    .into_any()
}

fn comparison_line(labels: Vec<AxisLabel>, series: [ChartSeries; 2]) -> AnyView {
    view! {
        <ChartCard
            title="Invoices Follow the Weekly Rhythm of Orders"
            subtitle="Daily sales order and AR invoice totals, Jul 1 to Sep 30, 2026"
            heading_level=HeadingLevel::Three
        >
            <LineChart labels=labels series=series.to_vec() format=DOLLARS />
        </ChartCard>
    }
    .into_any()
}

fn empty_line() -> AnyView {
    let no_labels: Vec<AxisLabel> = Vec::new();
    let no_series: Vec<ChartSeries> = Vec::new();
    view! {
        <ChartCard
            title="Sales by Day"
            subtitle="Daily totals, Oct 1, 2026"
            heading_level=HeadingLevel::Three
        >
            <LineChart labels=no_labels series=no_series format=DOLLARS />
        </ChartCard>
    }
    .into_any()
}

fn top_customers() -> AnyView {
    let entries: Vec<BarEntry> = TOP_CUSTOMERS
        .iter()
        .map(|(name, invoiced)| BarEntry {
            label: (*name).to_owned(),
            value: *invoiced,
            href: None,
        })
        .collect();
    let remainder = Some(BarEntry {
        label: "Other (34 customers)".to_owned(),
        value: OTHER_CUSTOMERS_TOTAL,
        href: None,
    });
    view! {
        <ChartCard
            title="Three Customers Make Up Two Fifths of Invoicing"
            subtitle="Top ten customers by AR invoice total, Jul 1 to Sep 30, 2026"
            heading_level=HeadingLevel::Three
            source="AR invoices from the company database"
            notes=vec!["Credit notes are netted against the invoices they correct.".to_owned()]
        >
            <HorizontalBarChart entries=entries format=DOLLARS remainder=remainder />
        </ChartCard>
    }
    .into_any()
}

fn stat_tiles() -> AnyView {
    view! {
        <StatGrid>
            <StatTile
                label="Invoiced, September"
                value="$1,866,410"
                change=Some(StatChange {
                    percent: 8.4,
                    comparison: "Sep 2025".to_owned(),
                    higher_is_better: true,
                })
                footnote="AR invoices posted"
            />
            <StatTile
                label="Ordered, September"
                value="$1,801,760"
                change=Some(StatChange {
                    percent: -3.1,
                    comparison: "Sep 2025".to_owned(),
                    higher_is_better: true,
                })
                footnote="Sales orders placed"
            />
            <StatTile
                label="Open Sales Orders"
                value="$642,180"
                footnote="Ordered, not yet invoiced"
            />
            <StatTile
                label="Average Order, September"
                value="$3,412"
                change=Some(StatChange {
                    percent: 0.0,
                    comparison: "Sep 2025".to_owned(),
                    higher_is_better: true,
                })
                footnote="Per sales order"
            />
        </StatGrid>
    }
    .into_any()
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

fn monthly_columns() -> AnyView {
    let labels: Vec<AxisLabel> = MONTHLY_INVOICED
        .iter()
        .map(|(short, long, _)| AxisLabel {
            short: (*short).to_owned(),
            long: (*long).to_owned(),
        })
        .collect();
    let series = vec![ChartSeries {
        label: "AR Invoices".to_owned(),
        color: ChartColor::Blue,
        values: MONTHLY_INVOICED
            .iter()
            .map(|(_, _, invoiced)| *invoiced)
            .collect(),
    }];
    view! {
        <ChartCard
            title="Invoicing Has Met Target Since April"
            subtitle="AR invoice totals by month, January to September 2026"
            heading_level=HeadingLevel::Three
        >
            <ColumnChart
                labels=labels
                series=series
                format=DOLLARS
                reference=ReferenceLine {
                    label: "Target".to_owned(),
                    value: MONTHLY_INVOICE_TARGET,
                }
            />
        </ChartCard>
    }
    .into_any()
}

fn regional_columns() -> AnyView {
    let labels: Vec<AxisLabel> = REGION_TOTALS
        .iter()
        .map(|(short, long, _, _)| AxisLabel {
            short: (*short).to_owned(),
            long: (*long).to_owned(),
        })
        .collect();
    let series = vec![
        ChartSeries {
            label: "Sales Orders".to_owned(),
            color: ChartColor::Blue,
            values: REGION_TOTALS
                .iter()
                .map(|(_, _, ordered, _)| *ordered)
                .collect(),
        },
        ChartSeries {
            label: "AR Invoices".to_owned(),
            color: ChartColor::Orange,
            values: REGION_TOTALS
                .iter()
                .map(|(_, _, _, invoiced)| *invoiced)
                .collect(),
        },
    ];
    view! {
        <ChartCard
            title="The North Leads Both Orders and Invoices"
            subtitle="Sales order and AR invoice totals by region, January to September 2026"
            heading_level=HeadingLevel::Three
        >
            <ColumnChart labels=labels series=series format=DOLLARS />
        </ChartCard>
    }
    .into_any()
}

fn principles() -> AnyView {
    view! {
        <div class="space-y-3 text-sm text-gray-700">
            <ul class="list-disc space-y-1 pl-5">
                {PRINCIPLES.iter().map(|principle| view! { <li>{*principle}</li> }).collect_view()}
            </ul>
            <p class="text-gray-500">
                "Drawn from the Government Analysis Function's "
                <a
                    href=GUIDANCE_HREF
                    class="text-gray-900 underline underline-offset-2 hover:text-focus"
                >
                    "data visualisation guidance for charts"
                </a> "."
            </p>
        </div>
    }
    .into_any()
}

fn sales_employee_ranking() -> AnyView {
    let query = use_query_map();
    let selected_employee = move || {
        let selected_slug = query.with(|parameters| parameters.get(SELECTED_EMPLOYEE_PARAMETER));
        SALES_EMPLOYEES
            .iter()
            .find(|(name, _)| selected_slug.as_deref() == Some(slug(name).as_str()))
            .map_or_else(
                || "Click a row to select a sales employee.".to_owned(),
                |(name, _)| format!("Selected: {name}"),
            )
    };
    let entries: Vec<BarEntry> = SALES_EMPLOYEES
        .iter()
        .map(|(name, invoiced)| BarEntry {
            label: (*name).to_owned(),
            value: *invoiced,
            href: Some(format!("?{SELECTED_EMPLOYEE_PARAMETER}={}", slug(name))),
        })
        .collect();
    view! {
        <ChartCard
            title="Sales Employees"
            subtitle="AR invoice totals by sales employee, Jul 1 to Sep 30, 2026"
            heading_level=HeadingLevel::Three
        >
            <HorizontalBarChart entries=entries format=DOLLARS />
            <p class="mt-4 text-sm text-gray-500">{selected_employee}</p>
        </ChartCard>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ChartsPage() -> impl IntoView {
    let labels = quarter_labels();
    let series = quarter_series(labels.len());
    view! {
        <main data-canvas class="mx-auto max-w-screen-xl space-y-10 px-6 py-8">
            <PageHeader title="Charts" />
            {chart_section("Principles", principles())}
            {chart_section("Stat Tiles", stat_tiles())}
            {chart_section("Line, Interactive", interactive_line(labels.clone(), series.clone()))}
            {chart_section("Line, Two Series", comparison_line(labels, series))}
            {chart_section("Line, Empty Range", empty_line())}
            {chart_section("Columns, One Series", monthly_columns())}
            {chart_section("Columns, Clustered", regional_columns())}
            {chart_section("Horizontal Bar, Top Customers", top_customers())}
            {chart_section("Horizontal Bar, Clickable Rows", sales_employee_ranking())}
            {business_chart_sections()
                .into_iter()
                .map(|(title, content)| chart_section(title, content))
                .collect_view()}
        </main>
    }
    .into_any()
}
