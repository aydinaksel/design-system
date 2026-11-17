use std::collections::BTreeSet;

use design_system::{
    notify_success, Checkbox, ChoiceList, ChoiceOption, Combobox, ComboboxGroup, ComboboxOption,
    ComboboxSearch, DatePicker, Field, FieldEditor, FieldGrid, FieldRow, FieldSave, FieldSection,
    Fieldset, FormRow, FormSection, Input, InputKind, Listbox, ListboxOption, MultiSelectList,
    MultiSelectOption, RadioGroup, RadioLayout, RadioOption, SearchInput, SearchSize,
    SectionEditing, Switch, Textarea,
};
use leptos::prelude::*;

use super::overlays::PRIORITIES;

const COMBOBOX_CHOICES: [(&str, &str); 5] = [
    ("C00012", "Acme Stone Supply"),
    ("C00187", "Brightline Interiors"),
    ("C00341", "Harbor Tile Company"),
    ("C00460", "Dunmore Builders"),
    ("C00592", "Evergreen Remodeling"),
];

pub(crate) fn choice_list() -> AnyView {
    let priority = RwSignal::new(String::new());
    let options: Vec<ChoiceOption> = PRIORITIES
        .iter()
        .map(|priority| ChoiceOption::new(*priority, *priority))
        .collect();
    view! {
        <div class="max-w-xs">
            <ChoiceList
                options=options
                current=priority
                label="Priority"
                on_choose=Callback::new(move |chosen: String| priority.set(chosen))
            />
        </div>
    }
    .into_any()
}

