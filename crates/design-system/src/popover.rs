use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html::Button;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::HtmlElement;

use crate::editing::{focus_is_within_or_lost, FIRST_CONTROL_SELECTOR};
use crate::element_key::ElementKey;
use crate::native_popover::{LightDismiss, NativePopover, PanelAlignment, PANEL_MOTION_CLASS};

const GAP_PIXELS: u32 = 6;
const DEFAULT_WIDTH_CLASS: &str = "w-72";
const PANEL_CLASS: &str = "max-w-[calc(100vw-1rem)] whitespace-normal rounded-xl border border-edge bg-surface text-left text-sm text-gray-900 shadow-lg focus:outline-none";
const PADDED_PANEL_CLASS: &str = "p-3";
const UNPADDED_PANEL_CLASS: &str = "overflow-hidden";
const AUTOFOCUS_SELECTOR: &str = "[data-autofocus]";
const TABBABLE_SELECTOR: &str = "a[href], button:not([disabled]), input:not([disabled]):not([type=hidden]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    #[default]
    Start,
    Center,
    End,
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

pub(crate) fn keep_tab_inside(panel: &HtmlElement, event: &KeyboardEvent) {
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
    number: ElementKey,
    popover: NativePopover,
    trigger: NodeRef<Button>,
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
            number: ElementKey::next(),
            popover: NativePopover::new(RwSignal::new(false)),
            trigger: NodeRef::new(),
            titled: RwSignal::new(false),
            described: RwSignal::new(false),
        }
    }

    #[must_use]
    pub fn is_open(self) -> bool {
        self.popover.open.get()
    }

    pub fn close(self) {
        let focus_was_inside = self
            .popover
            .panel
            .get_untracked()
            .is_none_or(|panel| focus_is_within_or_lost(&panel));
        self.popover.open.set(false);
        if focus_was_inside {
            self.return_focus();
        }
    }

    pub(crate) fn reopen(self) {
        if self.popover.open.get_untracked() {
            return;
        }
        self.popover.open.set(true);
        let panel = self.popover.panel;
        request_animation_frame(move || {
            if let Some(panel) = panel.try_get_untracked().flatten() {
                focus_initial(&panel);
            }
        });
    }

    fn return_focus(self) {
        let trigger = self.trigger;
        request_animation_frame(move || {
            if let Some(element) = trigger.try_get_untracked().flatten() {
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
}

#[derive(Clone, Copy)]
struct PopoverContext {
    state: PopoverState,
    align: PopoverAlign,
    on_open: Option<Callback<()>>,
}

impl PopoverContext {
    fn toggle(self) {
        if self.state.popover.open.get_untracked() {
            self.state.close();
            return;
        }
        if let Some(on_open) = self.on_open {
            on_open.run(());
        }
        self.state.reopen();
    }

    const fn alignment(self) -> PanelAlignment {
        match self.align {
            PopoverAlign::Start => PanelAlignment::Start,
            PopoverAlign::Center => PanelAlignment::Center,
            PopoverAlign::End => PanelAlignment::End,
        }
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
    children: Children,
) -> impl IntoView {
    let context = PopoverContext {
        state: state.unwrap_or_default(),
        align,
        on_open,
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
            style=state.map(|state| state.popover.trigger_style())
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
    #[prop(optional)] unpadded: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let Some(context) = use_context::<PopoverContext>() else {
        return ().into_any();
    };
    let padding_class = if unpadded {
        UNPADDED_PANEL_CLASS
    } else {
        PADDED_PANEL_CLASS
    };
    let state = context.state;
    let popover = state.popover;
    let on_keydown = move |event: KeyboardEvent| {
        if event.default_prevented() {
            return;
        }
        match event.key().as_str() {
            "Escape" => {
                event.prevent_default();
                event.stop_propagation();
                state.close();
            }
            "Tab" => {
                if let Some(panel) = popover.panel.get_untracked() {
                    keep_tab_inside(&panel, &event);
                }
            }
            _ => {}
        }
    };
    let on_toggle = move |_| {
        if popover.closed_by_the_browser() {
            state.return_focus();
        }
    };
    let light_dismiss = LightDismiss::ClosesAndBlocksThePage;
    view! {
        <div
            node_ref=popover.panel
            id=state.panel_id()
            role="dialog"
            tabindex="-1"
            popover=light_dismiss.popover_attribute()
            data-blocks-page=light_dismiss.blocks_page_attribute()
            data-popover-panel=context.alignment().attribute()
            aria-labelledby=move || state.titled.get().then(|| state.title_id())
            aria-describedby=move || state.described.get().then(|| state.description_id())
            class=move || {
                format!("{PANEL_CLASS} {PANEL_MOTION_CLASS} {padding_class} {}", class.get())
            }
            style=popover.panel_style(GAP_PIXELS)
            on:keydown=on_keydown
            on:toggle=on_toggle
        >
            {move || popover.open.get().then(|| children())}
        </div>
    }
    .into_any()
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
