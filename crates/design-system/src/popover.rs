use std::sync::atomic::{AtomicU64, Ordering};

use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html::{Button, Div};
use leptos::portal::Portal;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{Element, EventTarget, HtmlElement, Node};
use leptos_use::{
    use_event_listener, use_event_listener_with_options, use_window, UseEventListenerOptions,
};

use crate::editing::{focus_is_within_or_lost, FIRST_CONTROL_SELECTOR};

static NEXT_POPOVER_NUMBER: AtomicU64 = AtomicU64::new(0);

const GAP_PIXELS: f64 = 6.0;
const VIEWPORT_MARGIN_PIXELS: f64 = 8.0;
const DEFAULT_WIDTH_CLASS: &str = "w-72";
const PANEL_CLASS: &str = "fixed z-50 max-w-[calc(100vw-1rem)] whitespace-normal rounded-surface border border-gray-200 bg-white p-3 text-left text-sm text-gray-900 shadow-lg focus:outline-none motion-safe:animate-fade-in motion-safe:transition-shadow";
const UNPLACED_STYLE: &str = "left:0;top:0;opacity:0";
const AUTOFOCUS_SELECTOR: &str = "[data-autofocus]";
const TABBABLE_SELECTOR: &str = "a[href], button:not([disabled]), input:not([disabled]):not([type=hidden]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dismissal {
    Escape,
    OutsideClick,
    TriggerClick,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bounds {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Extent {
    width: f64,
    height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Placement {
    left: f64,
    top: f64,
}

impl Placement {
    fn style(self) -> String {
        format!("left:{}px;top:{}px", self.left, self.top)
    }
}

fn place(trigger: Bounds, panel: Extent, viewport: Extent, align: PopoverAlign) -> Placement {
    let below = trigger.bottom + GAP_PIXELS;
    let above = trigger.top - GAP_PIXELS - panel.height;
    let fits_below = below + panel.height <= viewport.height - VIEWPORT_MARGIN_PIXELS;
    let fits_above = above >= VIEWPORT_MARGIN_PIXELS;
    let more_room_above = trigger.top > viewport.height - trigger.bottom;
    let top = if fits_below || (!fits_above && !more_room_above) {
        below
    } else {
        above
    };
    let preferred_left = match align {
        PopoverAlign::Start => trigger.left,
        PopoverAlign::Center => (trigger.left + trigger.right - panel.width) / 2.0,
        PopoverAlign::End => trigger.right - panel.width,
    };
    let rightmost_left = viewport.width - VIEWPORT_MARGIN_PIXELS - panel.width;
    Placement {
        left: preferred_left
            .min(rightmost_left)
            .max(VIEWPORT_MARGIN_PIXELS),
        top,
    }
}

fn bounds_of(element: &Element) -> Bounds {
    let rectangle = element.get_bounding_client_rect();
    Bounds {
        left: rectangle.left(),
        top: rectangle.top(),
        right: rectangle.right(),
        bottom: rectangle.bottom(),
    }
}

fn viewport() -> Option<Extent> {
    let root = document().document_element()?;
    Some(Extent {
        width: f64::from(root.client_width()),
        height: f64::from(root.client_height()),
    })
}

fn focus_initial(panel: &HtmlElement) {
    let target = [
        AUTOFOCUS_SELECTOR,
        FIRST_CONTROL_SELECTOR,
        TABBABLE_SELECTOR,
    ]
    .into_iter()
    .find_map(|selector| panel.query_selector(selector).ok().flatten())
    .and_then(|element| element.dyn_into::<HtmlElement>().ok());
    match target {
        Some(control) => control.focus().unwrap_or_default(),
        None => panel.focus().unwrap_or_default(),
    }
}

fn is_tabbable(element: &HtmlElement) -> bool {
    let rendered = element.get_client_rects().length() > 0;
    let visible = window()
        .get_computed_style(element)
        .ok()
        .flatten()
        .and_then(|style| style.get_property_value("visibility").ok())
        .is_none_or(|visibility| visibility != "hidden");
    let inert = element.closest("[inert]").ok().flatten().is_some();
    rendered && visible && !inert
}

fn tabbable_elements(panel: &HtmlElement) -> Vec<HtmlElement> {
    let Ok(nodes) = panel.query_selector_all(TABBABLE_SELECTOR) else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|position| nodes.item(position))
        .filter_map(|node| node.dyn_into::<HtmlElement>().ok())
        .filter(is_tabbable)
        .collect()
}

fn keep_tab_inside(panel: &HtmlElement, event: &KeyboardEvent) {
    let tabbables = tabbable_elements(panel);
    let (Some(first), Some(last)) = (tabbables.first(), tabbables.last()) else {
        event.prevent_default();
        return;
    };
    let active = document().active_element();
    let is_active = |element: &HtmlElement| {
        active
            .as_ref()
            .is_some_and(|active| active.is_same_node(Some(element)))
    };
    let on_panel_itself = is_active(panel);
    let wraps_to = if event.shift_key() {
        (is_active(first) || on_panel_itself).then_some(last)
    } else {
        is_active(last).then_some(first)
    };
    if let Some(target) = wraps_to {
        event.prevent_default();
        target.focus().unwrap_or_default();
    }
}

#[derive(Clone, Copy)]
pub struct PopoverState {
    number: u64,
    open: RwSignal<bool>,
    trigger: NodeRef<Button>,
    panel: NodeRef<Div>,
    placement: RwSignal<Option<Placement>>,
    titled: RwSignal<bool>,
    described: RwSignal<bool>,
}

impl Default for PopoverState {
    fn default() -> Self {
        Self::new()
    }
}

impl PopoverState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            number: NEXT_POPOVER_NUMBER.fetch_add(1, Ordering::Relaxed),
            open: RwSignal::new(false),
            trigger: NodeRef::new(),
            panel: NodeRef::new(),
            placement: RwSignal::new(None),
            titled: RwSignal::new(false),
            described: RwSignal::new(false),
        }
    }

    #[must_use]
    pub fn is_open(self) -> bool {
        self.open.get()
    }

    pub fn close(self) {
        let focus_was_inside = self
            .panel
            .get_untracked()
            .is_none_or(|panel| focus_is_within_or_lost(&panel));
        self.hide(focus_was_inside);
    }

    fn hide(self, returns_focus: bool) {
        self.open.set(false);
        self.placement.set(None);
        if !returns_focus {
            return;
        }
        let trigger = self.trigger;
        request_animation_frame(move || {
            if let Some(element) = trigger.get_untracked() {
                element.focus().unwrap_or_default();
            }
        });
    }

    fn panel_id(self) -> String {
        format!("popover-{}", self.number)
    }

    fn title_id(self) -> String {
        format!("popover-{}-title", self.number)
    }

    fn description_id(self) -> String {
        format!("popover-{}-description", self.number)
    }

    fn reposition(self, align: PopoverAlign) {
        let (Some(trigger), Some(panel), Some(viewport)) = (
            self.trigger.get_untracked(),
            self.panel.get_untracked(),
            viewport(),
        ) else {
            return;
        };
        let panel_bounds = bounds_of(&panel);
        let panel_extent = Extent {
            width: panel_bounds.right - panel_bounds.left,
            height: panel_bounds.bottom - panel_bounds.top,
        };
        self.placement.set(Some(place(
            bounds_of(&trigger),
            panel_extent,
            viewport,
            align,
        )));
    }

    fn contains(self, target: Option<EventTarget>) -> bool {
        let Some(node) = target.and_then(|target| target.dyn_into::<Node>().ok()) else {
            return false;
        };
        let in_panel = self
            .panel
            .get_untracked()
            .is_some_and(|panel| panel.contains(Some(&node)));
        let on_trigger = self
            .trigger
            .get_untracked()
            .is_some_and(|trigger| trigger.contains(Some(&node)));
        in_panel || on_trigger
    }
}

