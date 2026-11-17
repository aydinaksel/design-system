use design_system::icons::LucideGlyph;
use design_system::{
    address_actions, address_value, badge, multiline_value, notify_success, partner_value,
    selected_tab, text_value, total, DatePicker, DetailHeader, DetailPage, FieldEditor, FieldGrid,
    FieldRow, FieldSave, FieldSection, FileKind, FileSection, Input, Listbox, ListboxOption,
    MenuEntry, MenuItem, SectionEditing, TabStrip, Textarea, Tone, TotalsBar, UploadRules,
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use super::activity::activity_feed;
use super::files::{attached_documents, gallery_remover, gallery_uploader, TEN_MEGABYTES};
use super::forms::fake_save;
use super::line_items::{first_lines, line_items_editor, pounds, total_pence, StoneLine};
use super::overlays::PRIORITIES;

const TAB_PARAMETER: &str = "tab";
const DETAIL_TABS: [&str; 4] = ["Overview", "Lines", "Activity", "Files"];
const DELIVERY_CHARGE_PENCE: u64 = 35_000;
const VAT_PERCENT: u64 = 20;
const PERCENT: u64 = 100;
const SHIP_TO: &str = "1200 Harbor Way\nLong Beach, CA 90802";
const SAVE_NEVER_FAILS: bool = false;

#[derive(Clone, Copy)]
struct OrderFields {
    customer_reference: RwSignal<String>,
    delivery_date: RwSignal<String>,
    priority: RwSignal<String>,
    comments: RwSignal<String>,
    tracking_number: RwSignal<String>,
}

impl OrderFields {
    fn sample() -> Self {
        Self {
            customer_reference: RwSignal::new("PO-88412".to_owned()),
            delivery_date: RwSignal::new("2026-11-28".to_owned()),
            priority: RwSignal::new("Hot".to_owned()),
            comments: RwSignal::new("Deliver to the loading dock before noon.".to_owned()),
            tracking_number: RwSignal::new(String::new()),
        }
    }

    fn drafts(self) -> Self {
        Self {
            customer_reference: RwSignal::new(self.customer_reference.get_untracked()),
            delivery_date: RwSignal::new(self.delivery_date.get_untracked()),
            priority: RwSignal::new(self.priority.get_untracked()),
            comments: RwSignal::new(self.comments.get_untracked()),
            tracking_number: RwSignal::new(self.tracking_number.get_untracked()),
        }
    }

    fn copy_from(self, drafts: Self) {
        self.customer_reference
            .set(drafts.customer_reference.get_untracked());
        self.delivery_date.set(drafts.delivery_date.get_untracked());
        self.priority.set(drafts.priority.get_untracked());
        self.comments.set(drafts.comments.get_untracked());
        self.tracking_number
            .set(drafts.tracking_number.get_untracked());
    }
}

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

fn priority_badge(priority: &str) -> AnyView {
    let tone = if priority == "Hot" {
        Tone::Caution
    } else {
        Tone::Neutral
    };
    badge(priority.to_owned(), tone)
}

fn priority_options() -> Vec<ListboxOption> {
    PRIORITIES
        .iter()
        .map(|priority| ListboxOption::new(*priority, *priority))
        .collect()
}

fn order_totals(lines: &[StoneLine]) -> AnyView {
    let subtotal = total_pence(lines);
    let before_vat = subtotal.saturating_add(DELIVERY_CHARGE_PENCE);
    let vat = before_vat.saturating_mul(VAT_PERCENT) / PERCENT;
    let totals = vec![
        total("Subtotal", pounds(subtotal)),
        total("Delivery", pounds(DELIVERY_CHARGE_PENCE)),
        total("VAT", pounds(vat)),
        total("Total", pounds(before_vat.saturating_add(vat))),
    ];
    view! { <TotalsBar totals=totals /> }.into_any()
}

fn customer_section() -> AnyView {
    view! {
        <FieldSection title="Customer">
            <FieldRow label="Customer">
                {partner_value(
                    Some("C00341".to_owned()),
                    Some("Harbor Tile Company".to_owned()),
                    Some("#customer".to_owned()),
                )}
            </FieldRow>
            <FieldRow label="Contact">"Tom Reeve"</FieldRow>
            <FieldRow label="Sales Employee">"Alex Morgan"</FieldRow>
        </FieldSection>
    }
    .into_any()
}

fn order_section(saved: OrderFields, drafts: OrderFields) -> AnyView {
    let editing = SectionEditing::new(move |save: FieldSave| {
        fake_save(save, SAVE_NEVER_FAILS, move || saved.copy_from(drafts));
    });
    let options = Signal::stored(priority_options());
    view! {
        <FieldSection title="Order" editing=editing>
            <FieldRow
                label="Customer PO"
                editor=FieldEditor::new(
                    drafts.customer_reference,
                    saved.customer_reference,
                    move || view! { <Input value=drafts.customer_reference /> },
                )
            >
                {move || saved.customer_reference.get()}
            </FieldRow>
            <FieldRow
                label="Delivery Date"
                editor=FieldEditor::new(
                    drafts.delivery_date,
                    saved.delivery_date,
                    move || view! { <DatePicker value=drafts.delivery_date /> },
                )
            >
                {move || text_value(Some(saved.delivery_date.get()))}
            </FieldRow>
            <FieldRow
                label="Priority"
                editor=FieldEditor::new(
                    drafts.priority,
                    saved.priority,
                    move || view! { <Listbox options=options selected=drafts.priority /> },
                )
            >
                {move || priority_badge(&saved.priority.get())}
            </FieldRow>
            <FieldRow
                label="Comments"
                editor=FieldEditor::new(
                    drafts.comments,
                    saved.comments,
                    move || view! { <Textarea value=drafts.comments /> },
                )
            >
                {move || multiline_value(Some(saved.comments.get()))}
            </FieldRow>
        </FieldSection>
    }
    .into_any()
}

fn shipping_section(saved: OrderFields, drafts: OrderFields) -> AnyView {
    let editing = SectionEditing::new(move |save: FieldSave| {
        fake_save(save, SAVE_NEVER_FAILS, move || saved.copy_from(drafts));
    });
    view! {
        <FieldSection title="Shipping" editing=editing>
            <FieldRow label="Ship To" actions=address_actions(Some(SHIP_TO))>
                {address_value(Some(SHIP_TO.to_owned()))}
            </FieldRow>
            <FieldRow
                label="Tracking Number"
                editor=FieldEditor::new(
                    drafts.tracking_number,
                    saved.tracking_number,
                    move || view! { <Input value=drafts.tracking_number /> },
                )
            >
                {move || {
                    let tracking_number = saved.tracking_number.get();
                    text_value((!tracking_number.is_empty()).then_some(tracking_number))
                }}
            </FieldRow>
        </FieldSection>
    }
    .into_any()
}

fn overview_tab(lines: RwSignal<Vec<StoneLine>>, saved: OrderFields) -> AnyView {
    let drafts = saved.drafts();
    let totals = move || lines.with(|all| order_totals(all));
    view! {
        <div class="space-y-6">
            <FieldGrid>
                {customer_section()} {order_section(saved, drafts)}
                {shipping_section(saved, drafts)}
            </FieldGrid>
            {totals}
        </div>
    }
    .into_any()
}

fn files_tab() -> AnyView {
    view! {
        <div class="max-w-xl">
            <FileSection
                files=Signal::stored(attached_documents())
                label="Order Documents"
                rules=UploadRules::accepting([FileKind::Pdf, FileKind::Image]).up_to(TEN_MEGABYTES)
                upload=gallery_uploader()
                remove=gallery_remover()
            />
        </div>
    }
    .into_any()
}

fn tab_content(tab: usize, lines: RwSignal<Vec<StoneLine>>, saved: OrderFields) -> AnyView {
    match tab {
        1 => line_items_editor(lines),
        2 => activity_feed(),
        3 => files_tab(),
        _ => overview_tab(lines, saved),
    }
}

pub(crate) fn detail_page() -> AnyView {
    let query = use_query_map();
    let active_tab =
        Memo::new(move |_| selected_tab(&DETAIL_TABS, query.read().get(TAB_PARAMETER).as_deref()));
    let lines = RwSignal::new(first_lines());
    let saved = OrderFields::sample();
    view! {
        <main>
            <DetailPage
                header=|| {
                    view! {
                        <DetailHeader
                            title="Sales Order".to_owned()
                            identifier="24817".to_owned()
                            back_href="/tables".to_owned()
                            back_label="Sales Orders".to_owned()
                            actions=Callback::new(|()| header_actions())
                        />
                    }
                }
                tabs=move || {
                    view! { <TabStrip tabs=DETAIL_TABS.to_vec() active=Signal::from(active_tab) /> }
                }
            >
                {move || tab_content(active_tab.get(), lines, saved)}
            </DetailPage>
        </main>
    }
    .into_any()
}
