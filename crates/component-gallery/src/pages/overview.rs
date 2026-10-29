use design_system::icons::{self, LucideGlyph};
use design_system::{
    address_actions, address_value, badge, button_class, button_icon_class, icon_button_class,
    multiline_value, notify_failure, notify_success, partner_value, text_value, total,
    use_dialog_close, BadgeTone, BusyOverlay, ButtonGroup, ButtonSize, ButtonTone, Card, CardGrid,
    Checkbox, ChoiceList, ChoiceOption, Combobox, ComboboxOption, ComboboxSearch, ConfirmDialog,
    CopyButton, CopyIconButton, CopyLinkButton, DatePicker, DetailHeader, Dialog, DialogSize,
    DropdownMenu, Field, FieldGrid, FieldRow, FieldSave, FieldSection, Fieldset, FilterChip,
    GalleryImage, ImageGallery, ImageGrid, Input, InputKind, Listbox, ListboxOption, MenuEntry,
    MenuItem, OverflowMenu, PageHeader, RadioGroup, RadioLayout, RadioOption, ResponsiveImage,
    SearchInput, SearchSize, SectionEditing, Switch, Textarea, Tooltip, TotalsBar,
};
use leptos::prelude::*;

use super::section;
use crate::fake_data::swatch;

const BUTTON_TONES: [(ButtonTone, &str); 5] = [
    (ButtonTone::Primary, "Primary"),
    (ButtonTone::Secondary, "Secondary"),
    (ButtonTone::Danger, "Danger"),
    (ButtonTone::DangerOutline, "Danger Outline"),
    (ButtonTone::Ghost, "Ghost"),
];

const BUTTON_SIZES: [(ButtonSize, &str); 3] = [
    (ButtonSize::Standard, "Standard"),
    (ButtonSize::Compact, "Compact"),
    (ButtonSize::Dense, "Dense"),
];

const BADGE_TONES: [(BadgeTone, &str); 4] = [
    (BadgeTone::Neutral, "Neutral"),
    (BadgeTone::Positive, "Positive"),
    (BadgeTone::Informative, "Informative"),
    (BadgeTone::Caution, "Caution"),
];

const SWATCHES: [(&str, &str, &str); 4] = [
    ("Ivory Honed Travertine", "TRVT-024", "e9e2d0"),
    ("Black Polished Granite", "GRNT-076", "3b3b3b"),
    ("Green Brushed Slate", "SLTE-112", "8a9a7b"),
    ("Beige Tumbled Limestone", "LMST-063", "d8c3a5"),
];

const COMBOBOX_CHOICES: [(&str, &str); 5] = [
    ("C00012", "Acme Stone Supply"),
    ("C00187", "Brightline Interiors"),
    ("C00341", "Harbor Tile Company"),
    ("C00460", "Dunmore Builders"),
    ("C00592", "Evergreen Remodeling"),
];

fn buttons() -> AnyView {
    let rows = BUTTON_SIZES
        .iter()
        .map(|(size, size_label)| {
            let buttons = BUTTON_TONES
                .iter()
                .map(|(tone, tone_label)| {
                    view! {
                        <button type="button" class=button_class(*tone, *size)>
                            {*tone_label}
                        </button>
                    }
                })
                .collect_view();
            view! {
                <div class="flex flex-wrap items-center gap-3">
                    <span class="w-20 text-xs text-gray-500">{*size_label}</span>
                    {buttons}
                    <button
                        type="button"
                        class=icon_button_class(ButtonTone::Secondary, *size)
                        aria-label="Back"
                    >
                        {icons::icon(LucideGlyph::ArrowLeft, button_icon_class(*size))}
                    </button>
                </div>
            }
        })
        .collect_view();
    view! { <div class="space-y-4">{rows}</div> }.into_any()
}