#[derive(Clone, Copy)]
struct PopoverContext {
    state: PopoverState,
    align: PopoverAlign,
    on_open: Option<Callback<()>>,
    before_dismiss: Option<Callback<Dismissal, bool>>,
}

impl PopoverContext {
    fn toggle(self) {
        if self.state.open.get_untracked() {
            self.dismiss(Dismissal::TriggerClick);
            return;
        }
        if let Some(on_open) = self.on_open {
            on_open.run(());
        }
        self.state.open.set(true);
    }

    fn dismiss(self, reason: Dismissal) -> bool {
        let allowed = self
            .before_dismiss
            .is_none_or(|before_dismiss| before_dismiss.run(reason));
        if allowed {
            self.state.hide(matches!(
                reason,
                Dismissal::Escape | Dismissal::TriggerClick
            ));
        }
        allowed
    }

    fn watch_while_open(self) {
        let state = self.state;
        drop(use_event_listener_with_options(
            use_window(),
            leptos::ev::click,
            move |event: MouseEvent| {
                if state.contains(event.target()) || self.dismiss(Dismissal::OutsideClick) {
                    return;
                }
                event.prevent_default();
                event.stop_propagation();
            },
            UseEventListenerOptions::default().capture(true),
        ));
        drop(use_event_listener_with_options(
            use_window(),
            leptos::ev::scroll,
            move |_| state.reposition(self.align),
            UseEventListenerOptions::default()
                .capture(true)
                .passive(true),
        ));
        drop(use_event_listener(
            use_window(),
            leptos::ev::resize,
            move |_| state.reposition(self.align),
        ));
        request_animation_frame(move || {
            state.reposition(self.align);
            if let Some(panel) = state.panel.get_untracked() {
                focus_initial(&panel);
            }
        });
    }
}

