use std::sync::Arc;

use leptos::ev::{FocusEvent, KeyboardEvent, MouseEvent};
use leptos::html::{Button, Div};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{HtmlElement, Node};
use leptos_use::{
    use_event_listener_with_options, use_timeout_fn, use_window, UseEventListenerOptions,
};

use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::icons;

const ITEM_MARKER_CLASS: &str = "menu-item";
const ITEM_CLASS: &str = "menu-item group flex w-full cursor-pointer items-center gap-2 whitespace-nowrap rounded-lg px-3 py-1.5 text-left text-gray-700 focus:bg-gray-100 focus:text-gray-900 focus:outline-none";
const DANGER_ITEM_CLASS: &str = "menu-item group flex w-full cursor-pointer items-center gap-2 whitespace-nowrap rounded-lg px-3 py-1.5 text-left text-red-700 focus:bg-red-50 focus:text-red-800 focus:outline-none";
const DANGER_ICON_CLASS: &str = "h-4 w-4 shrink-0 text-red-500 group-focus:text-red-600";
const DISABLED_ITEM_CLASS: &str = "flex w-full cursor-not-allowed items-center gap-2 whitespace-nowrap rounded-lg px-3 py-1.5 text-left text-gray-700 opacity-50";
const ICON_CLASS: &str = "h-4 w-4 shrink-0 text-gray-400 group-focus:text-gray-600";
const SHORTCUT_CLASS: &str =
    "invisible ml-auto pl-6 font-sans text-xs text-gray-500 group-focus:visible";
const PANEL_CLASS: &str = "fixed z-50 w-max min-w-48 rounded-xl border border-gray-200 bg-white p-1 text-sm/6 shadow-lg duration-100 ease-out will-change-[opacity] focus:outline-none motion-reduce:transition-none";
const PANEL_OPEN_CLASS: &str = "visible opacity-100 transition-opacity";
const PANEL_CLOSED_CLASS: &str = "invisible opacity-0 transition-[opacity,visibility]";
const PANEL_GAP_PIXELS: f64 = 4.0;
const VIEWPORT_MARGIN_PIXELS: f64 = 8.0;
const TRIGGER_OPEN_CLASS: &str = "aria-expanded:bg-gray-100";
pub(crate) const TYPEAHEAD_RESET_DELAY_MILLISECONDS: f64 = 350.0;

pub struct MenuItem {
    label: &'static str,
    glyph: icons::LucideGlyph,
    target: MenuTarget,
    shortcut: Option<&'static str>,
    disabled: bool,
    danger: bool,
}

enum MenuTarget {
    Action(Arc<dyn Fn() + Send + Sync>),
    Download(Signal<String>),
    Link(String),
}

impl MenuItem {
    #[must_use]
    pub fn action(
        label: &'static str,
        glyph: icons::LucideGlyph,
        on_select: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self::new(label, glyph, MenuTarget::Action(Arc::new(on_select)))
    }

    #[must_use]
    pub fn download(
        label: &'static str,
        glyph: icons::LucideGlyph,
        href: impl Into<Signal<String>>,
    ) -> Self {
        Self::new(label, glyph, MenuTarget::Download(href.into()))
    }

    #[must_use]
    pub fn link(label: &'static str, glyph: icons::LucideGlyph, href: impl Into<String>) -> Self {
        Self::new(label, glyph, MenuTarget::Link(href.into()))
    }

    #[must_use]
    pub fn shortcut(self, keys: &'static str) -> Self {
        Self {
            shortcut: Some(keys),
            ..self
        }
    }

    #[must_use]
    pub fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    #[must_use]
    pub fn danger(self) -> Self {
        Self {
            danger: true,
            ..self
        }
    }

    const fn class(&self) -> &'static str {
        if self.danger {
            DANGER_ITEM_CLASS
        } else {
            ITEM_CLASS
        }
    }

    fn new(label: &'static str, glyph: icons::LucideGlyph, target: MenuTarget) -> Self {
        Self {
            label,
            glyph,
            target,
            shortcut: None,
            disabled: false,
            danger: false,
        }
    }
}