fn badges_and_hints() -> AnyView {
    let badges = BADGE_TONES
        .iter()
        .map(|(tone, label)| badge(*label, *tone))
        .collect_view();
    view! {
        <div class="flex flex-wrap items-center gap-4">
            {badges} <Tooltip label="A tooltip on hover">
                <span class="text-sm text-gray-700 underline decoration-dotted">"Hover Me"</span>
            </Tooltip>
            <span class="inline-flex items-center gap-1 text-sm text-gray-700">
                "TRVT-048" <CopyIconButton value="TRVT-048".to_string() />
            </span> <span class="inline-flex items-center gap-2">
                <CopyButton
                    value="TRVT-048".to_string()
                    glyph=LucideGlyph::Copy
                    label="Copy SKU"
                    copied_label="SKU Copied"
                />
                <CopyLinkButton />
            </span>
        </div>
    }
    .into_any()
}

fn button_group() -> AnyView {
    view! {
        <div class="flex flex-wrap items-center gap-6">
            <ButtonGroup>
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                >
                    "Left"
                </button>
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                >
                    "Middle"
                </button>
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                >
                    "Right"
                </button>
            </ButtonGroup>
        </div>
    }
    .into_any()
}

const STOCK_FILTERS: [&str; 3] = ["Committed", "On the Water", "Ordered"];

fn stock_filter_chips(size: ButtonSize) -> AnyView {
    let chosen = RwSignal::new(vec![STOCK_FILTERS[0]]);
    STOCK_FILTERS
        .into_iter()
        .map(|filter| {
            view! {
                <FilterChip
                    label=filter
                    size=size
                    selected=Signal::derive(move || chosen.with(|chosen| chosen.contains(&filter)))
                    on_toggle=move |()| {
                        chosen
                            .update(|chosen| {
                                if chosen.contains(&filter) {
                                    chosen.retain(|existing| *existing != filter);
                                } else {
                                    chosen.push(filter);
                                }
                            });
                    }
                />
            }
        })
        .collect_view()
        .into_any()
}

fn filter_chips() -> AnyView {
    view! {
        <div class="space-y-3">
            <div role="group" aria-label="Stock Filters" class="flex flex-wrap gap-2">
                {stock_filter_chips(ButtonSize::Compact)}
            </div>
            <div role="group" aria-label="Stock Filters, Dense" class="flex flex-wrap gap-1.5">
                {stock_filter_chips(ButtonSize::Dense)}
            </div>
        </div>
    }
    .into_any()
}

fn menus() -> AnyView {
    let last_action = RwSignal::new(String::from("Nothing chosen yet."));
    let choose = move |label: &'static str| move || last_action.set(format!("Chose {label}."));
    let entries = move || {
        vec![
            MenuItem::action("Edit", LucideGlyph::Pencil, choose("Edit"))
                .shortcut("⌘E")
                .into(),
            MenuItem::action("Duplicate", LucideGlyph::Copy, choose("Duplicate"))
                .shortcut("⌘D")
                .into(),
            MenuEntry::Separator,
            MenuEntry::Section {
                heading: "Export",
                items: vec![
                    MenuItem::download(
                        "Download",
                        LucideGlyph::Download,
                        Signal::derive(|| "data:text/plain,component%20gallery".to_string()),
                    ),
                    MenuItem::action("Email", LucideGlyph::Mail, choose("Email")).disabled(true),
                ],
            },
            MenuEntry::Separator,
            MenuItem::link("Go to Tables", LucideGlyph::Table, "/tables").into(),
            MenuItem::action("Delete", LucideGlyph::X, choose("Delete"))
                .shortcut("⌫")
                .into(),
        ]
    };
    view! {
        <div class="flex flex-wrap items-center gap-6">
            <DropdownMenu label="Actions" entries=entries() />
            <OverflowMenu entries=entries() />
            <span class="text-sm text-gray-500">{move || last_action.get()}</span>
        </div>
    }
    .into_any()
}

