use design_system::icons::LucideGlyph;
use design_system::{
    button_class, Button, ButtonGroup, ButtonSize, ButtonTone, DropdownMenu, FilterChip,
    IconButton, Link, LinkButton, MenuEntry, MenuItem, OverflowMenu,
};
use leptos::prelude::*;

const BUTTON_TONES: [(ButtonTone, &str); 6] = [
    (ButtonTone::Primary, "Primary"),
    (ButtonTone::Secondary, "Secondary"),
    (ButtonTone::Danger, "Danger"),
    (ButtonTone::DangerOutline, "Danger Outline"),
    (ButtonTone::Ghost, "Ghost"),
    (ButtonTone::Subtle, "Subtle"),
];

const BUSY_DEMONSTRATION_MILLISECONDS: u64 = 1500;

const BUTTON_SIZES: [(ButtonSize, &str); 3] = [
    (ButtonSize::Standard, "Standard"),
    (ButtonSize::Compact, "Compact"),
    (ButtonSize::Dense, "Dense"),
];

pub(crate) fn buttons() -> AnyView {
    let rows = BUTTON_SIZES
        .iter()
        .map(|(size, size_label)| {
            let buttons = BUTTON_TONES
                .iter()
                .map(|(tone, tone_label)| {
                    view! {
                        <Button tone=*tone size=*size>
                            {*tone_label}
                        </Button>
                    }
                })
                .collect_view();
            view! {
                <div class="flex flex-wrap items-center gap-3">
                    <span class="w-20 text-xs text-gray-500">{*size_label}</span>
                    {buttons}
                    <IconButton
                        glyph=LucideGlyph::ArrowLeft
                        label="Back"
                        tone=ButtonTone::Secondary
                        size=*size
                    />
                </div>
            }
        })
        .collect_view();
    let saving = RwSignal::new(false);
    let save = move |_| {
        saving.set(true);
        set_timeout(
            move || saving.set(false),
            std::time::Duration::from_millis(BUSY_DEMONSTRATION_MILLISECONDS),
        );
    };
    view! {
        <div class="space-y-4">
            {rows} <div class="flex flex-wrap items-center gap-3">
                <span class="w-20 text-xs text-gray-500">"States"</span>
                <Button glyph=Some(LucideGlyph::Plus)>"Add Contact"</Button>
                <Button tone=ButtonTone::Primary busy=saving on_click=save>
                    "Save"
                </Button>
                <Button disabled=true>"Disabled"</Button>
                <LinkButton href="/tables" glyph=Some(LucideGlyph::Table)>
                    "Go to Tables"
                </LinkButton>
                <Link href="/tables">"A Text Link"</Link>
            </div>
        </div>
    }
    .into_any()
}

pub(crate) fn button_group() -> AnyView {
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

pub(crate) fn filter_chips() -> AnyView {
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

pub(crate) fn menus() -> AnyView {
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
                    MenuItem::action(
                        "Copy for Email",
                        LucideGlyph::Copy,
                        choose("Copy for Email"),
                    )
                    .unavailable_while(Signal::stored(Some(
                        "Narrow the list to 50 rows or fewer to copy it.".to_owned(),
                    ))),
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
