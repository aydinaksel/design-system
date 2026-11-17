use design_system::icons::LucideGlyph;
use design_system::{
    address_actions, address_value, badge, button_class, multiline_value, notify_success,
    partner_value, selected_tab, text_value, total, ButtonSize, ButtonTone, DetailHeader,
    DetailPage, EmptyState, FieldGrid, FieldRow, FieldSection, MenuEntry, MenuItem, PageSection,
    TabStrip, Tone, TotalsBar,
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

fn header_actions() -> Vec<MenuEntry> {
    vec![
        MenuEntry::Section {
            heading: "View",
            items: vec![
                MenuItem::action("History", LucideGlyph::ScrollText, || {
                    notify_success("History Opens Here");
                }),
                MenuItem::link("Relationship Map", LucideGlyph::Link, "#relationships"),
            ],
        },
        MenuEntry::Section {
            heading: "Sales Order",
            items: vec![MenuItem::action(
                "Generate Labels",
                LucideGlyph::FileText,
                || {
                    notify_success("Labels Generated");
                },
            )],
        },
        MenuEntry::Section {
            heading: "Copy To",
            items: vec![
                MenuItem::link("Delivery", LucideGlyph::ArrowRight, "#delivery"),
                MenuItem::link("AR Invoice", LucideGlyph::ArrowRight, "#invoice"),
            ],
        },
    ]
}

const DETAIL_TABS: [&str; 2] = ["Overview", "Lines"];

fn detail_overview() -> AnyView {
    view! {
        <div class="space-y-6">
            <FieldGrid>
                <FieldSection title="Overview">
                    <FieldRow label="Customer">
                        {partner_value(
                            Some("C00341".to_string()),
                            Some("Harbor Tile Company".to_string()),
                            Some("#customer".to_string()),
                        )}
                    </FieldRow>
                    <FieldRow label="Customer PO">
                        {text_value(Some("PO-88412".to_string()))}
                    </FieldRow>
                    <FieldRow label="Posting Date">
                        {text_value(Some("2026-10-06".to_string()))}
                    </FieldRow>
                    <FieldRow label="Priority">{badge("Hot", Tone::Caution)}</FieldRow>
                    <FieldRow label="Comments">
                        {multiline_value(
                            Some("Deliver to the loading dock before noon.".to_string()),
                        )}
                    </FieldRow>
                </FieldSection>
                <FieldSection title="Shipping">
                    <FieldRow
                        label="Ship To"
                        actions=address_actions(Some("1200 Harbor Way\nLong Beach, CA 90802"))
                    >
                        {address_value(Some("1200 Harbor Way\nLong Beach, CA 90802".to_string()))}
                    </FieldRow>
                    <FieldRow label="Tracking Number">{text_value(None)}</FieldRow>
                </FieldSection>
            </FieldGrid>
            <TotalsBar totals=vec![
                total("Subtotal", "$12,410.00"),
                total("Freight", "$350.00"),
                total("Total", "$12,760.00"),
                total("Margin", "-$1,240.00").tone(Tone::Critical),
            ] />
        </div>
    }
    .into_any()
}

pub(crate) fn detail_pages() -> AnyView {
    let query = use_query_map();
    let active_tab =
        Signal::derive(move || selected_tab(&DETAIL_TABS, query.read().get("tab").as_deref()));
    view! {
        <div class="-m-6">
            <DetailPage
                header=|| {
                    view! {
                        <DetailHeader
                            title="Sales Order".to_string()
                            identifier="24817".to_string()
                            back_href="/tables".to_string()
                            back_label="Sales Orders".to_string()
                            actions=Callback::new(|()| header_actions())
                        />
                    }
                }
                tabs=move || view! { <TabStrip tabs=DETAIL_TABS.to_vec() active=active_tab /> }
            >
                {move || { if active_tab.get() == 0 { detail_overview() } else { detail_lines() } }}
            </DetailPage>
        </div>
    }
    .into_any()
}

fn detail_lines() -> AnyView {
    view! {
        <PageSection
            title="Lines"
            actions=view! {
                <button type="button" class=button_class(ButtonTone::Secondary, ButtonSize::Dense)>
                    "Add Line"
                </button>
            }
                .into_any()
        >
            <EmptyState title="No lines yet" />
        </PageSection>
    }
    .into_any()
}