fn toasts() -> AnyView {
    view! {
        <div class="flex flex-wrap items-center gap-3">
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=|_| notify_success("Sales Order 10482 Created")
            >
                "Show Success"
            </button>
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=|_| notify_failure("Could Not Reach the Server", None)
            >
                "Show Failure"
            </button>
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=|_| {
                    notify_failure(
                        "Failed Saving Classification",
                        Some("The Service Layer timed out after 30 seconds.".to_owned()),
                    );
                }
            >
                "Show Failure with Detail"
            </button>
        </div>
    }
    .into_any()
}

const PRIORITIES: [&str; 4] = ["Hot", "High", "Normal", "Low"];
const BUSY_DEMONSTRATION_MILLISECONDS: u64 = 2000;

fn gallery_button(label: &'static str, tone: ButtonTone, on_click: impl Fn() + 'static) -> AnyView {
    view! {
        <button
            type="button"
            class=button_class(tone, ButtonSize::Standard)
            on:click=move |_| on_click()
        >
            {label}
        </button>
    }
    .into_any()
}

#[component]
fn RenameForm() -> impl IntoView {
    let name = RwSignal::new(String::from("Ivory Honed Travertine"));
    let close = use_dialog_close();
    let finish = move || {
        if let Some(close) = close {
            close.run(());
        }
    };
    view! {
        <Input value=name attr:data-autofocus="" />
        <div class="mt-5 flex justify-end gap-2">
            <button
                type="button"
                class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                on:click=move |_| finish()
            >
                "Cancel"
            </button>
            <button
                type="button"
                class=button_class(ButtonTone::Primary, ButtonSize::Standard)
                on:click=move |_| {
                    notify_success(format!("Renamed to {}", name.get_untracked()));
                    finish();
                }
            >
                "Save"
            </button>
        </div>
    }
}

fn form_dialog(open: RwSignal<bool>) -> impl IntoView {
    move || {
        open.get().then(|| {
            view! {
                <Dialog
                    title="Rename Item"
                    description="The new name shows on documents created from now on."
                    on_close=Callback::new(move |()| open.set(false))
                >
                    <RenameForm />
                </Dialog>
            }
        })
    }
}

fn priority_dialog(open: RwSignal<bool>, priority: RwSignal<String>) -> impl IntoView {
    let options: Vec<ChoiceOption> = PRIORITIES
        .iter()
        .map(|priority| ChoiceOption::new(*priority, *priority))
        .collect();
    let options = StoredValue::new(options);
    move || {
        open.get().then(|| {
            view! {
                <Dialog
                    title="Change Priority for PR 1042"
                    size=DialogSize::Small
                    on_close=Callback::new(move |()| open.set(false))
                >
                    <ChoiceList
                        options=options.get_value()
                        current=priority
                        label="Priority"
                        autofocus=true
                        on_choose=Callback::new(move |chosen: String| {
                            priority.set(chosen);
                            open.set(false);
                        })
                    />
                </Dialog>
            }
        })
    }
}

fn history_dialog(open: RwSignal<bool>) -> impl IntoView {
    move || {
        open.get().then(|| {
            let rows = (1..=30_u32)
                .map(|day| {
                    view! {
                        <li class="py-2.5 text-sm text-gray-700">
                            {format!("2026-09-{day:02} \u{b7} Quantity changed")}
                        </li>
                    }
                })
                .collect::<Vec<_>>();
            view! {
                <Dialog
                    title="Sales Order 24817 History"
                    size=DialogSize::Large
                    on_close=Callback::new(move |()| open.set(false))
                >
                    <ul class="divide-y divide-gray-100">{rows}</ul>
                </Dialog>
            }
        })
    }
}

