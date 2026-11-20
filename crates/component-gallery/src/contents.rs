use design_system::icons::LucideGlyph;
use leptos::prelude::*;

use crate::pages::articles::ArticlesPage;
use crate::pages::charts::ChartsPage;
use crate::pages::client_side_table::ClientSideTablePage;
use crate::pages::files::FilesPage;
use crate::pages::quick_filters::QuickFiltersPage;
use crate::pages::tables::TablesPage;
use crate::pages::{
    actions, activity, cell_editing, content, detail_page, forms, line_items, overlays,
    period_filter, popovers, row_selection,
};

#[derive(Clone, Copy)]
pub enum Presentation {
    Showcase(fn() -> AnyView),
    Page(fn() -> AnyView),
    Standalone,
}

#[derive(Clone, Copy)]
pub struct Entry {
    pub href: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub glyph: LucideGlyph,
    pub presentation: Presentation,
}

impl Entry {
    fn slug(&self) -> Option<&'static str> {
        self.href.strip_prefix('/')
    }
}

pub struct Group {
    pub key: &'static str,
    pub title: &'static str,
    pub entries: &'static [Entry],
}

fn tables_page() -> AnyView {
    view! { <TablesPage /> }.into_any()
}

fn client_side_table_page() -> AnyView {
    view! { <ClientSideTablePage /> }.into_any()
}

fn quick_filters_page() -> AnyView {
    view! { <QuickFiltersPage /> }.into_any()
}

fn files_page() -> AnyView {
    view! { <FilesPage /> }.into_any()
}

fn charts_page() -> AnyView {
    view! { <ChartsPage /> }.into_any()
}

fn articles_page() -> AnyView {
    view! { <ArticlesPage /> }.into_any()
}

const ACTIONS: [Entry; 4] = [
    Entry {
        href: "/buttons",
        title: "Buttons",
        summary: "Every tone and size an action can take.",
        glyph: LucideGlyph::MousePointerClick,
        presentation: Presentation::Showcase(actions::buttons),
    },
    Entry {
        href: "/button-groups",
        title: "Button Groups",
        summary: "Joined buttons for switching between a few views.",
        glyph: LucideGlyph::Columns3,
        presentation: Presentation::Showcase(actions::button_group),
    },
    Entry {
        href: "/filter-chips",
        title: "Filter Chips",
        summary: "Toggles that narrow a list, with counts.",
        glyph: LucideGlyph::ListFilter,
        presentation: Presentation::Showcase(actions::filter_chips),
    },
    Entry {
        href: "/menus",
        title: "Menus",
        summary: "Dropdown, overflow and row action menus.",
        glyph: LucideGlyph::EllipsisVertical,
        presentation: Presentation::Showcase(actions::menus),
    },
];

const OVERLAYS: [Entry; 3] = [
    Entry {
        href: "/dialogs",
        title: "Dialogs",
        summary: "Forms, choices, confirmations and busy states.",
        glyph: LucideGlyph::AppWindow,
        presentation: Presentation::Showcase(overlays::dialogs),
    },
    Entry {
        href: "/popovers",
        title: "Popovers",
        summary: "Panels anchored to the control that opened them.",
        glyph: LucideGlyph::PanelTop,
        presentation: Presentation::Showcase(popovers::popovers),
    },
    Entry {
        href: "/toasts",
        title: "Toasts",
        summary: "Short confirmations and failures after an action.",
        glyph: LucideGlyph::Bell,
        presentation: Presentation::Showcase(overlays::toasts),
    },
];

const FORMS: [Entry; 7] = [
    Entry {
        href: "/form-controls",
        title: "Form Controls",
        summary: "Inputs, pickers, comboboxes and radios.",
        glyph: LucideGlyph::TextCursorInput,
        presentation: Presentation::Showcase(forms::form_controls),
    },
    Entry {
        href: "/choice-list",
        title: "Choice List",
        summary: "A short list where one choice applies at once.",
        glyph: LucideGlyph::ListChecks,
        presentation: Presentation::Showcase(forms::choice_list),
    },
    Entry {
        href: "/fieldsets",
        title: "Fieldsets",
        summary: "Labelled fields stacked for dialogs and panels.",
        glyph: LucideGlyph::Rows3,
        presentation: Presentation::Showcase(forms::fieldsets),
    },
    Entry {
        href: "/field-sections",
        title: "Field Sections",
        summary: "Record fields that edit in place.",
        glyph: LucideGlyph::SquarePen,
        presentation: Presentation::Showcase(forms::field_sections),
    },
    Entry {
        href: "/form-sections",
        title: "Form Sections",
        summary: "A card of fields saved together.",
        glyph: LucideGlyph::ClipboardList,
        presentation: Presentation::Showcase(forms::form_section),
    },
    Entry {
        href: "/period-filter",
        title: "Period Filter",
        summary: "A named period or custom dates for a report, kept in the URL.",
        glyph: LucideGlyph::CalendarRange,
        presentation: Presentation::Showcase(period_filter::period_filter),
    },
    Entry {
        href: "/cell-editing",
        title: "Cell Editing",
        summary: "Table cells that edit in place.",
        glyph: LucideGlyph::Pencil,
        presentation: Presentation::Showcase(cell_editing::cell_editing),
    },
];

