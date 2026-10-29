use codee::string::FromToStringCodec;
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};

use crate::brand::brand;
use crate::icons::{self, LucideGlyph};
use crate::menu::{menu, MenuEntry};

const COOKIE_PATH: &str = "/";
const COOKIE_MAX_AGE_MILLISECONDS: i64 = 365 * 24 * 60 * 60 * 1000;
const CLOSED_SECTION_SEPARATOR: char = ',';
const EXPANDED_WIDTH: &str = "13rem";
const COLLAPSED_WIDTH: &str = "3rem";
const UNNAMED_USER_LABEL: &str = "Account";

const ICON_CLASS: &str = "block h-4 w-4 shrink-0";
const LABEL_CLASS: &str =
    "transition-opacity duration-200 group-data-[collapsed=true]/sidebar:opacity-0";
const BRAND_CLASS: &str = "absolute left-4 whitespace-nowrap text-sm font-semibold tracking-tight text-gray-900 transition-opacity duration-200 group-data-[collapsed=true]/sidebar:pointer-events-none group-data-[collapsed=true]/sidebar:opacity-0";
const TOGGLE_CLASS: &str = "ml-auto flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-gray-200 text-gray-500 hover:text-gray-900";
const ACTIVE_ITEM_CLASS: &str = "flex items-center gap-3 overflow-hidden whitespace-nowrap rounded-md p-2 text-sm font-medium bg-gray-100 text-gray-900 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-600";
const INACTIVE_ITEM_CLASS: &str = "flex items-center gap-3 overflow-hidden whitespace-nowrap rounded-md p-2 text-sm text-gray-600 hover:bg-gray-100 hover:text-gray-900 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-600";
const SECTION_TOGGLE_CLASS: &str = "flex h-7 w-full cursor-pointer items-center justify-between whitespace-nowrap rounded-md px-2 text-xs font-medium text-gray-500 transition-opacity duration-200 hover:text-gray-900 group-data-[collapsed=true]/sidebar:pointer-events-none group-data-[collapsed=true]/sidebar:opacity-0 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-600";
const SECTION_DIVIDER_CLASS: &str = "pointer-events-none absolute inset-x-1 top-1/2 h-px bg-gray-200 opacity-0 transition-opacity duration-200 group-data-[collapsed=true]/sidebar:opacity-100";
const USER_TRIGGER_CLASS: &str = "flex w-full cursor-pointer items-center gap-2 overflow-hidden whitespace-nowrap rounded-md p-1 text-left text-sm text-gray-700 hover:bg-gray-100 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-600";
const AVATAR_CLASS: &str = "flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-gray-900 text-[0.625rem] font-medium text-white";

#[derive(Clone, Copy)]
struct SidebarState {
    collapsed: Signal<bool>,
    closed_sections: Signal<Option<String>>,
    set_closed_sections: WriteSignal<Option<String>>,
    drawer_open: Option<RwSignal<bool>>,
}

impl SidebarState {
    fn section_closed(self, key: &str) -> bool {
        self.closed_sections.get().is_some_and(|closed_keys| {
            closed_keys
                .split(CLOSED_SECTION_SEPARATOR)
                .any(|closed_key| closed_key == key)
        })
    }

    fn toggle_section(self, key: &str) {
        let mut closed_keys: Vec<String> = self
            .closed_sections
            .get_untracked()
            .unwrap_or_default()
            .split(CLOSED_SECTION_SEPARATOR)
            .filter(|closed_key| !closed_key.is_empty())
            .map(str::to_owned)
            .collect();
        let was_closed = closed_keys.iter().any(|closed_key| closed_key == key);
        closed_keys.retain(|closed_key| closed_key != key);
        if !was_closed {
            closed_keys.push(key.to_owned());
        }
        self.set_closed_sections.set(Some(
            closed_keys.join(&CLOSED_SECTION_SEPARATOR.to_string()),
        ));
    }

    fn close_drawer(self) {
        if let Some(drawer_open) = self.drawer_open {
            drawer_open.set(false);
        }
    }
}