fn dialogs() -> AnyView {
    let form_open = RwSignal::new(false);
    let priority_open = RwSignal::new(false);
    let history_open = RwSignal::new(false);
    let confirm_open = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let priority = RwSignal::new(String::from("Normal"));
    let show_busy = move || {
        busy.set(true);
        set_timeout(
            move || busy.set(false),
            std::time::Duration::from_millis(BUSY_DEMONSTRATION_MILLISECONDS),
        );
    };
    view! {
        <div class="flex flex-wrap items-center gap-3">
            {gallery_button("Rename Item", ButtonTone::Secondary, move || form_open.set(true))}
            {gallery_button(
                "Change Priority",
                ButtonTone::Secondary,
                move || priority_open.set(true),
            )}
            {gallery_button("Show History", ButtonTone::Secondary, move || history_open.set(true))}
            {gallery_button("Delete Something", ButtonTone::Danger, move || confirm_open.set(true))}
            {gallery_button("Show Busy Overlay", ButtonTone::Secondary, show_busy)}
            <span class="text-sm text-gray-500">
                {move || format!("Priority: {}", priority.get())}
            </span>
        </div>
        {form_dialog(form_open)}
        {priority_dialog(priority_open, priority)}
        {history_dialog(history_open)}
        <ConfirmDialog
            open=confirm_open
            title="Delete this line?"
            body="It is removed from the document and cannot be restored."
            confirm_label="Delete"
            tone=ButtonTone::Danger
            on_confirm=Callback::new(move |()| notify_success("Line Deleted"))
        />
        <BusyOverlay visible=busy message="Generating Labels\u{2026}" />
    }
    .into_any()
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

fn detail_pages() -> AnyView {
    view! {
        <div class="space-y-6">
            <DetailHeader
                title="Sales Order".to_string()
                identifier="24817".to_string()
                back_href="/tables".to_string()
                back_label="Sales Orders".to_string()
                actions=Callback::new(|()| header_actions())
            />
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
                    <FieldRow label="Priority">{badge("Hot", BadgeTone::Caution)}</FieldRow>
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
            ] />
        </div>
    }
    .into_any()
}

fn fieldsets() -> AnyView {
    let container = RwSignal::new(String::from("CSQU3054383"));
    let serial_number = RwSignal::new(String::new());
    let disabled = RwSignal::new(false);
    view! {
        <div class="max-w-md space-y-4 rounded-lg border border-gray-200 p-5">
            <Fieldset legend="Shipment" disabled=disabled>
                <Field
                    label="Container"
                    description=|| {
                        view! { <span class="text-green-700">"Check digit valid."</span> }
                    }
                >
                    <Input value=container />
                </Field>
                <Field label="Serial Number">
                    <Input value=serial_number placeholder="Not Set" />
                </Field>
            </Fieldset>
            <label class="flex items-center gap-2 text-sm text-gray-700">
                <Checkbox
                    checked=disabled
                    on_toggle=Callback::new(move |()| disabled.update(|value| *value = !*value))
                />
                "Disable the Fieldset"
            </label>
        </div>
    }
    .into_any()
}

const FAKE_SAVE_MILLISECONDS: u64 = 900;

const FINISHES: [(&str, &str); 5] = [
    ("", "Not Set"),
    ("HON", "Honed (HON)"),
    ("POL", "Polished (POL)"),
    ("TUM", "Tumbled (TUM)"),
    ("BRU", "Brushed (BRU)"),
];

const COUNTRIES: [(&str, &str); 6] = [
    ("TR", "Turkey"),
    ("IT", "Italy"),
    ("ES", "Spain"),
    ("PT", "Portugal"),
    ("GR", "Greece"),
    ("MX", "Mexico"),
];

fn finish_label(code: &str) -> String {
    FINISHES
        .iter()
        .find(|(value, _)| *value == code)
        .map(|(_, label)| (*label).to_owned())
        .unwrap_or_default()
}

fn fake_save(save: FieldSave, fails: bool) {
    set_timeout(
        move || {
            if fails {
                save.failed("The server rejected the change: the item is locked by another user.");
            } else {
                save.succeeded();
            }
        },
        std::time::Duration::from_millis(FAKE_SAVE_MILLISECONDS),
    );
}