pub enum MenuEntry {
    Item(MenuItem),
    Separator,
    Section {
        heading: &'static str,
        items: Vec<MenuItem>,
    },
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        Self::Item(item)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Movement {
    First,
    Last,
    Next,
    Previous,
}

pub(crate) fn moved_position(
    current: Option<usize>,
    count: usize,
    movement: Movement,
) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(match (movement, current) {
        (Movement::First, _) | (Movement::Next, None) => 0,
        (Movement::Last, _) | (Movement::Previous, None) => last,
        (Movement::Next, Some(index)) => index.saturating_add(1).min(last),
        (Movement::Previous, Some(index)) => index.saturating_sub(1),
    })
}

pub(crate) fn typeahead_position(
    labels: &[String],
    current: Option<usize>,
    query: &str,
) -> Option<usize> {
    let query = query.to_lowercase();
    let is_extending_query = query.chars().count() > 1;
    let start = match current {
        Some(index) if is_extending_query => index,
        Some(index) => index.saturating_add(1),
        None => 0,
    };
    (0..labels.len())
        .filter_map(|offset| start.saturating_add(offset).checked_rem(labels.len()))
        .find(|&position| {
            labels
                .get(position)
                .is_some_and(|label| label.to_lowercase().starts_with(&query))
        })
}

fn enabled_items(panel: &HtmlElement) -> Vec<HtmlElement> {
    let collection = panel.get_elements_by_class_name(ITEM_MARKER_CLASS);
    (0..collection.length())
        .filter_map(|index| collection.item(index))
        .filter_map(|element| element.dyn_into::<HtmlElement>().ok())
        .collect()
}

fn focused_position(items: &[HtmlElement]) -> Option<usize> {
    let active = document().active_element()?;
    items
        .iter()
        .position(|item| active.is_same_node(Some(item.as_ref())))
}

fn focus_element(element: &HtmlElement) {
    element.focus().unwrap_or_default();
}

fn focus_item(panel: &HtmlElement, movement: Movement) {
    let items = enabled_items(panel);
    let target = moved_position(focused_position(&items), items.len(), movement);
    if let Some(item) = target.and_then(|position| items.get(position)) {
        focus_element(item);
    }
}

fn focus_typeahead_match(panel: &HtmlElement, query: &str) {
    let items = enabled_items(panel);
    let labels: Vec<String> = items
        .iter()
        .map(|item| item.text_content().unwrap_or_default().trim().to_owned())
        .collect();
    let target = typeahead_position(&labels, focused_position(&items), query);
    if let Some(item) = target.and_then(|position| items.get(position)) {
        focus_element(item);
    }
}

fn activate_focused_item(panel: &HtmlElement) {
    let items = enabled_items(panel);
    if let Some(item) = focused_position(&items).and_then(|position| items.get(position)) {
        item.click();
    }
}

