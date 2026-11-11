use std::sync::Arc;

use leptos::html::Div;
use leptos::portal::Portal;
use leptos::prelude::*;

use crate::placement::{Alignment, AnchoredPanel, Anchoring, PreferredSide};

const CARD_CLASS: &str = "pointer-events-none fixed z-[70] min-w-44 space-y-1.5 rounded-item border border-gray-200 bg-surface px-3 py-2 text-xs shadow-lg";
const GAP_PIXELS: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HoverSide {
    Beside,
    Above,
}

impl HoverSide {
    fn anchoring(self) -> Anchoring {
        let (alignment, side) = match self {
            Self::Beside => (Alignment::Center, PreferredSide::Beside),
            Self::Above => (Alignment::Start, PreferredSide::Above),
        };
        Anchoring {
            alignment,
            side,
            gap_pixels: GAP_PIXELS,
            ..Anchoring::default()
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HoverCard {
    pub(crate) anchor: NodeRef<Div>,
    open: RwSignal<bool>,
}

impl HoverCard {
    pub(crate) fn new() -> Self {
        Self {
            anchor: NodeRef::new(),
            open: RwSignal::new(false),
        }
    }

    pub(crate) fn show(self) {
        self.open.set(true);
    }

    pub(crate) fn hide(self) {
        self.open.set(false);
    }

    pub(crate) fn card(
        self,
        side: HoverSide,
        content: impl Fn() -> AnyView + Send + Sync + 'static,
    ) -> AnyView {
        let content: CardContent = Arc::new(content);
        view! {
            {move || {
                self.open.get().then(|| placed_card(self.anchor, side, Arc::clone(&content)))
            }}
        }
        .into_any()
    }
}

type CardContent = Arc<dyn Fn() -> AnyView + Send + Sync>;

fn placed_card(anchor: NodeRef<Div>, side: HoverSide, content: CardContent) -> AnyView {
    let panel = AnchoredPanel::new();
    panel.follow(
        Signal::stored(true),
        move || anchor.get_untracked().map(Into::into),
        side.anchoring(),
    );
    view! {
        <Portal>
            <div
                node_ref=panel.node
                aria-hidden="true"
                class=CARD_CLASS
                style=move || panel.style()
            >
                {
                    let content = Arc::clone(&content);
                    move || content()
                }
            </div>
        </Portal>
    }
    .into_any()
}