fn field_sections() -> AnyView {
    let saved_finish = RwSignal::new(String::from("HON"));
    let saved_thickness = RwSignal::new(String::from("3"));
    let finish = RwSignal::new(saved_finish.get_untracked());
    let thickness = RwSignal::new(saved_thickness.get_untracked());
    let save_fails = RwSignal::new(false);
    let finish_options = Signal::stored(
        FINISHES
            .iter()
            .map(|(value, label)| ListboxOption::new(*value, *label))
            .collect::<Vec<_>>(),
    );
    let reset = move || {
        finish.set(saved_finish.get_untracked());
        thickness.set(saved_thickness.get_untracked());
    };
    let editing = SectionEditing {
        can_edit: true,
        is_editing: RwSignal::new(false),
        on_edit: Callback::new(move |()| reset()),
        on_cancel: Callback::new(move |()| reset()),
        on_save: Callback::new(move |save: FieldSave| {
            let fails = save_fails.get_untracked();
            if !fails {
                saved_finish.set(finish.get_untracked());
                saved_thickness.set(thickness.get_untracked());
            }
            fake_save(save, fails);
        }),
    };
    view! {
        <div class="mb-4 flex items-center gap-3 text-sm text-gray-700">
            <Switch
                checked=save_fails
                on_toggle=Callback::new(move |()| save_fails.update(|fails| *fails = !*fails))
                label="Make the Next Save Fail"
            />
            "Make the Next Save Fail"
        </div>
        <FieldGrid>
            <FieldSection title="Classification" editing=editing>
                <FieldRow
                    label="Finish"
                    changed=Signal::derive(move || finish.get() != saved_finish.get())
                    on_revert=move |()| finish.set(saved_finish.get_untracked())
                    editor=move || view! { <Listbox options=finish_options selected=finish /> }
                >
                    {move || finish_label(&finish.get())}
                </FieldRow>
                <FieldRow
                    label="Thickness (cm)"
                    description="Measured at the thickest edge."
                    changed=Signal::derive(move || thickness.get() != saved_thickness.get())
                    on_revert=move |()| thickness.set(saved_thickness.get_untracked())
                    editor=move || view! { <Input value=thickness kind=InputKind::Decimal /> }
                >
                    {move || thickness.get()}
                </FieldRow>
                <FieldRow label="Pattern">"Versailles"</FieldRow>
            </FieldSection>
            <FieldSection title="Identity">
                <FieldRow label="SKU">"TRVT-024"</FieldRow>
                <FieldRow label="Item Name">"Ivory Honed Travertine 18x18"</FieldRow>
                <FieldRow label="Barcode">
                    <span class="italic text-amber-600">"Not set"</span>
                </FieldRow>
            </FieldSection>
            <FieldSection title="Pricing">
                <FieldRow label="Base Price">"$4.25"</FieldRow>
                <FieldRow label="Last Purchase Price">"$2.10 USD"</FieldRow>
            </FieldSection>
            <FieldSection title="Status">
                <FieldRow label="Active">"Yes"</FieldRow>
                <FieldRow label="Frozen">"No"</FieldRow>
            </FieldSection>
        </FieldGrid>
    }
    .into_any()
}

fn labelled(label: &'static str, control: AnyView) -> AnyView {
    view! {
        <div>
            <span class="mb-1 block text-xs font-medium text-gray-600">{label}</span>
            {control}
        </div>
    }
    .into_any()
}

fn matching_customers(search: &str) -> Vec<ComboboxOption> {
    let search = search.to_lowercase();
    COMBOBOX_CHOICES
        .iter()
        .filter(|(code, name)| {
            code.to_lowercase().contains(&search) || name.to_lowercase().contains(&search)
        })
        .map(|(code, name)| ComboboxOption::new(*code, *name))
        .collect()
}