pub(crate) fn fieldsets() -> AnyView {
    let container = RwSignal::new(String::from("CSQU3054383"));
    let serial_number = RwSignal::new(String::new());
    let disabled = RwSignal::new(false);
    view! {
        <div class="max-w-md space-y-4 rounded-xl border border-edge p-5">
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
    ("HON", "Honed"),
    ("POL", "Polished"),
    ("TUM", "Tumbled"),
    ("BRU", "Brushed"),
];

const COUNTRIES: [(&str, &str); 6] = [
    ("TR", "Turkey"),
    ("IT", "Italy"),
    ("ES", "Spain"),
    ("PT", "Portugal"),
    ("GR", "Greece"),
    ("MX", "Mexico"),
];

const STONE_COLORS: [&str; 12] = [
    "Beige", "Black", "Blue", "Brown", "Cream", "Green", "Grey", "Gold", "Pink", "Red", "Silver",
    "White",
];
const MULTI_SELECT_PANEL_CLASS: &str = "w-64 rounded-xl border border-edge bg-surface";

fn multi_select_list(label: &'static str, options: Vec<MultiSelectOption>) -> AnyView {
    let selected = RwSignal::new(BTreeSet::new());
    let on_toggle = Callback::new(move |value: String| {
        selected.update(|chosen: &mut BTreeSet<String>| {
            if !chosen.remove(&value) {
                chosen.insert(value);
            }
        });
    });
    view! {
        <div class=MULTI_SELECT_PANEL_CLASS>
            <MultiSelectList
                options=Signal::stored(options)
                selected=selected
                on_toggle=on_toggle
                label=label
            />
        </div>
    }
    .into_any()
}

fn finish_multi_select() -> AnyView {
    let options = FINISHES
        .iter()
        .skip(1)
        .map(|(value, label)| MultiSelectOption {
            value: (*value).to_string(),
            label: (*label).to_string(),
            row_count: None,
        })
        .collect();
    multi_select_list("Finishes", options)
}

fn color_multi_select() -> AnyView {
    let options = STONE_COLORS
        .iter()
        .map(|color| MultiSelectOption {
            value: (*color).to_string(),
            label: (*color).to_string(),
            row_count: None,
        })
        .collect();
    multi_select_list("Colors", options)
}

fn finish_label(code: &str) -> String {
    FINISHES
        .iter()
        .find(|(value, _)| *value == code)
        .map(|(_, label)| (*label).to_owned())
        .unwrap_or_default()
}

pub(crate) fn form_section() -> AnyView {
    let reference = RwSignal::new(String::new());
    let finish = RwSignal::new(String::new());
    let arrival = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let finish_options = Signal::stored(
        FINISHES
            .iter()
            .skip(1)
            .map(|(value, label)| ListboxOption::new(*value, *label))
            .collect::<Vec<_>>(),
    );
    let incomplete = Signal::derive(move || {
        reference.with(|reference| reference.trim().is_empty()) || finish.with(String::is_empty)
    });
    let on_submit = Callback::new(move |()| {
        submitting.set(true);
        set_timeout(
            move || {
                submitting.set(false);
                notify_success(format!(
                    "Stone {} Created",
                    reference.get_untracked().trim()
                ));
            },
            std::time::Duration::from_millis(FAKE_SAVE_MILLISECONDS),
        );
    });
    view! {
        <FormSection
            title="New Stone"
            submit_label="Create Stone"
            on_submit=on_submit
            submitting=submitting
            incomplete=incomplete
            cancel_href="/"
        >
            <FormRow label="Reference">
                <Input value=reference placeholder="e.g. MRBL-0042" />
            </FormRow>
            <FormRow label="Finish">
                <Listbox options=finish_options selected=finish placeholder="Choose a Finish" />
            </FormRow>
            <FormRow label="Expected Arrival" help="Leave empty if not yet booked.">
                <DatePicker value=arrival clearable=true />
            </FormRow>
            <FormRow label="Notes">
                <Textarea value=notes placeholder="Anything the next person should know" />
            </FormRow>
        </FormSection>
    }
    .into_any()
}

fn fake_save(save: FieldSave, fails: bool, commit: impl FnOnce() + 'static) {
    set_timeout(
        move || {
            if fails {
                save.failed("The server rejected the change: the item is locked by another user.");
                return;
            }
            commit();
            save.succeeded();
        },
        std::time::Duration::from_millis(FAKE_SAVE_MILLISECONDS),
    );
}

fn thickness_error(text: &str) -> Option<String> {
    match text.trim().parse::<f64>() {
        Ok(thickness) if thickness > 0.0 => None,
        Ok(_) => Some("Thickness must be greater than zero.".to_owned()),
        Err(_) => Some("Enter a number, such as 3 or 1.5.".to_owned()),
    }
}

fn item_name_error(text: &str) -> Option<String> {
    text.trim()
        .is_empty()
        .then(|| "Enter the item name.".to_owned())
}

pub(crate) fn field_sections() -> AnyView {
    let saved_finish = RwSignal::new(String::from("HON"));
    let saved_thickness = RwSignal::new(String::from("3"));
    let saved_item_name = RwSignal::new(String::from("Ivory Honed Travertine 18x18"));
    let finish = RwSignal::new(saved_finish.get_untracked());
    let thickness = RwSignal::new(saved_thickness.get_untracked());
    let item_name = RwSignal::new(saved_item_name.get_untracked());
    let save_fails = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let finish_options = Signal::stored(
        FINISHES
            .iter()
            .map(|(value, label)| ListboxOption::new(*value, *label))
            .collect::<Vec<_>>(),
    );
    let classification_editing = SectionEditing::new(move |save: FieldSave| {
        let (new_finish, new_thickness) = (finish.get_untracked(), thickness.get_untracked());
        fake_save(save, save_fails.get_untracked(), move || {
            saved_finish.set(new_finish);
            saved_thickness.set(new_thickness);
        });
    });
    let identity_editing = SectionEditing::new(move |save: FieldSave| {
        let new_item_name = item_name.get_untracked();
        fake_save(save, save_fails.get_untracked(), move || {
            saved_item_name.set(new_item_name);
        });
    });
    view! {
        <div class="mb-4 flex items-center gap-3 text-sm text-gray-700">
            <Switch
                checked=save_fails
                on_toggle=Callback::new(move |()| save_fails.update(|fails| *fails = !*fails))
                label="Make the Next Save Fail"
            />
            "Make the Next Save Fail"
            <Switch
                checked=loading
                on_toggle=Callback::new(move |()| loading.update(|loading| *loading = !*loading))
                label="Show Loading"
            />
            "Show Loading"
        </div>
        <FieldGrid>
            <FieldSection title="Classification" editing=classification_editing loading=loading>
                <FieldRow
                    label="Finish"
                    editor=FieldEditor::new(
                        finish,
                        saved_finish,
                        move || view! { <Listbox options=finish_options selected=finish /> },
                    )
                >
                    {move || finish_label(&saved_finish.get())}
                </FieldRow>
                <FieldRow
                    label="Thickness (cm)"
                    help="Measured at the thickest edge."
                    editor=FieldEditor::new(
                            thickness,
                            saved_thickness,
                            move || view! { <Input value=thickness kind=InputKind::Decimal /> },
                        )
                        .error(move || thickness.with(|text| thickness_error(text)))
                >
                    {move || saved_thickness.get()}
                </FieldRow>
                <FieldRow label="Pattern">"Versailles"</FieldRow>
            </FieldSection>
            <FieldSection title="Identity" editing=identity_editing loading=loading>
                <FieldRow label="SKU">"TRVT-024"</FieldRow>
                <FieldRow
                    label="Item Name"
                    editor=FieldEditor::new(
                            item_name,
                            saved_item_name,
                            move || view! { <Input value=item_name /> },
                        )
                        .error(move || item_name.with(|text| item_name_error(text)))
                >
                    {move || saved_item_name.get()}
                </FieldRow>
                <FieldRow label="Barcode">
                    <span class="italic text-amber-600">"Not Set"</span>
                </FieldRow>
            </FieldSection>
            <FieldSection title="Pricing" loading=loading>
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

const RECENT_COUNTRY_CODES: [&str; 2] = ["IT", "PT"];

fn country_option((code, name): &(&str, &str)) -> ComboboxOption {
    ComboboxOption::new(*code, *name)
}

fn grouped_country_list() -> AnyView {
    let country_code = RwSignal::new(String::new());
    let country_name = RwSignal::new(String::new());
    let recent: Vec<ComboboxOption> = COUNTRIES
        .iter()
        .filter(|(code, _)| RECENT_COUNTRY_CODES.contains(code))
        .map(country_option)
        .collect();
    let everything: Vec<ComboboxOption> = COUNTRIES.iter().map(country_option).collect();
    let groups = vec![
        ComboboxGroup::new(recent).with_heading("Recent"),
        ComboboxGroup::new(everything),
    ];
    view! {
        <Combobox
            selected_value=country_code
            selected_label=country_name
            source=groups
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
            <DatePicker value=chosen_date />
            <DatePicker
                value=RwSignal::new(String::new())
                placeholder="No Deadline"
                clearable=true
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

pub(crate) fn form_controls() -> AnyView {
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
                view! { <SearchInput value=search placeholder="Search Stones" /> }.into_any(),
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
            )} {labelled("Multi-Select List", finish_multi_select())}
            {labelled("Multi-Select List, Searchable", color_multi_select())}
            {labelled("Combobox, Searches as You Type", customer_search())}
            {labelled("Combobox, Filters a Fixed List", country_list())}
            {labelled("Combobox, Recent Then Everything", grouped_country_list())}
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
