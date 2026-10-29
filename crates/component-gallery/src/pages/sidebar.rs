use design_system::icons::LucideGlyph;
use design_system::{MenuEntry, MenuItem, NavigationItem, NavigationSection, Sidebar, SidebarUser};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

fn account_entries() -> Vec<MenuEntry> {
    vec![
        MenuEntry::Section {
            heading: "Tools",
            items: vec![
                MenuItem::external_link(
                    "Documentation",
                    LucideGlyph::FileText,
                    "https://leptos.dev",
                ),
                MenuItem::external_link(
                    "Status",
                    LucideGlyph::ExternalLink,
                    "https://www.githubstatus.com",
                ),
            ],
        },
        MenuEntry::Separator,
        MenuItem::link("Back to the Gallery", LucideGlyph::ArrowLeft, "/").into(),
    ]
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn SidebarPage() -> impl IntoView {
    let pathname = use_location().pathname;

    view! {
        <div class="flex h-dvh overflow-hidden">
            <Sidebar
                cookie_prefix="gallery_sidebar"
                footer=|| view! { <SidebarUser name="Ada Lovelace" entries=account_entries() /> }
            >
                <NavigationItem
                    href="/sidebar/dashboard"
                    label="Dashboard"
                    glyph=LucideGlyph::LayoutDashboard
                />
                <NavigationSection key="people" title="People">
                    <NavigationItem
                        href="/sidebar/contacts"
                        label="Contacts"
                        glyph=LucideGlyph::Users
                    />
                    <NavigationItem href="/sidebar/inbox" label="Inbox" glyph=LucideGlyph::Inbox />
                </NavigationSection>
                <NavigationSection key="planning" title="Planning">
                    <NavigationItem
                        href="/sidebar/calendar"
                        label="Calendar"
                        glyph=LucideGlyph::Calendar
                    />
                    <NavigationItem
                        href="/sidebar/tasks"
                        label="Tasks"
                        glyph=LucideGlyph::ListTodo
                    />
                    <NavigationItem
                        href="/sidebar/documents"
                        label="Documents"
                        glyph=LucideGlyph::FileText
                    />
                </NavigationSection>
                <NavigationSection key="system" title="System">
                    <NavigationItem
                        href="/sidebar/settings"
                        label="Settings"
                        glyph=LucideGlyph::Settings
                    />
                </NavigationSection>
            </Sidebar>
            <main class="flex-1 overflow-y-auto px-8 pb-8 pt-20 sm:pt-8">
                <h1 class="text-2xl font-bold tracking-tight">"Sidebar"</h1>
                <p class="mt-1 text-sm text-gray-500">
                    "Collapse it with the button by the brand, open and close sections by their headers, and reload: both survive in cookies."
                </p>
                <p class="mt-6 text-sm text-gray-700">
                    "Current Page: "
                    <code class="rounded bg-gray-100 px-1.5 py-0.5">{move || pathname.get()}</code>
                </p>
                <A href="/" attr:class="mt-6 inline-block text-sm text-blue-600 hover:underline">
                    "Back to the Gallery"
                </A>
            </main>
        </div>
    }
    .into_any()
}