fn customer_search() -> AnyView {
    let customer_code = RwSignal::new(String::new());
    let customer_name = RwSignal::new(String::new());
    let search = ComboboxSearch::new(|search: String| async move {
        Ok::<Vec<ComboboxOption>, String>(matching_customers(&search))
    })
    .with_suggestions("Recently Used", || async {
        Ok::<Vec<ComboboxOption>, String>(
            COMBOBOX_CHOICES
                .iter()
                .take(3)
                .map(|(code, name)| ComboboxOption::new(*code, *name))
                .collect(),
        )
    });
    view! {
        <Combobox
            selected_value=customer_code
            selected_label=customer_name
            source=search
            placeholder="Search Customers"
            empty_message="No customers match."
            clear_label="Clear Customer"
        />
    }
    .into_any()
}

fn country_list() -> AnyView {
    let country_code = RwSignal::new(String::new());
    let country_name = RwSignal::new(String::new());
    let countries: Vec<ComboboxOption> = COUNTRIES
        .iter()
        .map(|(code, name)| ComboboxOption::new(*code, *name))
        .collect();
    view! {
        <Combobox
            selected_value=country_code
            selected_label=country_name
            source=countries
            placeholder="Country of Origin"
            empty_message="No country matches."
            clear_label="Clear Country"
        />
    }
    .into_any()
}

fn date_picker() -> AnyView {
    let chosen_date = RwSignal::new(String::from("2026-10-15"));
    view! {
        <div class="space-y-2">
            <DatePicker
                date=chosen_date.get_untracked()
                on_select=Callback::new(move |date| chosen_date.set(date))
            />
            <p class="text-xs text-gray-500">{move || format!("Chose {}", chosen_date.get())}</p>
        </div>
    }
    .into_any()
}

fn finish_radios() -> AnyView {
    let finish = RwSignal::new(String::from("HON"));
    let options = Signal::stored(
        FINISHES
            .iter()
            .skip(1)
            .map(|(value, label)| RadioOption::new(*value, *label))
            .collect::<Vec<_>>(),
    );
    view! { <RadioGroup options=options selected=finish label="Finish" /> }.into_any()
}

fn delivery_radios() -> AnyView {
    let delivery = RwSignal::new(String::from("pickup"));
    let options = Signal::stored(vec![
        RadioOption::new("pickup", "Customer Pickup")
            .with_description("Collected from the warehouse dock."),
        RadioOption::new("truck", "Our Truck").with_description("Delivered within 40 miles."),
        RadioOption::new("freight", "Freight Carrier")
            .with_description("Quoted per pallet by the carrier."),
        RadioOption::new("courier", "Courier")
            .with_description("Samples only, not offered for slabs.")
            .disabled(),
    ]);
    view! {
        <RadioGroup
            options=options
            selected=delivery
            layout=RadioLayout::Cards
            label="Delivery Method"
        />
    }
    .into_any()
}

