use leptos::html::Div;
use leptos::prelude::*;

use crate::element_key::ElementKey;

const OPEN_SELECTOR: &str = ":popover-open";
pub(crate) const PANEL_MOTION_CLASS: &str = "opacity-0 open:opacity-100 starting:open:opacity-0 -translate-y-1 open:translate-y-0 starting:open:-translate-y-1 transition-[opacity,translate,display,overlay] transition-discrete duration-150 ease-out motion-reduce:transition-none";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelAlignment {
    Start,
    Center,
    End,
}

impl PanelAlignment {
    pub(crate) const fn attribute(self) -> &'static str {
        match self {
            Self::Start => "below-start",
            Self::Center => "below-center",
            Self::End => "below-end",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LightDismiss {
    ClosesAndBlocksThePage,
    Off,
}

impl LightDismiss {
    pub(crate) const fn popover_attribute(self) -> &'static str {
        match self {
            Self::ClosesAndBlocksThePage => "auto",
            Self::Off => "manual",
        }
    }

    pub(crate) fn blocks_page_attribute(self) -> Option<&'static str> {
        (self == Self::ClosesAndBlocksThePage).then_some("")
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NativePopover {
    key: ElementKey,
    pub(crate) open: RwSignal<bool>,
    pub(crate) panel: NodeRef<Div>,
}

impl NativePopover {
    pub(crate) fn new(open: RwSignal<bool>) -> Self {
        let popover = Self {
            key: ElementKey::next(),
            open,
            panel: NodeRef::new(),
        };
        popover.keep_in_step();
        popover
    }

    fn anchor_name(self) -> String {
        format!("--popover-anchor-{}", self.key)
    }

    pub(crate) fn trigger_style(self) -> String {
        format!("anchor-name: {}", self.anchor_name())
    }

    pub(crate) fn panel_style(self, gap_pixels: u32) -> String {
        format!(
            "position-anchor: {}; --popover-gap: {gap_pixels}px",
            self.anchor_name()
        )
    }

    fn keep_in_step(self) {
        Effect::new(move |_| {
            let open = self.open.get();
            let Some(panel) = self.panel.get() else {
                return;
            };
            let showing = panel.matches(OPEN_SELECTOR).unwrap_or(false);
            if open && !showing {
                panel.show_popover().unwrap_or_default();
            } else if !open && showing {
                panel.hide_popover().unwrap_or_default();
            }
        });
    }

    pub(crate) fn closed_by_the_browser(self) -> bool {
        let showing = self
            .panel
            .get_untracked()
            .is_some_and(|panel| panel.matches(OPEN_SELECTOR).unwrap_or(false));
        let closed = !showing && self.open.get_untracked();
        if closed {
            self.open.set(false);
        }
        closed
    }
}
