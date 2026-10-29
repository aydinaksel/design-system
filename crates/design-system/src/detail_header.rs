use leptos::prelude::*;
use leptos_router::components::A;

use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::clipboard::{copy_current_address_with_toast, copy_with_toast};
use crate::icons;
use crate::menu::{DropdownMenu, MenuEntry, MenuItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentStatus {
    Open,
    Closed,
    Canceled,
}

impl DocumentStatus {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Closed => "Closed",
            Self::Canceled => "Canceled",
        }
    }

    #[must_use]
    pub fn text_class(self) -> &'static str {
        match self {
            Self::Open => "text-blue-700",
            Self::Closed => "text-gray-500",
            Self::Canceled => "text-red-700",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IdentifierKind {
    #[default]
    Number,
    Sku,
    Code,
}

impl IdentifierKind {
    const fn copy_label(self) -> &'static str {
        match self {
            Self::Number => "Copy Number",
            Self::Sku => "Copy SKU",
            Self::Code => "Copy Code",
        }
    }

    const fn copied_message(self) -> &'static str {
        match self {
            Self::Number => "Number Copied",
            Self::Sku => "SKU Copied",
            Self::Code => "Code Copied",
        }
    }

    const fn text_class(self) -> &'static str {
        match self {
            Self::Number => "",
            Self::Sku | Self::Code => "font-semibold text-gray-400",
        }
    }
}

fn copy_entries(identifier: Signal<String>, kind: IdentifierKind) -> Vec<MenuEntry> {
    vec![
        MenuItem::action(kind.copy_label(), icons::LucideGlyph::Copy, move || {
            copy_with_toast(&identifier.get_untracked(), kind.copied_message());
        })
        .into(),
        MenuItem::action(
            "Copy Link",
            icons::LucideGlyph::Link,
            copy_current_address_with_toast,
        )
        .into(),
    ]
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn DetailHeader(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] identifier: Signal<String>,
    #[prop(optional)] identifier_kind: IdentifierKind,
    #[prop(optional, into)] back_href: MaybeProp<String>,
    #[prop(optional, into)] back_label: MaybeProp<String>,
    #[prop(optional, into)] actions: Option<Callback<(), Vec<MenuEntry>>>,
) -> impl IntoView {
    let back_link = move || {
        back_href.get().map(|href| {
            let label = back_label.get().unwrap_or_else(|| "Back".to_string());
            view! {
                <A
                    href=href
                    attr:class=icon_button_class(ButtonTone::Secondary, ButtonSize::Standard)
                    attr:title=label.clone()
                    attr:aria-label=label
                >
                    {icons::icon(icons::LucideGlyph::ArrowLeft, "h-4 w-4 opacity-70")}
                </A>
            }
        })
    };
    let actions_menu = move || {
        let mut entries = copy_entries(identifier, identifier_kind);
        let extra = actions.map(|actions| actions.run(())).unwrap_or_default();
        if !extra.is_empty() {
            entries.push(MenuEntry::Separator);
            entries.extend(extra);
        }
        view! { <DropdownMenu label="Actions" entries=entries /> }
    };

    view! {
        <header class="mb-4 flex flex-wrap items-center justify-between gap-3">
            <div class="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1">
                <h1 class="flex min-w-0 flex-wrap items-baseline gap-x-2 text-xl font-bold tracking-tight text-gray-900">
                    <span>{move || title.get()}</span>
                    <span class=identifier_kind.text_class()>{move || identifier.get()}</span>
                </h1>
            </div>
            <div class="flex items-center gap-2">{back_link} {actions_menu}</div>
        </header>
    }
    .into_any()
}
