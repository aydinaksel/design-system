use codee::string::FromToStringCodec;
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html::{Aside, Button};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};

use crate::brand::brand;
use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::editing::focus_is_within_or_lost;
use crate::element_key::ElementKey;
use crate::icons::{self, LucideGlyph};
use crate::menu::{menu, MenuEntry};
use crate::popover::keep_tab_inside;
use crate::tooltip::{Tooltip, TooltipSide};

const COOKIE_PATH: &str = "/";
const COOKIE_MAX_AGE_MILLISECONDS: i64 = 365 * 24 * 60 * 60 * 1000;
const CLOSED_SECTION_SEPARATOR: char = ',';
const UNNAMED_USER_LABEL: &str = "Account";

const ICON_CLASS: &str = "block h-4 w-4 shrink-0";
const LABEL_CLASS: &str =
    "truncate transition-opacity duration-200 sm:group-data-[rail=true]/sidebar:opacity-0";
const TOP_BAR_BRAND_CLASS: &str =
    "whitespace-nowrap text-sm font-semibold tracking-tight text-gray-900";
const BRAND_CLASS: &str = "absolute left-4 whitespace-nowrap text-sm font-semibold tracking-tight text-gray-900 transition-opacity duration-200 sm:group-data-[rail=true]/sidebar:pointer-events-none sm:group-data-[rail=true]/sidebar:opacity-0";
const ACTIVE_ITEM_CLASS: &str = "flex items-center gap-3 overflow-hidden whitespace-nowrap rounded-md p-2 text-sm font-medium bg-gray-100 text-gray-900 focus-ring-inset";
const INACTIVE_ITEM_CLASS: &str = "flex items-center gap-3 overflow-hidden whitespace-nowrap rounded-md p-2 text-sm text-gray-600 hover:bg-gray-100 hover:text-gray-900 focus-ring-inset";
const SECTION_TOGGLE_CLASS: &str = "flex h-7 w-full cursor-pointer items-center justify-between whitespace-nowrap rounded-md px-2 text-xs font-medium text-gray-500 transition-opacity duration-200 hover:text-gray-900 sm:group-data-[rail=true]/sidebar:pointer-events-none sm:group-data-[rail=true]/sidebar:opacity-0 focus-ring-inset";
const SECTION_DIVIDER_CLASS: &str = "pointer-events-none absolute inset-x-1 top-1/2 h-px bg-gray-200 opacity-0 transition-opacity duration-200 sm:group-data-[rail=true]/sidebar:opacity-100";
const USER_TRIGGER_CLASS: &str = "flex w-full cursor-pointer items-center gap-3 overflow-hidden whitespace-nowrap rounded-md p-2 text-left text-sm text-gray-600 hover:bg-gray-100 hover:text-gray-900 focus-ring-inset";
const PANEL_CLASS: &str = "flex h-full w-full flex-col border-r border-gray-200 bg-white";
const ASIDE_CLASS: &str = "group/sidebar z-50 w-52 shrink-0 overflow-hidden bg-white duration-200 motion-reduce:duration-0 max-sm:fixed max-sm:inset-y-0 max-sm:left-0 max-sm:shadow-lg sm:h-full sm:transition-[width]";
const RAIL_WIDTH_CLASS: &str = "sm:w-12";
const DRAWER_OPEN_CLASS: &str = "max-sm:visible max-sm:translate-x-0 max-sm:transition-[translate]";
const DRAWER_CLOSED_CLASS: &str =
    "max-sm:invisible max-sm:-translate-x-full max-sm:transition-[translate,visibility]";
const BACKDROP_CLASS: &str =
    "fixed inset-0 z-40 bg-black/40 duration-200 motion-reduce:duration-0 sm:hidden";
const BACKDROP_OPEN_CLASS: &str = "visible opacity-100 transition-opacity";
const BACKDROP_CLOSED_CLASS: &str = "invisible opacity-0 transition-[opacity,visibility]";