fn form_controls() -> AnyView {
    let name = RwSignal::new(String::from("Ivory Honed Travertine"));
    let phone = RwSignal::new(String::from("+1305555"));
    let notes = RwSignal::new(String::new());
    let finish = RwSignal::new(String::new());
    let active = RwSignal::new(true);
    let checked = RwSignal::new(true);
    let search = RwSignal::new(String::new());
    let compact_search = RwSignal::new(String::new());
    let finish_options = Signal::stored(
        FINISHES
            .iter()
            .skip(1)
            .map(|(value, label)| ListboxOption::new(*value, *label))
            .collect::<Vec<_>>(),
    );
    view! {
        <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {labelled("Input", view! { <Input value=name /> }.into_any())}
            {labelled(
                "Search Input",
                view! { <SearchInput value=search placeholder="Search by Code or Name" /> }
                    .into_any(),
            )}
            {labelled(
                "Search Input, Compact",
                view! { <SearchInput value=compact_search size=SearchSize::Compact /> }.into_any(),
            )}
            {labelled(
                "Input, Invalid",
                view! {
                    <Input value=phone kind=InputKind::Telephone invalid=true />
                    <p class="mt-1 text-xs text-red-600">"Too short for a phone number."</p>
                }
                    .into_any(),
            )}
            {labelled(
                "Textarea",
                view! {
                    <Textarea value=notes placeholder="Anything the next person should know" />
                }
                    .into_any(),
            )}
            {labelled(
                "Listbox",
                view! {
                    <Listbox options=finish_options selected=finish placeholder="Choose a Finish" />
                }
                    .into_any(),
            )} {labelled("Combobox, Searches as You Type", customer_search())}
            {labelled("Combobox, Filters a Fixed List", country_list())}
            {labelled("Date Picker", date_picker())} {labelled("Radio Group", finish_radios())}
            <div class="sm:col-span-2 lg:col-span-3">
                {labelled("Radio Group, Cards", delivery_radios())}
            </div>
            {labelled(
                "Switch",
                view! {
                    <div class="flex items-center gap-3 text-sm text-gray-700">
                        <Switch
                            checked=active
                            on_toggle=Callback::new(move |()| active.update(|on| *on = !*on))
                            label="Active"
                        />
                        {move || if active.get() { "Active" } else { "Inactive" }}
                    </div>
                }
                    .into_any(),
            )}
            {labelled(
                "Checkbox",
                view! {
                    <label class="flex items-center gap-2 text-sm text-gray-700">
                        <Checkbox
                            checked=checked
                            on_toggle=Callback::new(move |()| {
                                checked.update(|value| *value = !*value);
                            })
                        />
                        {move || if checked.get() { "Checked" } else { "Unchecked" }}
                    </label>
                }
                    .into_any(),
            )}
        </div>
    }
    .into_any()
}

fn images() -> AnyView {
    let gallery_images: Vec<GalleryImage> = SWATCHES
        .iter()
        .map(|(name, _, color_hex)| {
            let source = swatch(color_hex);
            GalleryImage {
                label: (*name).to_string(),
                thumbnail_url: source.display_url.clone(),
                display_url: source.display_url,
                source_set: source.source_set,
            }
        })
        .collect();
    let cards = SWATCHES
        .iter()
        .map(|(name, code, color_hex)| {
            let image = view! { <ResponsiveImage source=Some(swatch(color_hex)) alternative_text=*name sizes="18rem" /> }
            .into_any();
            view! {
                <Card href=format!("#{code}") image=image>
                    <p class="text-sm font-medium text-gray-900">{*name}</p>
                    <p class="mt-0.5 font-mono text-xs text-gray-500">{*code}</p>
                </Card>
            }
        })
        .collect_view();
    let missing_image =
        view! { <ResponsiveImage source=None alternative_text="No photograph" sizes="18rem" /> }
            .into_any();
    view! {
        <ImageGrid images=gallery_images.clone() />
        <div class="mt-8 grid gap-8 lg:grid-cols-2">
            <ImageGallery images=Signal::stored(gallery_images) selected_image=RwSignal::new(0) />
            <CardGrid>
                {cards} <Card href="#missing" image=missing_image>
                    <p class="text-sm font-medium text-gray-900">"Without a Photograph"</p>
                </Card>
            </CardGrid>
        </div>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn OverviewPage() -> impl IntoView {
    view! {
        <main class="mx-auto max-w-screen-2xl space-y-10 px-6 py-8">
            <PageHeader title="Overview">
                <span class="text-sm text-gray-500">"Every component, with fake data"</span>
            </PageHeader>
            {section("Buttons", buttons())}
            {section("Badges, Tooltips and Copying", badges_and_hints())}
            {section("Button Groups", button_group())}
            {section("Filter Chips", filter_chips())}
            {section("Menus", menus())}
            {section("Dialogs", dialogs())}
            {section("Toasts", toasts())}
            {section("Detail Pages", detail_pages())}
            {section("Fieldsets", fieldsets())}
            {section("Form Controls", form_controls())}
            {section("Field Sections", field_sections())}
            {section("Images and Cards", images())}
        </main>
    }
    .into_any()
}