const DATA: [Entry; 8] = [
    Entry {
        href: "/tables",
        title: "Tables",
        summary: "Server tables with facets, cards and a calendar.",
        glyph: LucideGlyph::Table,
        presentation: Presentation::Page(tables_page),
    },
    Entry {
        href: "/client-side-table",
        title: "Client-Side Table",
        summary: "A table that sorts and filters in the browser.",
        glyph: LucideGlyph::Sheet,
        presentation: Presentation::Page(client_side_table_page),
    },
    Entry {
        href: "/quick-filters",
        title: "Quick Filters",
        summary: "A server table filtered by buttons beside its search.",
        glyph: LucideGlyph::ListFilter,
        presentation: Presentation::Page(quick_filters_page),
    },
    Entry {
        href: "/row-selection",
        title: "Row Selection",
        summary: "Tables whose rows are picked with a checkbox or a click.",
        glyph: LucideGlyph::SquareCheck,
        presentation: Presentation::Showcase(row_selection::row_selection),
    },
    Entry {
        href: "/line-items",
        title: "Line Items",
        summary: "Order lines edited in place, with totals.",
        glyph: LucideGlyph::ListOrdered,
        presentation: Presentation::Showcase(line_items::line_items),
    },
    Entry {
        href: "/charts",
        title: "Charts",
        summary: "Charts and stat tiles for business reporting.",
        glyph: LucideGlyph::TrendingUp,
        presentation: Presentation::Page(charts_page),
    },
    Entry {
        href: "/files",
        title: "Files",
        summary: "Uploading, arranging and viewing documents.",
        glyph: LucideGlyph::Folder,
        presentation: Presentation::Page(files_page),
    },
    Entry {
        href: "/images",
        title: "Images and Cards",
        summary: "Photo grids, a gallery and linked cards.",
        glyph: LucideGlyph::Image,
        presentation: Presentation::Showcase(content::images),
    },
];

const CONTENT: [Entry; 6] = [
    Entry {
        href: "/activity-feed",
        title: "Activity Feed",
        summary: "Emails, calls, meetings and notes by day.",
        glyph: LucideGlyph::Clock,
        presentation: Presentation::Showcase(activity::activity_feed),
    },
    Entry {
        href: "/articles",
        title: "Articles",
        summary: "Help docs and how-tos written in Markdown.",
        glyph: LucideGlyph::FileText,
        presentation: Presentation::Page(articles_page),
    },
    Entry {
        href: "/badges",
        title: "Badges, Tooltips and Copying",
        summary: "Status badges, hover hints and copy buttons.",
        glyph: LucideGlyph::Tag,
        presentation: Presentation::Showcase(content::badges_and_hints),
    },
    Entry {
        href: "/alerts",
        title: "Alerts",
        summary: "Messages that sit in the page in each tone.",
        glyph: LucideGlyph::TriangleAlert,
        presentation: Presentation::Showcase(content::alerts),
    },
    Entry {
        href: "/meters",
        title: "Meters",
        summary: "Progress towards a limit.",
        glyph: LucideGlyph::Hourglass,
        presentation: Presentation::Showcase(content::meters),
    },
    Entry {
        href: "/qr-codes",
        title: "QR Codes",
        summary: "Codes for links, with a button to show one.",
        glyph: LucideGlyph::QrCode,
        presentation: Presentation::Showcase(content::qr_codes),
    },
];

const PAGES: [Entry; 2] = [
    Entry {
        href: "/detail-page",
        title: "Detail Page",
        summary: "A record's header, tabs, fields and totals.",
        glyph: LucideGlyph::PanelsTopLeft,
        presentation: Presentation::Page(detail_page::detail_page),
    },
    Entry {
        href: "/sidebar/dashboard",
        title: "Sidebar",
        summary: "Collapsible navigation that remembers its state.",
        glyph: LucideGlyph::PanelLeft,
        presentation: Presentation::Standalone,
    },
];

pub const GROUPS: [Group; 6] = [
    Group {
        key: "actions",
        title: "Actions",
        entries: &ACTIONS,
    },
    Group {
        key: "overlays",
        title: "Overlays",
        entries: &OVERLAYS,
    },
    Group {
        key: "forms",
        title: "Forms",
        entries: &FORMS,
    },
    Group {
        key: "data",
        title: "Data",
        entries: &DATA,
    },
    Group {
        key: "content",
        title: "Content",
        entries: &CONTENT,
    },
    Group {
        key: "pages",
        title: "Pages",
        entries: &PAGES,
    },
];

#[must_use]
pub fn entry_for(slug: &str) -> Option<Entry> {
    GROUPS
        .iter()
        .flat_map(|group| group.entries.iter())
        .find(|entry| entry.slug() == Some(slug))
        .copied()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_entry_has_its_own_address() {
        let hrefs: Vec<&str> = GROUPS
            .iter()
            .flat_map(|group| group.entries.iter().map(|entry| entry.href))
            .collect();
        let unique: HashSet<&str> = hrefs.iter().copied().collect();
        assert_eq!(unique.len(), hrefs.len());
    }

    #[test]
    fn finds_an_entry_by_its_slug() {
        assert_eq!(
            entry_for("line-items").map(|entry| entry.title),
            Some("Line Items")
        );
        assert!(entry_for("nowhere").is_none());
    }
}