#[derive(Clone, Copy)]
struct SidebarState {
    rail: Signal<bool>,
    closed_sections: Signal<Option<String>>,
    set_closed_sections: WriteSignal<Option<String>>,
    drawer_open: RwSignal<bool>,
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
        self.drawer_open.set(false);
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

fn focus_button(button: NodeRef<Button>) {
    if let Some(element) = button.get_untracked() {
        element.focus().unwrap_or_default();
    }
}

fn move_focus_with_drawer(
    drawer_open: RwSignal<bool>,
    aside: NodeRef<Aside>,
    close_button: NodeRef<Button>,
    open_button: NodeRef<Button>,
) {
    Effect::new(move |was_open: Option<bool>| {
        let open = drawer_open.get();
        if was_open.is_none_or(|was_open| was_open == open) {
            return open;
        }
        let focus_was_inside = aside
            .get_untracked()
            .is_none_or(|panel| focus_is_within_or_lost(&panel));
        if open {
            request_animation_frame(move || focus_button(close_button));
        } else if focus_was_inside {
            focus_button(open_button);
        }
        open
    });
}

fn keep_focus_in_drawer(event: &KeyboardEvent, drawer_open: RwSignal<bool>, aside: NodeRef<Aside>) {
    if !drawer_open.get_untracked() {
        return;
    }
    match event.key().as_str() {
        "Escape" => {
            event.prevent_default();
            drawer_open.set(false);
        }
        "Tab" => {
            if let Some(panel) = aside.get_untracked() {
                keep_tab_inside(&panel, event);
            }
        }
        _ => {}
    }
}

fn aside_class(drawer_open: bool, rail: bool) -> String {
    let drawer = if drawer_open {
        DRAWER_OPEN_CLASS
    } else {
        DRAWER_CLOSED_CLASS
    };
    let width = if rail { RAIL_WIDTH_CLASS } else { "" };
    format!("{ASIDE_CLASS} {drawer} {width}")
}

fn backdrop_class(drawer_open: bool) -> String {
    let motion = if drawer_open {
        BACKDROP_OPEN_CLASS
    } else {
        BACKDROP_CLOSED_CLASS
    };
    format!("{BACKDROP_CLASS} {motion}")
}

fn sidebar_id(key: ElementKey) -> String {
    format!("sidebar-{key}")
}

#[component]
fn PhoneTopBar(
    drawer_open: RwSignal<bool>,
    open_button: NodeRef<Button>,
    aside_key: ElementKey,
) -> impl IntoView {
    let brand = brand();
    view! {
        <div class="fixed inset-x-0 top-0 z-30 flex h-12 items-center gap-3 border-b border-gray-200 bg-white px-3 sm:hidden">
            <button
                node_ref=open_button
                type="button"
                aria-label="Open Navigation"
                aria-expanded=move || drawer_open.get().to_string()
                aria-controls=sidebar_id(aside_key)
                class=icon_button_class(ButtonTone::Ghost, ButtonSize::Dense)
                on:click=move |_| drawer_open.set(true)
            >
                {icons::icon(LucideGlyph::Menu, ICON_CLASS)}
            </button>
            <A href=brand.home_href attr:class=TOP_BAR_BRAND_CLASS>
                {brand.name}
            </A>
        </div>
    }
}

#[component]
fn PanelHeader(
    state: SidebarState,
    collapsed: Signal<bool>,
    set_collapsed_cookie: WriteSignal<Option<bool>>,
    close_button: NodeRef<Button>,
    aside_key: ElementKey,
) -> impl IntoView {
    let brand = brand();
    let ghost_button_class = icon_button_class(ButtonTone::Ghost, ButtonSize::Dense);
    view! {
        <div class="relative flex h-12 shrink-0 items-center border-b border-gray-200 px-2.5">
            <A
                href=brand.home_href
                attr:class=BRAND_CLASS
                attr:tabindex=move || state.rail.get().then_some("-1")
                on:click=move |_| state.close_drawer()
            >
                {brand.name}
            </A>
            <button
                type="button"
                aria-label="Toggle Navigation"
                aria-expanded=move || (!collapsed.get()).to_string()
                aria-controls=sidebar_id(aside_key)
                class=format!("ml-auto max-sm:hidden {ghost_button_class}")
                on:click=move |_| set_collapsed_cookie.set(Some(!collapsed.get_untracked()))
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
            <button
                node_ref=close_button
                type="button"
                aria-label="Close Navigation"
                class=format!("ml-auto sm:hidden {ghost_button_class}")
                on:click=move |_| state.close_drawer()
            >
                {icons::icon(LucideGlyph::X, ICON_CLASS)}
            </button>
        </div>
    }
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
    let rail = Signal::derive(move || collapsed.get() && !drawer_open.get());
    let state = SidebarState {
        rail,
        closed_sections,
        set_closed_sections,
        drawer_open,
    };
    let aside_key = ElementKey::next();
    let aside: NodeRef<Aside> = NodeRef::new();
    let open_button: NodeRef<Button> = NodeRef::new();
    let close_button: NodeRef<Button> = NodeRef::new();

    move_focus_with_drawer(drawer_open, aside, close_button, open_button);
    let on_aside_keydown =
        move |event: KeyboardEvent| keep_focus_in_drawer(&event, drawer_open, aside);

    view! {
        <PhoneTopBar drawer_open open_button aside_key />
        <div
            class=move || backdrop_class(drawer_open.get())
            aria-hidden="true"
            on:click=move |_| drawer_open.set(false)
        ></div>

        <aside
            node_ref=aside
            id=sidebar_id(aside_key)
            aria-label="Navigation"
            role=move || drawer_open.get().then_some("dialog")
            aria-modal=move || drawer_open.get().then_some("true")
            data-rail=move || rail.get().to_string()
            class=move || aside_class(drawer_open.get(), rail.get())
            on:keydown=on_aside_keydown
        >
            <Provider value=state>
                <div class=PANEL_CLASS>
                    <PanelHeader state collapsed set_collapsed_cookie close_button aside_key />
                    <nav class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto overflow-x-hidden p-2">
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
                </div>
            </Provider>
        </aside>
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
    let open = move || state.rail.get() || !state.section_closed(key);

    view! {
        <div role="group" aria-label=title class="flex flex-col">
            <div class="relative">
                <button
                    type="button"
                    class=SECTION_TOGGLE_CLASS
                    aria-expanded=move || open().to_string()
                    tabindex=move || state.rail.get().then_some("-1")
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

    let in_rail = Signal::derive(move || state.is_some_and(|state| state.rail.get()));
    view! {
        <Tooltip label=label side=TooltipSide::Beside enabled=in_rail>
            <A
                href=href
                attr:class=class
                on:click=move |_| {
                    if let Some(state) = state {
                        state.close_drawer();
                    }
                }
            >
                {icons::icon(glyph, ICON_CLASS)}
                <span class=LABEL_CLASS>{label}</span>
            </A>
        </Tooltip>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn SidebarUser(
    #[prop(optional, into)] name: MaybeProp<String>,
    entries: Vec<MenuEntry>,
) -> impl IntoView {
    let name = name.get_untracked().filter(|name| !name.trim().is_empty());
    let label = name.unwrap_or_else(|| UNNAMED_USER_LABEL.to_owned());
    let tooltip_label = label.clone();
    let content = view! {
        {icons::icon(LucideGlyph::CircleUserRound, ICON_CLASS)}
        <span class=LABEL_CLASS>{label}</span>
    }
    .into_any();

    let state = use_context::<SidebarState>();
    let in_rail = Signal::derive(move || state.is_some_and(|state| state.rail.get()));
    view! {
        <Tooltip label=tooltip_label side=TooltipSide::Beside enabled=in_rail>
            {menu(USER_TRIGGER_CLASS, None, content, entries)}
        </Tooltip>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::is_active;

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
}