#[must_use]
pub fn use_popover() -> Option<PopoverState> {
    use_context::<PopoverContext>().map(|context| context.state)
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Popover(
    #[prop(optional)] state: Option<PopoverState>,
    #[prop(optional)] align: PopoverAlign,
    #[prop(optional, into)] on_open: Option<Callback<()>>,
    #[prop(optional, into)] before_dismiss: Option<Callback<Dismissal, bool>>,
    children: Children,
) -> impl IntoView {
    let context = PopoverContext {
        state: state.unwrap_or_default(),
        align,
        on_open,
        before_dismiss,
    };
    view! { <Provider value=context>{children()}</Provider> }.into_any()
}

#[component]
pub fn PopoverTrigger(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let context = use_context::<PopoverContext>();
    let state = context.map(|context| context.state);
    let open = move || state.is_some_and(PopoverState::is_open);
    view! {
        <button
            node_ref=state.map_or_else(NodeRef::new, |state| state.trigger)
            type="button"
            class=class
            aria-haspopup="dialog"
            aria-expanded=move || open().to_string()
            aria-controls=move || state.filter(|_| open()).map(PopoverState::panel_id)
            prop:disabled=move || disabled.get()
            on:click=move |event: MouseEvent| {
                event.stop_propagation();
                if let Some(context) = context {
                    context.toggle();
                }
            }
        >
            {children()}
        </button>
    }
    .into_any()
}

#[component]
pub fn PopoverContent(
    #[prop(into, default = Signal::stored(DEFAULT_WIDTH_CLASS.to_owned()))] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let Some(context) = use_context::<PopoverContext>() else {
        return ().into_any();
    };
    let state = context.state;
    let on_keydown = move |event: KeyboardEvent| {
        if event.default_prevented() {
            return;
        }
        match event.key().as_str() {
            "Escape" => {
                event.prevent_default();
                event.stop_propagation();
                context.dismiss(Dismissal::Escape);
            }
            "Tab" => {
                if let Some(panel) = state.panel.get_untracked() {
                    keep_tab_inside(&panel, &event);
                }
            }
            _ => {}
        }
    };
    let panel = move || {
        context.watch_while_open();
        let children = children.clone();
        view! {
            <Portal>
                <div
                    node_ref=state.panel
                    id=state.panel_id()
                    role="dialog"
                    tabindex="-1"
                    aria-labelledby=move || state.titled.get().then(|| state.title_id())
                    aria-describedby=move || state.described.get().then(|| state.description_id())
                    class=move || format!("{PANEL_CLASS} {}", class.get())
                    style=move || {
                        state
                            .placement
                            .get()
                            .map_or_else(|| UNPLACED_STYLE.to_owned(), Placement::style)
                    }
                    on:keydown=on_keydown
                >
                    {children()}
                </div>
            </Portal>
        }
    };
    view! { {move || state.is_open().then(panel.clone())} }.into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn PopoverHeader(children: Children) -> impl IntoView {
    view! { <div class="mb-3 flex flex-col gap-1">{children()}</div> }
}

#[component]
pub fn PopoverTitle(
    #[prop(optional, into)] control: Option<String>,
    children: Children,
) -> impl IntoView {
    let state = use_popover();
    if let Some(state) = state {
        state.titled.set(true);
    }
    let id = state.map(PopoverState::title_id);
    let class = "block text-sm font-medium text-gray-900";
    match control {
        Some(control) => view! {
            <label id=id for=control class=class>
                {children()}
            </label>
        }
        .into_any(),
        None => view! {
            <h2 id=id class=class>
                {children()}
            </h2>
        }
        .into_any(),
    }
}

#[component]
pub fn PopoverDescription(children: Children) -> impl IntoView {
    let state = use_popover();
    if let Some(state) = state {
        state.described.set(true);
    }
    view! {
        <p id=state.map(PopoverState::description_id) class="text-sm text-gray-500">
            {children()}
        </p>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Extent = Extent {
        width: 1000.0,
        height: 800.0,
    };
    const PANEL: Extent = Extent {
        width: 288.0,
        height: 200.0,
    };

    fn trigger_at(left: f64, top: f64) -> Bounds {
        Bounds {
            left,
            top,
            right: left + 120.0,
            bottom: top + 32.0,
        }
    }

    #[test]
    fn opens_below_and_lines_up_with_the_start_of_the_trigger() {
        let placement = place(
            trigger_at(100.0, 100.0),
            PANEL,
            VIEWPORT,
            PopoverAlign::Start,
        );
        assert_eq!(
            placement,
            Placement {
                left: 100.0,
                top: 138.0
            }
        );
    }

    #[test]
    fn centres_on_the_trigger() {
        let placement = place(
            trigger_at(400.0, 100.0),
            PANEL,
            VIEWPORT,
            PopoverAlign::Center,
        );
        assert_eq!(
            placement,
            Placement {
                left: 316.0,
                top: 138.0
            }
        );
    }

    #[test]
    fn lines_up_with_the_end_of_the_trigger() {
        let placement = place(trigger_at(400.0, 100.0), PANEL, VIEWPORT, PopoverAlign::End);
        assert_eq!(
            placement,
            Placement {
                left: 232.0,
                top: 138.0
            }
        );
    }

    #[test]
    fn flips_above_when_there_is_no_room_below() {
        let placement = place(
            trigger_at(100.0, 700.0),
            PANEL,
            VIEWPORT,
            PopoverAlign::Start,
        );
        assert_eq!(
            placement,
            Placement {
                left: 100.0,
                top: 494.0
            }
        );
    }

    #[test]
    fn stays_inside_the_right_edge_of_the_viewport() {
        let placement = place(
            trigger_at(900.0, 100.0),
            PANEL,
            VIEWPORT,
            PopoverAlign::Start,
        );
        assert_eq!(
            placement,
            Placement {
                left: 704.0,
                top: 138.0
            }
        );
    }

    #[test]
    fn stays_inside_the_left_edge_of_the_viewport() {
        let placement = place(trigger_at(0.0, 100.0), PANEL, VIEWPORT, PopoverAlign::End);
        assert_eq!(
            placement,
            Placement {
                left: VIEWPORT_MARGIN_PIXELS,
                top: 138.0
            }
        );
    }
}
