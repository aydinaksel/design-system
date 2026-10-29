use leptos::prelude::*;
use leptos::web_sys::HtmlElement;

pub(crate) const PANEL_MAXIMUM_HEIGHT_PIXELS: f64 = 240.0;
pub(crate) const ANCHORED_PANEL_CLASS: &str = "fixed z-50 max-h-60 overflow-y-auto rounded-xl border border-gray-200 bg-white p-1 text-sm/6 shadow-lg focus:outline-none motion-safe:animate-fade-in";
const GAP_PIXELS: f64 = 4.0;
const VIEWPORT_MARGIN_PIXELS: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TriggerBounds {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Viewport {
    width: f64,
    height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VerticalEdge {
    FromTop(f64),
    FromBottom(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnchoredPosition {
    edge: VerticalEdge,
    left: f64,
    minimum_width: f64,
}

impl Default for AnchoredPosition {
    fn default() -> Self {
        Self {
            edge: VerticalEdge::FromTop(0.0),
            left: 0.0,
            minimum_width: 0.0,
        }
    }
}

impl AnchoredPosition {
    pub(crate) fn style(self) -> String {
        let vertical = match self.edge {
            VerticalEdge::FromTop(distance) => format!("top:{distance}px"),
            VerticalEdge::FromBottom(distance) => format!("bottom:{distance}px"),
        };
        format!(
            "{vertical};left:{}px;min-width:{}px",
            self.left, self.minimum_width
        )
    }
}

fn place(trigger: TriggerBounds, viewport: Viewport) -> AnchoredPosition {
    let panel_extent = GAP_PIXELS + PANEL_MAXIMUM_HEIGHT_PIXELS + VIEWPORT_MARGIN_PIXELS;
    let space_below = viewport.height - trigger.bottom;
    let space_above = trigger.top;
    let opens_above = space_below < panel_extent && space_above > space_below;
    let edge = if opens_above {
        VerticalEdge::FromBottom(viewport.height - trigger.top + GAP_PIXELS)
    } else {
        VerticalEdge::FromTop(trigger.bottom + GAP_PIXELS)
    };
    let width = trigger.right - trigger.left;
    let rightmost_left = viewport.width - width - VIEWPORT_MARGIN_PIXELS;
    AnchoredPosition {
        edge,
        left: trigger.left.min(rightmost_left).max(VIEWPORT_MARGIN_PIXELS),
        minimum_width: width,
    }
}

pub(crate) fn anchor_to(trigger: &HtmlElement) -> AnchoredPosition {
    let Some(root) = document().document_element() else {
        return AnchoredPosition::default();
    };
    let rectangle = trigger.get_bounding_client_rect();
    place(
        TriggerBounds {
            left: rectangle.left(),
            right: rectangle.right(),
            top: rectangle.top(),
            bottom: rectangle.bottom(),
        },
        Viewport {
            width: f64::from(root.client_width()),
            height: f64::from(root.client_height()),
        },
    )
}

pub(crate) fn follow_trigger(
    open: Signal<bool>,
    measure: impl Fn() -> Option<AnchoredPosition> + Clone + 'static,
    position: RwSignal<AnchoredPosition>,
) {
    let reposition = move || {
        if !open.get_untracked() {
            return;
        }
        if let Some(measured) = measure() {
            position.set(measured);
        }
    };
    let on_resize = reposition.clone();
    let scrolling = window_event_listener(leptos::ev::scroll, move |_| reposition());
    let resizing = window_event_listener(leptos::ev::resize, move |_| on_resize());
    on_cleanup(move || {
        scrolling.remove();
        resizing.remove();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Viewport = Viewport {
        width: 1000.0,
        height: 800.0,
    };

    fn trigger_at(left: f64, top: f64) -> TriggerBounds {
        TriggerBounds {
            left,
            right: left + 200.0,
            top,
            bottom: top + 32.0,
        }
    }

    #[test]
    fn opens_below_the_trigger_when_there_is_room() {
        let expected = AnchoredPosition {
            edge: VerticalEdge::FromTop(136.0),
            left: 100.0,
            minimum_width: 200.0,
        };
        assert_eq!(place(trigger_at(100.0, 100.0), VIEWPORT), expected);
    }

    #[test]
    fn opens_above_the_trigger_near_the_bottom_of_the_viewport() {
        let position = place(trigger_at(100.0, 700.0), VIEWPORT);
        assert_eq!(position.edge, VerticalEdge::FromBottom(104.0));
    }

    #[test]
    fn stays_below_when_there_is_more_room_below_than_above() {
        let short_viewport = Viewport {
            width: 1000.0,
            height: 200.0,
        };
        let position = place(trigger_at(100.0, 40.0), short_viewport);
        assert_eq!(position.edge, VerticalEdge::FromTop(76.0));
    }

    #[test]
    fn keeps_the_panel_inside_the_right_edge() {
        let expected = AnchoredPosition {
            edge: VerticalEdge::FromTop(136.0),
            left: 792.0,
            minimum_width: 200.0,
        };
        assert_eq!(place(trigger_at(900.0, 100.0), VIEWPORT), expected);
    }
}
