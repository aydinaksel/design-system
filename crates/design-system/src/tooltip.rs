use std::cell::Cell;
use std::time::Duration;

use leptos::ev::{FocusEvent, KeyboardEvent, MouseEvent};
use leptos::html::Span;
use leptos::portal::Portal;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::element_key::ElementKey;
use crate::placement::{Alignment, AnchoredPanel, Anchoring, PreferredSide};

const BUBBLE_CLASS: &str = "pointer-events-none fixed z-[70] w-max max-w-xs whitespace-normal break-words rounded-md bg-gray-900 px-3 py-1.5 text-xs text-white shadow-md motion-safe:animate-fade-in";
const HELP_TERM_CLASS: &str = "cursor-help rounded-sm underline decoration-gray-300 decoration-dotted underline-offset-4 focus-ring";
const HOVER_DELAY: Duration = Duration::from_millis(300);
const GAP_PIXELS: f64 = 6.0;
const FOCUS_VISIBLE_SELECTOR: &str = ":focus-visible";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipAlign {
    #[default]
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipSide {
    #[default]
    Above,
    Beside,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TooltipPosition {
    pub(crate) align: TooltipAlign,
    pub(crate) side: TooltipSide,
}

impl TooltipPosition {
    const fn anchoring(self) -> Anchoring {
        let alignment = match self.align {
            TooltipAlign::Center => Alignment::Center,
            TooltipAlign::End => Alignment::End,
        };
        let side = match self.side {
            TooltipSide::Above => PreferredSide::Above,
            TooltipSide::Beside => PreferredSide::Beside,
        };
        Anchoring {
            alignment,
            side,
            gap_pixels: GAP_PIXELS,
            matches_trigger_width: false,
            reserved_height_pixels: 0.0,
        }
    }
}

fn event_element(event: &web_sys::Event) -> Option<web_sys::Element> {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
}

pub(crate) fn focus_is_visible(event: &FocusEvent) -> bool {
    event_element(event)
        .and_then(|element| element.matches(FOCUS_VISIBLE_SELECTOR).ok())
        .unwrap_or(false)
}

fn comes_from(event: &web_sys::Event, selector: Option<&str>) -> bool {
    selector.is_some_and(|selector| {
        event_element(event)
            .and_then(|element| element.closest(selector).ok().flatten())
            .is_some()
    })
}

thread_local! {
    static SHOWING_TOOLTIP: Cell<Option<RwSignal<bool>>> = const { Cell::new(None) };
}

fn become_the_only_tooltip(visible: RwSignal<bool>) {
    let previous = SHOWING_TOOLTIP.with(|showing| showing.replace(Some(visible)));
    if let Some(previous) = previous.filter(|previous| *previous != visible) {
        previous.try_set(false);
    }
    visible.try_set(true);
}

#[derive(Clone, Copy)]
pub(crate) struct TooltipControl {
    visible: RwSignal<bool>,
    enabled: Signal<bool>,
    timer: StoredValue<Option<TimeoutHandle>>,
}

impl TooltipControl {
    pub(crate) fn new() -> Self {
        Self::enabled_when(Signal::stored(true))
    }

    pub(crate) fn enabled_when(enabled: Signal<bool>) -> Self {
        let control = Self {
            visible: RwSignal::new(false),
            enabled,
            timer: StoredValue::new(None),
        };
        on_cleanup(move || control.cancel_timer());
        control
    }

    fn cancel_timer(self) {
        if let Some(timer) = self.timer.try_update_value(Option::take).flatten() {
            timer.clear();
        }
    }

    pub(crate) fn show_after_delay(self) {
        let waiting = self.timer.try_with_value(Option::is_some).unwrap_or(false);
        if waiting || self.visible.get_untracked() || !self.enabled.get_untracked() {
            return;
        }
        let visible = self.visible;
        let timer = self.timer;
        let started = set_timeout_with_handle(
            move || {
                timer.try_set_value(None);
                become_the_only_tooltip(visible);
            },
            HOVER_DELAY,
        );
        if let Ok(handle) = started {
            self.timer.set_value(Some(handle));
        }
    }

    pub(crate) fn show_now(self) {
        self.cancel_timer();
        if self.enabled.get_untracked() {
            become_the_only_tooltip(self.visible);
        }
    }

    pub(crate) fn hide(self) {
        self.cancel_timer();
        self.visible.set(false);
    }

    pub(crate) fn bubble(
        self,
        label: Signal<String>,
        anchor: impl Fn() -> Option<web_sys::Element> + Copy + Send + Sync + 'static,
        position: TooltipPosition,
    ) -> AnyView {
        let showing = move || self.visible.get() && self.enabled.get();
        view! { {move || showing().then(|| bubble_view(label, anchor, position))} }.into_any()
    }
}

fn bubble_view(
    label: Signal<String>,
    anchor: impl Fn() -> Option<web_sys::Element> + Clone + 'static,
    position: TooltipPosition,
) -> AnyView {
    let panel = AnchoredPanel::new();
    panel.follow(Signal::stored(true), anchor, position.anchoring());
    view! {
        <Portal>
            <div
                node_ref=panel.node
                aria-hidden="true"
                class=BUBBLE_CLASS
                style=move || panel.style()
            >
                {move || label.get()}
            </div>
        </Portal>
    }
    .into_any()
}

fn overflows(element: &web_sys::Element) -> bool {
    element.scroll_width() > element.client_width()
}

#[derive(Clone, Copy)]
pub(crate) struct OverflowTooltip {
    control: TooltipControl,
    pub(crate) anchor: NodeRef<Span>,
    ignored: Option<&'static str>,
}

impl OverflowTooltip {
    pub(crate) fn new() -> Self {
        Self {
            control: TooltipControl::new(),
            anchor: NodeRef::new(),
            ignored: None,
        }
    }

    pub(crate) fn ignoring(ignored: &'static str) -> Self {
        Self {
            ignored: Some(ignored),
            ..Self::new()
        }
    }

    fn is_cut_off(self) -> bool {
        self.anchor.get_untracked().is_some_and(|anchor| {
            overflows(&anchor)
                || anchor
                    .first_element_child()
                    .is_some_and(|first| overflows(&first))
        })
    }

    pub(crate) fn pointer_over(self, event: &MouseEvent) {
        if comes_from(event, self.ignored) || !self.is_cut_off() {
            self.control.hide();
        } else {
            self.control.show_after_delay();
        }
    }

    pub(crate) fn focused(self, event: &FocusEvent) {
        if focus_is_visible(event) && !comes_from(event, self.ignored) && self.is_cut_off() {
            self.control.show_now();
        }
    }

    pub(crate) fn hide(self) {
        self.control.hide();
    }

    pub(crate) fn bubble(self, text: &str) -> AnyView {
        let anchor = self.anchor;
        self.control.bubble(
            Signal::stored(text.to_owned()),
            move || anchor.get_untracked().map(Into::into),
            TooltipPosition::default(),
        )
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Tooltip(
    #[prop(into)] label: String,
    #[prop(optional)] align: TooltipAlign,
    #[prop(optional)] side: TooltipSide,
    #[prop(optional, into)] id: Option<String>,
    #[prop(into, default = Signal::stored(true))] enabled: Signal<bool>,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let control = TooltipControl::enabled_when(enabled);
    let anchor: NodeRef<Span> = NodeRef::new();
    let description = id.map(|id| {
        view! {
            <span id=id class="sr-only">
                {label.clone()}
            </span>
        }
    });
    let layout = match side {
        TooltipSide::Above => "inline-flex",
        TooltipSide::Beside => "flex flex-col",
    };
    let wrapper_class = format!("{layout} {class}");
    view! {
        <span
            node_ref=anchor
            class=wrapper_class
            on:mouseenter=move |_| control.show_after_delay()
            on:mouseleave=move |_| control.hide()
            on:pointerdown=move |_| control.hide()
            on:focusin=move |event: FocusEvent| {
                if focus_is_visible(&event) {
                    control.show_now();
                }
            }
            on:focusout=move |_| control.hide()
            on:keydown=move |event: KeyboardEvent| {
                if event.key() == "Escape" {
                    control.hide();
                }
            }
        >
            {children()}
            {description}
            {control
                .bubble(
                    Signal::stored(label),
                    move || anchor.get_untracked().map(Into::into),
                    TooltipPosition { align, side },
                )}
        </span>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn HelpTerm(
    #[prop(into)] help: String,
    #[prop(optional, into)] help_id: Option<String>,
    children: Children,
) -> impl IntoView {
    let help_id = help_id.unwrap_or_else(|| format!("help-term-{}", ElementKey::next()));
    view! {
        <Tooltip label=help id=help_id.clone()>
            <span tabindex="0" aria-describedby=help_id class=HELP_TERM_CLASS>
                {children()}
            </span>
        </Tooltip>
    }
    .into_any()
}