#[must_use]
pub fn is_active(pathname: &str, href: &str) -> bool {
    if href == "/" {
        return pathname == "/";
    }
    pathname == href
        || pathname
            .strip_prefix(href)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

#[component]
pub fn Sidebar(
    cookie_prefix: &'static str,
    children: ChildrenFn,
    #[prop(optional, into)] footer: Option<ViewFn>,
) -> impl IntoView {
    let (collapsed_cookie, set_collapsed_cookie) = use_cookie_with_options::<bool, FromToStringCodec>(
        &format!("{cookie_prefix}_collapsed"),
        UseCookieOptions::default()
            .path(COOKIE_PATH)
            .same_site(SameSite::Lax)
            .max_age(COOKIE_MAX_AGE_MILLISECONDS),
    );
    let (closed_sections, set_closed_sections) = use_cookie_with_options::<String, FromToStringCodec>(
        &format!("{cookie_prefix}_closed_sections"),
        UseCookieOptions::default()
            .path(COOKIE_PATH)
            .same_site(SameSite::Lax)
            .max_age(COOKIE_MAX_AGE_MILLISECONDS),
    );
    let collapsed = Signal::derive(move || collapsed_cookie.get().unwrap_or(false));
    let drawer_open = RwSignal::new(false);
    let desktop_state = SidebarState {
        collapsed,
        closed_sections,
        set_closed_sections,
        drawer_open: None,
    };
    let drawer_state = SidebarState {
        collapsed: Signal::derive(|| false),
        closed_sections,
        set_closed_sections,
        drawer_open: Some(drawer_open),
    };
    let brand = brand();
    let desktop_children = children.clone();
    let desktop_footer = footer.clone();

    view! {
        <MobileNavigation drawer_open state=drawer_state children footer />

        <aside
            class="group/sidebar hidden h-full shrink-0 flex-col overflow-hidden border-r border-gray-200 bg-white transition-[width] duration-200 sm:flex"
            data-collapsed=move || collapsed.get().to_string()
            style=move || {
                format!("width:{}", if collapsed.get() { COLLAPSED_WIDTH } else { EXPANDED_WIDTH })
            }
        >
            <Provider value=desktop_state>
                <div class="relative flex h-12 shrink-0 items-center border-b border-gray-200 px-2.5">
                    <A
                        href=brand.home_href
                        attr:class=BRAND_CLASS
                        attr:tabindex=move || collapsed.get().then_some("-1")
                    >
                        {brand.name}
                    </A>
                    <button
                        type="button"
                        aria-label="Toggle Navigation"
                        class=TOGGLE_CLASS
                        on:click=move |_| {
                            set_collapsed_cookie.set(Some(!collapsed.get_untracked()));
                        }
                    >
                        {move || {
                            let glyph = if collapsed.get() {
                                LucideGlyph::PanelLeftOpen
                            } else {
                                LucideGlyph::PanelLeftClose
                            };
                            icons::icon(glyph, ICON_CLASS)
                        }}
                    </button>
                </div>
                <nav class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto overflow-x-hidden p-2">
                    {desktop_children()}
                </nav>
                {desktop_footer
                    .map(|footer| {
                        view! {
                            <div class="shrink-0 border-t border-gray-200 p-2">{footer.run()}</div>
                        }
                    })}
            </Provider>
        </aside>
    }
    .into_any()
}

#[component]
fn MobileNavigation(
    drawer_open: RwSignal<bool>,
    state: SidebarState,
    children: ChildrenFn,
    footer: Option<ViewFn>,
) -> impl IntoView {
    let brand = brand();
    let drawer_brand_href = brand.home_href;

    view! {
        <div class="fixed inset-x-0 top-0 z-30 flex h-12 items-center gap-3 border-b border-gray-200 bg-white px-3 sm:hidden">
            <button
                type="button"
                aria-label="Open Navigation"
                class="flex cursor-pointer items-center justify-center p-1 text-gray-700"
                on:click=move |_| drawer_open.set(true)
            >
                {icons::icon(LucideGlyph::Menu, ICON_CLASS)}
            </button>
            <A href=brand.home_href attr:class="text-sm font-semibold tracking-tight text-gray-900">
                {brand.name}
            </A>
        </div>

        {move || {
            if !drawer_open.get() {
                return ().into_any();
            }
            let children = children.clone();
            let footer = footer.clone();
            view! {
                <div
                    class="fixed inset-0 z-40 bg-black/40 sm:hidden"
                    on:click=move |_| drawer_open.set(false)
                ></div>
                <div class="fixed inset-y-0 left-0 z-50 flex w-64 flex-col overflow-hidden bg-white shadow-lg sm:hidden">
                    <Provider value=state>
                        <div class="flex h-12 shrink-0 items-center border-b border-gray-200 px-3">
                            <A
                                href=drawer_brand_href
                                attr:class="text-sm font-semibold tracking-tight text-gray-900"
                                on:click=move |_| drawer_open.set(false)
                            >
                                {brand.name}
                            </A>
                            <button
                                type="button"
                                aria-label="Close Navigation"
                                class="ml-auto flex cursor-pointer items-center justify-center p-1 text-gray-500 hover:text-gray-700"
                                on:click=move |_| drawer_open.set(false)
                            >
                                {icons::icon(LucideGlyph::X, ICON_CLASS)}
                            </button>
                        </div>
                        <nav class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
                            {children()}
                        </nav>
                        {footer
                            .map(|footer| {
                                view! {
                                    <div class="shrink-0 border-t border-gray-200 p-2">
                                        {footer.run()}
                                    </div>
                                }
                            })}
                    </Provider>
                </div>
            }
                .into_any()
        }}
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn NavigationSection(
    key: &'static str,
    title: &'static str,
    children: Children,
) -> impl IntoView {
    let Some(state) = use_context::<SidebarState>() else {
        return children().into_any();
    };
    let open = move || state.collapsed.get() || !state.section_closed(key);

    view! {
        <div role="group" aria-label=title class="flex flex-col">
            <div class="relative">
                <button
                    type="button"
                    class=SECTION_TOGGLE_CLASS
                    aria-expanded=move || open().to_string()
                    tabindex=move || state.collapsed.get().then_some("-1")
                    on:click=move |_| state.toggle_section(key)
                >
                    <span>{title}</span>
                    <span class=move || {
                        if open() {
                            "transition-transform duration-200"
                        } else {
                            "-rotate-90 transition-transform duration-200"
                        }
                    }>{icons::icon(LucideGlyph::ChevronDown, "block h-3.5 w-3.5")}</span>
                </button>
                <div class=SECTION_DIVIDER_CLASS aria-hidden="true"></div>
            </div>
            <div
                class="grid transition-[grid-template-rows] duration-200"
                style=move || {
                    format!("grid-template-rows:{}", if open() { "1fr" } else { "0fr" })
                }
                inert=move || (!open()).then_some("")
            >
                <div class="flex min-h-0 flex-col gap-1 overflow-hidden">{children()}</div>
            </div>
        </div>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn NavigationItem(
    href: &'static str,
    label: &'static str,
    glyph: LucideGlyph,
) -> impl IntoView {
    let state = use_context::<SidebarState>();
    let pathname = use_location().pathname;
    let class = move || {
        if is_active(&pathname.get(), href) {
            ACTIVE_ITEM_CLASS
        } else {
            INACTIVE_ITEM_CLASS
        }
    };

    view! {
        <A
            href=href
            attr:class=class
            attr:title=label
            on:click=move |_| {
                if let Some(state) = state {
                    state.close_drawer();
                }
            }
        >
            {icons::icon(glyph, ICON_CLASS)}
            <span class=LABEL_CLASS>{label}</span>
        </A>
    }
    .into_any()
}

#[component]
pub fn SidebarUser(
    #[prop(optional, into)] name: MaybeProp<String>,
    entries: Vec<MenuEntry>,
) -> impl IntoView {
    let name = name.get_untracked().filter(|name| !name.trim().is_empty());
    let avatar = match name.as_deref().map(initials) {
        Some(initials) => view! { <span class=AVATAR_CLASS>{initials}</span> }.into_any(),
        None => icons::icon(
            LucideGlyph::CircleUserRound,
            "block h-6 w-6 shrink-0 text-gray-400",
        ),
    };
    let label = name.unwrap_or_else(|| UNNAMED_USER_LABEL.to_owned());
    let content = view! {
        {avatar}
        <span class=format!("truncate {LABEL_CLASS}")>{label}</span>
    }
    .into_any();

    menu(USER_TRIGGER_CLASS, None, content, entries)
}

#[cfg(test)]
mod tests {
    use super::{initials, is_active};

    #[test]
    fn the_home_link_is_active_only_on_the_home_page() {
        assert!(is_active("/", "/"));
        assert!(!is_active("/quotes", "/"));
    }

    #[test]
    fn a_link_is_active_on_its_own_page_and_the_pages_beneath_it() {
        assert!(is_active("/quotes", "/quotes"));
        assert!(is_active("/quotes/12", "/quotes"));
        assert!(!is_active("/quotes-archive", "/quotes"));
    }

    #[test]
    fn initials_take_the_first_letter_of_the_first_two_words() {
        assert_eq!(initials("aydin aksel"), "AA");
        assert_eq!(initials("Stephanie Mary Aksel"), "SM");
        assert_eq!(initials("aydin"), "A");
    }
}
