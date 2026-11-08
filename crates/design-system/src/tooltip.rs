use std::time::Duration;

use leptos::ev::{FocusEvent, KeyboardEvent};
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

impl TooltipAlign {
    const fn anchoring(self) -> Anchoring {
        let alignment = match self {
            Self::Center => Alignment::Center,
            Self::End => Alignment::End,
        };
        Anchoring {
            alignment,
            side: PreferredSide::Above,
            gap_pixels: GAP_PIXELS,
            matches_trigger_width: false,
            reserved_height_pixels: 0.0,
        }
    }
}

pub(crate) fn focus_is_visible(event: &FocusEvent) -> bool {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.matches(FOCUS_VISIBLE_SELECTOR).ok())
        .unwrap_or(false)
}

#[derive(Clone, Copy)]
pub(crate) struct TooltipControl {
    visible: RwSignal<bool>,
    timer: StoredValue<Option<TimeoutHandle>>,
}

impl TooltipControl {
    pub(crate) fn new() -> Self {
        let control = Self {
            visible: RwSignal::new(false),
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
        if waiting || self.visible.get_untracked() {
            return;
        }
        let visible = self.visible;
        let timer = self.timer;
        let started = set_timeout_with_handle(
            move || {
                timer.try_set_value(None);
                visible.try_set(true);
            },
            HOVER_DELAY,
        );
        if let Ok(handle) = started {
            self.timer.set_value(Some(handle));
        }
    }

    pub(crate) fn show_now(self) {
        self.cancel_timer();
        self.visible.set(true);
    }

    pub(crate) fn hide(self) {
        self.cancel_timer();
        self.visible.set(false);
    }

    pub(crate) fn bubble(
        self,
        label: Signal<String>,
        anchor: NodeRef<Span>,
        align: TooltipAlign,
    ) -> AnyView {
        let visible = self.visible;
        view! { {move || visible.get().then(|| bubble_view(label, anchor, align))} }.into_any()
    }
}

fn bubble_view(label: Signal<String>, anchor: NodeRef<Span>, align: TooltipAlign) -> AnyView {
    let panel = AnchoredPanel::new();
    panel.follow(
        Signal::stored(true),
        move || anchor.get_untracked().map(Into::into),
        align.anchoring(),
    );
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

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Tooltip(
    #[prop(into)] label: String,
    #[prop(optional)] align: TooltipAlign,
    #[prop(optional, into)] id: Option<String>,
    children: Children,
) -> impl IntoView {
    let control = TooltipControl::new();
    let anchor: NodeRef<Span> = NodeRef::new();
    let description = id.map(|id| {
        view! {
            <span id=id class="sr-only">
                {label.clone()}
            </span>
        }
    });
    view! {
        <span
            node_ref=anchor
            class="relative inline-flex"
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
            {control.bubble(Signal::stored(label), anchor, align)}
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