pub(crate) fn is_typeahead_key(event: &KeyboardEvent, query_in_progress: bool) -> bool {
    let key = event.key();
    let is_single_character = key.chars().count() == 1;
    let has_modifier = event.ctrl_key() || event.meta_key() || event.alt_key();
    is_single_character && !has_modifier && (key != " " || query_in_progress)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum VerticalSide {
    #[default]
    Below,
    Above,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum HorizontalAlignment {
    #[default]
    End,
    Start,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Placement {
    side: VerticalSide,
    alignment: HorizontalAlignment,
}

#[derive(Clone, Copy, Debug, Default)]
struct PanelPosition {
    placement: Placement,
    trigger: Bounds,
    viewport: Size,
}

impl PanelPosition {
    fn style(self) -> String {
        let Self {
            placement,
            trigger,
            viewport,
        } = self;
        let vertical = match placement.side {
            VerticalSide::Below => format!("top:{}px", trigger.bottom + PANEL_GAP_PIXELS),
            VerticalSide::Above => format!(
                "bottom:{}px",
                viewport.height - trigger.top + PANEL_GAP_PIXELS
            ),
        };
        let horizontal = match placement.alignment {
            HorizontalAlignment::End => format!("right:{}px", viewport.width - trigger.right),
            HorizontalAlignment::Start => format!("left:{}px", trigger.left),
        };
        format!("{vertical};{horizontal}")
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Bounds {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

#[derive(Clone, Copy, Debug, Default)]
struct Size {
    width: f64,
    height: f64,
}

fn choose_placement(trigger: Bounds, panel: Size, viewport: Size) -> Placement {
    let fits_end_aligned = trigger.right - panel.width >= VIEWPORT_MARGIN_PIXELS;
    let fits_start_aligned = trigger.left + panel.width <= viewport.width - VIEWPORT_MARGIN_PIXELS;
    let alignment = if !fits_end_aligned && fits_start_aligned {
        HorizontalAlignment::Start
    } else {
        HorizontalAlignment::End
    };
    let panel_extent = PANEL_GAP_PIXELS + panel.height;
    let fits_below = trigger.bottom + panel_extent <= viewport.height - VIEWPORT_MARGIN_PIXELS;
    let fits_above = trigger.top - panel_extent >= VIEWPORT_MARGIN_PIXELS;
    let side = if !fits_below && fits_above {
        VerticalSide::Above
    } else {
        VerticalSide::Below
    };
    Placement { side, alignment }
}

fn measure_position(trigger: &HtmlElement, panel: &HtmlElement) -> PanelPosition {
    let Some(root) = document().document_element() else {
        return PanelPosition::default();
    };
    let trigger_rectangle = trigger.get_bounding_client_rect();
    let trigger_bounds = Bounds {
        left: trigger_rectangle.left(),
        right: trigger_rectangle.right(),
        top: trigger_rectangle.top(),
        bottom: trigger_rectangle.bottom(),
    };
    let panel_size = Size {
        width: f64::from(panel.offset_width()),
        height: f64::from(panel.offset_height()),
    };
    let viewport_size = Size {
        width: f64::from(root.client_width()),
        height: f64::from(root.client_height()),
    };
    PanelPosition {
        placement: choose_placement(trigger_bounds, panel_size, viewport_size),
        trigger: trigger_bounds,
        viewport: viewport_size,
    }
}

#[derive(Clone, Copy)]
struct MenuState {
    open: RwSignal<bool>,
    position: RwSignal<PanelPosition>,
    trigger: NodeRef<Button>,
    panel: NodeRef<Div>,
}

impl MenuState {
    fn open_and_focus(self, movement: Option<Movement>) {
        if let (Some(trigger), Some(panel)) =
            (self.trigger.get_untracked(), self.panel.get_untracked())
        {
            self.position.set(measure_position(&trigger, &panel));
        }
        self.open.set(true);
        request_animation_frame(move || {
            let Some(panel) = self.panel.get_untracked() else {
                return;
            };
            match movement {
                Some(movement) => focus_item(&panel, movement),
                None => focus_element(&panel),
            }
        });
    }

    fn close(self) {
        self.open.set(false);
    }

    fn close_and_refocus(self) {
        self.close();
        if let Some(trigger) = self.trigger.get_untracked() {
            focus_element(&trigger);
        }
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn OverflowMenu(entries: Vec<MenuEntry>) -> impl IntoView {
    menu(
        &icon_button_class(ButtonTone::Secondary, ButtonSize::Standard),
        Some("More Actions"),
        icons::icon(icons::LucideGlyph::Ellipsis, "h-5 w-5"),
        entries,
    )
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn DropdownMenu(label: &'static str, entries: Vec<MenuEntry>) -> impl IntoView {
    let content = view! {
        {label}
        {icons::icon(icons::LucideGlyph::ChevronDown, "h-4 w-4 shrink-0 opacity-70")}
    }
    .into_any();
    menu(
        &button_class(ButtonTone::Secondary, ButtonSize::Standard),
        None,
        content,
        entries,
    )
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn RowActions(
    #[prop(optional)] on_edit: Option<Callback<()>>,
    #[prop(default = "Edit")] edit_label: &'static str,
    #[prop(optional)] entries: Vec<MenuEntry>,
) -> impl IntoView {
    let compact_ghost = icon_button_class(ButtonTone::Ghost, ButtonSize::Compact);
    let edit_button = on_edit.map(|on_edit| {
        view! {
            <button
                type="button"
                class=compact_ghost.clone()
                aria-label=edit_label
                title=edit_label
                on:click=move |_| on_edit.run(())
            >
                {icons::icon(icons::LucideGlyph::Pencil, "h-4 w-4 text-gray-500")}
            </button>
        }
    });
    let overflow = (!entries.is_empty()).then(|| {
        menu(
            &compact_ghost,
            Some("More Actions"),
            icons::icon(icons::LucideGlyph::Ellipsis, "h-4 w-4 text-gray-500"),
            entries,
        )
    });
    view! {
        <div
            class="-my-1.5 flex items-center justify-end gap-0.5"
            on:click=|event: MouseEvent| event.stop_propagation()
        >
            {edit_button}
            {overflow}
        </div>
    }
    .into_any()
}

/// Follows the WAI-ARIA menu button pattern
/// (<https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/>) with roving
/// focus, so the highlighted item is the focused element.
#[expect(
    clippy::too_many_lines,
    reason = "one widget: the trigger, the panel, and their key handling belong together"
)]
fn menu(
    trigger_class: &str,
    accessible_label: Option<&'static str>,
    trigger_content: AnyView,
    entries: Vec<MenuEntry>,
) -> AnyView {
    let state = MenuState {
        open: RwSignal::new(false),
        position: RwSignal::new(PanelPosition::default()),
        trigger: NodeRef::new(),
        panel: NodeRef::new(),
    };
    let wrapper: NodeRef<Div> = NodeRef::new();
    drop(use_event_listener_with_options(
        use_window(),
        leptos::ev::scroll,
        move |_| {
            if state.open.get_untracked() {
                state.close();
            }
        },
        UseEventListenerOptions::default()
            .capture(true)
            .passive(true),
    ));
    let typeahead_query = StoredValue::new(String::new());
    let reset_typeahead = use_timeout_fn(
        move |()| typeahead_query.set_value(String::new()),
        TYPEAHEAD_RESET_DELAY_MILLISECONDS,
    )
    .start;

    let on_trigger_keydown = move |event: KeyboardEvent| {
        let movement = match event.key().as_str() {
            "Enter" | " " | "ArrowDown" => Movement::First,
            "ArrowUp" => Movement::Last,
            _ => return,
        };
        event.prevent_default();
        state.open_and_focus(Some(movement));
    };

    let on_trigger_click = move |_: MouseEvent| {
        if state.open.get_untracked() {
            state.close_and_refocus();
        } else {
            state.open_and_focus(None);
        }
    };

    let extend_typeahead = move |panel: &HtmlElement, key: &str| {
        typeahead_query.update_value(|query| query.push_str(key));
        focus_typeahead_match(panel, &typeahead_query.get_value());
        reset_typeahead(());
    };

    let on_panel_keydown = move |event: KeyboardEvent| {
        let Some(panel) = state.panel.get_untracked() else {
            return;
        };
        let query_in_progress = typeahead_query.with_value(|query| !query.is_empty());
        let movement = match event.key().as_str() {
            "ArrowDown" => Some(Movement::Next),
            "ArrowUp" => Some(Movement::Previous),
            "Home" | "PageUp" => Some(Movement::First),
            "End" | "PageDown" => Some(Movement::Last),
            _ => None,
        };
        if let Some(movement) = movement {
            event.prevent_default();
            focus_item(&panel, movement);
            return;
        }
        if is_typeahead_key(&event, query_in_progress) {
            event.prevent_default();
            extend_typeahead(&panel, &event.key());
            return;
        }
        match event.key().as_str() {
            "Enter" | " " => {
                event.prevent_default();
                activate_focused_item(&panel);
            }
            "Escape" => {
                event.prevent_default();
                event.stop_propagation();
                state.close_and_refocus();
            }
            "Tab" => state.close_and_refocus(),
            _ => {}
        }
    };

    let on_wrapper_focusout = move |event: FocusEvent| {
        let Some(wrapper) = wrapper.get_untracked() else {
            return;
        };
        let focus_stayed_inside = event
            .related_target()
            .and_then(|target| target.dyn_into::<Node>().ok())
            .is_some_and(|node| wrapper.contains(Some(&node)));
        if !focus_stayed_inside {
            state.close();
        }
    };

    let on_panel_mouseleave = move |_: MouseEvent| {
        if let Some(panel) = state.panel.get_untracked() {
            if state.open.get_untracked() {
                focus_element(&panel);
            }
        }
    };

    let entries = entries
        .into_iter()
        .map(|entry| menu_entry(state, entry))
        .collect_view();

    view! {
        <div class="relative" node_ref=wrapper on:focusout=on_wrapper_focusout>
            <button
                type="button"
                node_ref=state.trigger
                aria-haspopup="menu"
                aria-label=accessible_label
                title=accessible_label
                aria-expanded=move || state.open.get().to_string()
                class=format!("{trigger_class} {TRIGGER_OPEN_CLASS}")
                on:keydown=on_trigger_keydown
                on:keyup=|event: KeyboardEvent| {
                    if event.key() == " " {
                        event.prevent_default();
                    }
                }
                on:mousedown=move |event: MouseEvent| {
                    if state.open.get_untracked() {
                        event.prevent_default();
                    }
                }
                on:click=on_trigger_click
            >
                {trigger_content}
            </button>
            <div
                role="menu"
                tabindex="-1"
                node_ref=state.panel
                class=move || {
                    let visibility = if state.open.get() {
                        PANEL_OPEN_CLASS
                    } else {
                        PANEL_CLOSED_CLASS
                    };
                    format!("{PANEL_CLASS} {visibility}")
                }
                style=move || state.position.get().style()
                on:keydown=on_panel_keydown
                on:mousedown=|event: MouseEvent| event.prevent_default()
                on:mouseleave=on_panel_mouseleave
            >
                {entries}
            </div>
        </div>
    }
    .into_any()
}

fn menu_entry(state: MenuState, entry: MenuEntry) -> AnyView {
    match entry {
        MenuEntry::Item(item) => menu_item(state, item),
        MenuEntry::Separator => {
            view! { <div role="separator" class="my-1 h-px bg-gray-200"></div> }.into_any()
        }
        MenuEntry::Section { heading, items } => {
            let items = items
                .into_iter()
                .map(|item| menu_item(state, item))
                .collect_view();
            view! {
                <div role="group" aria-label=heading>
                    <div
                        aria-hidden="true"
                        class="px-3 pb-1 pt-1.5 text-xs font-medium text-gray-500"
                    >
                        {heading}
                    </div>
                    {items}
                </div>
            }
            .into_any()
        }
    }
}

fn item_content(item: &MenuItem) -> AnyView {
    view! {
        {icons::icon(item.glyph, if item.danger { DANGER_ICON_CLASS } else { ICON_CLASS })}
        {item.label}
        {item.shortcut.map(|keys| view! { <kbd class=SHORTCUT_CLASS>{keys}</kbd> })}
    }
    .into_any()
}

fn focus_on_hover(event: &MouseEvent) {
    let hovered = event
        .current_target()
        .and_then(|target| target.dyn_into::<HtmlElement>().ok());
    let Some(hovered) = hovered else {
        return;
    };
    let already_focused = document()
        .active_element()
        .is_some_and(|active| active.is_same_node(Some(hovered.as_ref())));
    if !already_focused {
        focus_element(&hovered);
    }
}

fn menu_item(state: MenuState, item: MenuItem) -> AnyView {
    let content = item_content(&item);
    let class = item.class();
    if item.disabled {
        return view! {
            <div role="menuitem" aria-disabled="true" class=DISABLED_ITEM_CLASS>
                {content}
            </div>
        }
        .into_any();
    }
    match item.target {
        MenuTarget::Action(on_select) => view! {
            <button
                type="button"
                role="menuitem"
                tabindex="-1"
                class=class
                on:mousemove=|event: MouseEvent| focus_on_hover(&event)
                on:click=move |_| {
                    state.close_and_refocus();
                    on_select();
                }
            >
                {content}
            </button>
        }
        .into_any(),
        MenuTarget::Download(href) => view! {
            <a
                href=move || href.get()
                download
                role="menuitem"
                tabindex="-1"
                class=class
                on:mousemove=|event: MouseEvent| focus_on_hover(&event)
                on:click=move |_| state.close_and_refocus()
            >
                {content}
            </a>
        }
        .into_any(),
        MenuTarget::Link(href) => view! {
            <a
                href=href
                role="menuitem"
                tabindex="-1"
                class=class
                on:mousemove=|event: MouseEvent| focus_on_hover(&event)
                on:click=move |_| state.close()
            >
                {content}
            </a>
        }
        .into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    const VIEWPORT: Size = Size {
        width: 1000.0,
        height: 800.0,
    };
    const PANEL: Size = Size {
        width: 200.0,
        height: 150.0,
    };

    fn trigger_at(left: f64, top: f64) -> Bounds {
        Bounds {
            left,
            right: left + 40.0,
            top,
            bottom: top + 40.0,
        }
    }

    #[test]
    fn opens_below_aligned_to_the_trigger_end_when_there_is_room() {
        let placement = choose_placement(trigger_at(500.0, 100.0), PANEL, VIEWPORT);
        assert_eq!(placement, Placement::default());
    }

    #[test]
    fn aligns_to_the_trigger_start_near_the_left_edge() {
        let placement = choose_placement(trigger_at(20.0, 100.0), PANEL, VIEWPORT);
        assert_eq!(placement.alignment, HorizontalAlignment::Start);
    }

    #[test]
    fn opens_above_near_the_bottom_edge() {
        let placement = choose_placement(trigger_at(500.0, 700.0), PANEL, VIEWPORT);
        assert_eq!(placement.side, VerticalSide::Above);
    }

    #[test]
    fn stays_below_when_neither_side_has_room() {
        let tall_panel = Size {
            width: 200.0,
            height: 700.0,
        };
        let placement = choose_placement(trigger_at(500.0, 300.0), tall_panel, VIEWPORT);
        assert_eq!(placement.side, VerticalSide::Below);
    }

    #[test]
    fn moves_down_from_nothing_to_the_first_item() {
        assert_eq!(moved_position(None, 3, Movement::Next), Some(0));
    }

    #[test]
    fn moves_up_from_nothing_to_the_last_item() {
        assert_eq!(moved_position(None, 3, Movement::Previous), Some(2));
    }

    #[test]
    fn stops_at_either_end_of_the_menu() {
        assert_eq!(moved_position(Some(2), 3, Movement::Next), Some(2));
        assert_eq!(moved_position(Some(0), 3, Movement::Previous), Some(0));
    }

    #[test]
    fn jumps_to_either_end_from_anywhere() {
        assert_eq!(moved_position(Some(1), 3, Movement::First), Some(0));
        assert_eq!(moved_position(Some(1), 3, Movement::Last), Some(2));
    }

    #[test]
    fn moves_nowhere_in_an_empty_menu() {
        assert_eq!(moved_position(None, 0, Movement::Next), None);
    }

    #[test]
    fn typeahead_finds_the_first_item_starting_with_the_letter() {
        let names = labels(&["Edit", "Duplicate", "Delete"]);
        assert_eq!(typeahead_position(&names, None, "d"), Some(1));
    }

    #[test]
    fn repeating_a_letter_cycles_through_its_matches() {
        let names = labels(&["Edit", "Duplicate", "Delete"]);
        assert_eq!(typeahead_position(&names, Some(1), "d"), Some(2));
        assert_eq!(typeahead_position(&names, Some(2), "d"), Some(1));
    }

    #[test]
    fn typing_more_letters_narrows_from_the_current_item() {
        let names = labels(&["Edit", "Duplicate", "Delete"]);
        assert_eq!(typeahead_position(&names, Some(1), "du"), Some(1));
        assert_eq!(typeahead_position(&names, Some(1), "de"), Some(2));
    }

    #[test]
    fn typeahead_ignores_case_and_misses_cleanly() {
        let names = labels(&["Edit", "Duplicate"]);
        assert_eq!(typeahead_position(&names, None, "E"), Some(0));
        assert_eq!(typeahead_position(&names, None, "x"), None);
    }
}
