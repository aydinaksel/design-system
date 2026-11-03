use design_system::{
    button_class, ButtonSize, ButtonTone, Field, Fieldset, Input, InputKind, Popover, PopoverAlign,
    PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle, PopoverTrigger,
};
use leptos::prelude::*;

const ALIGNMENTS: [(PopoverAlign, &str); 3] = [
    (PopoverAlign::Start, "Start"),
    (PopoverAlign::Center, "Center"),
    (PopoverAlign::End, "End"),
];

fn trigger_class() -> String {
    button_class(ButtonTone::Secondary, ButtonSize::Standard)
}

fn basic() -> AnyView {
    view! {
        <Popover>
            <PopoverTrigger class=trigger_class()>"Open Popover"</PopoverTrigger>
            <PopoverContent>
                <PopoverHeader>
                    <PopoverTitle>"Delivery Window"</PopoverTitle>
                    <PopoverDescription>
                        "Trucks arrive between 7:00 and 11:00 on weekdays."
                    </PopoverDescription>
                </PopoverHeader>
            </PopoverContent>
        </Popover>
    }
    .into_any()
}

fn alignments() -> AnyView {
    ALIGNMENTS
        .iter()
        .map(|(align, label)| {
            view! {
                <Popover align=*align>
                    <PopoverTrigger class=trigger_class()>{*label}</PopoverTrigger>
                    <PopoverContent class="w-56">
                        <PopoverDescription>
                            {format!("Aligned to the {} of its trigger.", label.to_lowercase())}
                        </PopoverDescription>
                    </PopoverContent>
                </Popover>
            }
        })
        .collect_view()
        .into_any()
}

fn with_form() -> AnyView {
    let width = RwSignal::new(String::from("18"));
    let height = RwSignal::new(String::from("18"));
    view! {
        <Popover>
            <PopoverTrigger class=trigger_class()>"Dimensions"</PopoverTrigger>
            <PopoverContent>
                <PopoverHeader>
                    <PopoverTitle>"Dimensions"</PopoverTitle>
                    <PopoverDescription>"Set the tile size in inches."</PopoverDescription>
                </PopoverHeader>
                <Fieldset>
                    <Field label="Width">
                        <Input value=width kind=InputKind::Decimal />
                    </Field>
                    <Field label="Height">
                        <Input value=height kind=InputKind::Decimal />
                    </Field>
                </Fieldset>
            </PopoverContent>
        </Popover>
    }
    .into_any()
}

pub(crate) fn popovers() -> AnyView {
    view! {
        <div class="flex flex-wrap items-center gap-3">
            {basic()} <span class="mx-2 h-6 w-px bg-gray-200"></span> {alignments()}
            <span class="mx-2 h-6 w-px bg-gray-200"></span> {with_form()}
        </div>
    }
    .into_any()
}
