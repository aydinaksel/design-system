use leptos::html::Div;
use leptos::prelude::*;
use leptos::web_sys::Element;
use leptos_use::{
    use_event_listener, use_event_listener_with_options, use_window, UseEventListenerOptions,
};

const VIEWPORT_MARGIN_PIXELS: f64 = 8.0;
const UNPLACED_STYLE: &str = "left:0;top:0;opacity:0";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Alignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PreferredSide {
    #[default]
    Below,
    Above,
    Beside,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Anchoring {
    pub(crate) alignment: Alignment,
    pub(crate) side: PreferredSide,
    pub(crate) gap_pixels: f64,
    pub(crate) matches_trigger_width: bool,
    pub(crate) reserved_height_pixels: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bounds {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Bounds {
    fn of(element: &Element) -> Self {
        let rectangle = element.get_bounding_client_rect();
        Self {
            left: rectangle.left(),
            top: rectangle.top(),
            right: rectangle.right(),
            bottom: rectangle.bottom(),
        }
    }

    fn extent(self) -> Extent {
        Extent {
            width: self.right - self.left,
            height: self.bottom - self.top,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Extent {
    width: f64,
    height: f64,
}

fn viewport() -> Option<Extent> {
    let root = document().document_element()?;
    Some(Extent {
        width: f64::from(root.client_width()),
        height: f64::from(root.client_height()),
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VerticalEdge {
    Top(f64),
    Bottom(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Placement {
    edge: VerticalEdge,
    left: f64,
    minimum_width: Option<f64>,
}

impl Placement {
    fn style(self) -> String {
        let vertical = match self.edge {
            VerticalEdge::Top(distance) => format!("top:{distance}px"),
            VerticalEdge::Bottom(distance) => format!("bottom:{distance}px"),
        };
        let minimum_width = self
            .minimum_width
            .map(|width| format!(";min-width:{width}px"))
            .unwrap_or_default();
        format!("{vertical};left:{}px{minimum_width}", self.left)
    }
}

fn place_beside(trigger: Bounds, panel: Extent, viewport: Extent, gap: f64) -> Placement {
    let fits_right = trigger.right + gap + panel.width <= viewport.width - VIEWPORT_MARGIN_PIXELS;
    let left = if fits_right {
        trigger.right + gap
    } else {
        trigger.left - gap - panel.width
    };
    let centered_top = (trigger.top + trigger.bottom - panel.height) / 2.0;
    let lowest_top = viewport.height - VIEWPORT_MARGIN_PIXELS - panel.height;
    Placement {
        edge: VerticalEdge::Top(centered_top.min(lowest_top).max(VIEWPORT_MARGIN_PIXELS)),
        left: left.max(VIEWPORT_MARGIN_PIXELS),
        minimum_width: None,
    }
}

fn place(trigger: Bounds, panel: Extent, viewport: Extent, anchoring: Anchoring) -> Placement {
    if anchoring.side == PreferredSide::Beside {
        return place_beside(trigger, panel, viewport, anchoring.gap_pixels);
    }
    let trigger_width = trigger.right - trigger.left;
    let minimum_width = anchoring.matches_trigger_width.then_some(trigger_width);
    let width = panel.width.max(minimum_width.unwrap_or_default());
    let height = panel.height.max(anchoring.reserved_height_pixels);
    let gap = anchoring.gap_pixels;
    let fits_below = trigger.bottom + gap + height <= viewport.height - VIEWPORT_MARGIN_PIXELS;
    let fits_above = trigger.top - gap - height >= VIEWPORT_MARGIN_PIXELS;
    let more_room_above = trigger.top > viewport.height - trigger.bottom;
    let opens_above = match anchoring.side {
        PreferredSide::Below | PreferredSide::Beside => {
            !fits_below && (fits_above || more_room_above)
        }
        PreferredSide::Above => fits_above || (!fits_below && more_room_above),
    };
    let edge = if opens_above {
        VerticalEdge::Bottom(viewport.height - trigger.top + gap)
    } else {
        VerticalEdge::Top(trigger.bottom + gap)
    };
    let preferred_left = match anchoring.alignment {
        Alignment::Start => trigger.left,
        Alignment::Center => (trigger.left + trigger.right - width) / 2.0,
        Alignment::End => trigger.right - width,
    };
    let rightmost_left = viewport.width - VIEWPORT_MARGIN_PIXELS - width;
    Placement {
        edge,
        left: preferred_left
            .min(rightmost_left)
            .max(VIEWPORT_MARGIN_PIXELS),
        minimum_width,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AnchoredPanel {
    pub(crate) node: NodeRef<Div>,
    placement: RwSignal<Option<Placement>>,
}

impl AnchoredPanel {
    pub(crate) fn new() -> Self {
        Self {
            node: NodeRef::new(),
            placement: RwSignal::new(None),
        }
    }

    pub(crate) fn style(self) -> String {
        self.placement
            .get()
            .map_or_else(|| UNPLACED_STYLE.to_owned(), Placement::style)
    }

    pub(crate) fn unplace(self) {
        self.placement.set(None);
    }

    pub(crate) fn place_against(self, trigger: &Element, anchoring: Anchoring) {
        let Some(panel) = self.node.get_untracked() else {
            return;
        };
        let Some(viewport) = viewport() else {
            return;
        };
        if !panel.is_connected() {
            return;
        }
        self.placement.set(Some(place(
            Bounds::of(trigger),
            Bounds::of(&panel).extent(),
            viewport,
            anchoring,
        )));
    }

    pub(crate) fn follow(
        self,
        open: Signal<bool>,
        trigger: impl Fn() -> Option<Element> + Clone + 'static,
        anchoring: Anchoring,
    ) {
        let reposition = move || {
            if !open.get_untracked() {
                return;
            }
            if let Some(trigger) = trigger() {
                self.place_against(&trigger, anchoring);
            }
        };
        let on_mount = reposition.clone();
        Effect::new(move |_| {
            if self.node.get().is_some() {
                on_mount();
            }
        });
        let on_scroll = reposition.clone();
        drop(use_event_listener_with_options(
            use_window(),
            leptos::ev::scroll,
            move |_| on_scroll(),
            UseEventListenerOptions::default()
                .capture(true)
                .passive(true),
        ));
        drop(use_event_listener(
            use_window(),
            leptos::ev::resize,
            move |_| reposition(),
        ));
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
    const GAP_PIXELS: f64 = 6.0;

    fn anchoring(alignment: Alignment) -> Anchoring {
        Anchoring {
            alignment,
            gap_pixels: GAP_PIXELS,
            ..Anchoring::default()
        }
    }

    fn trigger_at(left: f64, top: f64) -> Bounds {
        Bounds {
            left,
            top,
            right: left + 120.0,
            bottom: top + 32.0,
        }
    }

    fn below_trigger_at(left: f64) -> Placement {
        Placement {
            edge: VerticalEdge::Top(138.0),
            left,
            minimum_width: None,
        }
    }

    #[test]
    fn opens_below_and_lines_up_with_the_start_of_the_trigger() {
        let placement = place(
            trigger_at(100.0, 100.0),
            PANEL,
            VIEWPORT,
            anchoring(Alignment::Start),
        );
        assert_eq!(
            placement,
            Placement {
                edge: VerticalEdge::Top(138.0),
                left: 100.0,
                minimum_width: None,
            }
        );
    }

    #[test]
    fn centres_on_the_trigger() {
        let placement = place(
            trigger_at(400.0, 100.0),
            PANEL,
            VIEWPORT,
            anchoring(Alignment::Center),
        );
        assert_eq!(placement, below_trigger_at(316.0));
    }

    #[test]
    fn lines_up_with_the_end_of_the_trigger() {
        let placement = place(
            trigger_at(400.0, 100.0),
            PANEL,
            VIEWPORT,
            anchoring(Alignment::End),
        );
        assert_eq!(placement, below_trigger_at(232.0));
    }

    #[test]
    fn flips_above_from_the_trigger_top_when_there_is_no_room_below() {
        let placement = place(
            trigger_at(100.0, 700.0),
            PANEL,
            VIEWPORT,
            anchoring(Alignment::Start),
        );
        assert_eq!(placement.edge, VerticalEdge::Bottom(106.0));
    }

    #[test]
    fn opens_above_when_asked_and_there_is_room() {
        let above = Anchoring {
            side: PreferredSide::Above,
            ..anchoring(Alignment::Start)
        };
        let placement = place(trigger_at(100.0, 400.0), PANEL, VIEWPORT, above);
        assert_eq!(placement.edge, VerticalEdge::Bottom(406.0));
    }

    #[test]
    fn falls_below_when_asked_for_above_without_room() {
        let above = Anchoring {
            side: PreferredSide::Above,
            ..anchoring(Alignment::Start)
        };
        let placement = place(trigger_at(100.0, 100.0), PANEL, VIEWPORT, above);
        assert_eq!(placement.edge, VerticalEdge::Top(138.0));
    }

    #[test]
    fn sits_beside_the_trigger_centred_on_it() {
        let beside = Anchoring {
            side: PreferredSide::Beside,
            ..anchoring(Alignment::Start)
        };
        let placement = place(trigger_at(100.0, 400.0), PANEL, VIEWPORT, beside);
        assert_eq!(
            placement,
            Placement {
                edge: VerticalEdge::Top(316.0),
                left: 226.0,
                minimum_width: None,
            }
        );
    }

    #[test]
    fn sits_on_the_left_when_the_right_has_no_room() {
        let beside = Anchoring {
            side: PreferredSide::Beside,
            ..anchoring(Alignment::Start)
        };
        let placement = place(trigger_at(800.0, 400.0), PANEL, VIEWPORT, beside);
        assert_eq!(
            placement,
            Placement {
                edge: VerticalEdge::Top(316.0),
                left: 506.0,
                minimum_width: None,
            }
        );
    }

    #[test]
    fn stays_inside_the_viewport_beside_a_trigger_near_the_bottom() {
        let beside = Anchoring {
            side: PreferredSide::Beside,
            ..anchoring(Alignment::Start)
        };
        let placement = place(trigger_at(100.0, 760.0), PANEL, VIEWPORT, beside);
        assert_eq!(placement.edge, VerticalEdge::Top(592.0));
    }

    #[test]
    fn takes_the_roomier_side_when_neither_fits() {
        let tall_panel = Extent {
            width: 288.0,
            height: 700.0,
        };
        let near_top = place(
            trigger_at(100.0, 300.0),
            tall_panel,
            VIEWPORT,
            anchoring(Alignment::Start),
        );
        let near_bottom = place(
            trigger_at(100.0, 500.0),
            tall_panel,
            VIEWPORT,
            anchoring(Alignment::Start),
        );
        assert_eq!(near_top.edge, VerticalEdge::Top(338.0));
        assert_eq!(near_bottom.edge, VerticalEdge::Bottom(306.0));
    }

    #[test]
    fn reserves_height_for_content_that_has_not_arrived() {
        let short_panel = Extent {
            width: 288.0,
            height: 40.0,
        };
        let reserving = Anchoring {
            reserved_height_pixels: 240.0,
            ..anchoring(Alignment::Start)
        };
        let placement = place(trigger_at(100.0, 600.0), short_panel, VIEWPORT, reserving);
        assert_eq!(placement.edge, VerticalEdge::Bottom(206.0));
    }

    #[test]
    fn stays_inside_the_right_edge_of_the_viewport() {
        let placement = place(
            trigger_at(900.0, 100.0),
            PANEL,
            VIEWPORT,
            anchoring(Alignment::Start),
        );
        assert_eq!(placement, below_trigger_at(704.0));
    }

    #[test]
    fn stays_inside_the_left_edge_of_the_viewport() {
        let placement = place(
            trigger_at(0.0, 100.0),
            PANEL,
            VIEWPORT,
            anchoring(Alignment::End),
        );
        assert_eq!(placement, below_trigger_at(VIEWPORT_MARGIN_PIXELS));
    }

    #[test]
    fn grows_to_the_trigger_width_and_keeps_that_width_on_screen() {
        let narrow_panel = Extent {
            width: 50.0,
            height: 200.0,
        };
        let matching = Anchoring {
            matches_trigger_width: true,
            ..anchoring(Alignment::Start)
        };
        let placement = place(trigger_at(900.0, 100.0), narrow_panel, VIEWPORT, matching);

        let expected = Placement {
            minimum_width: Some(120.0),
            ..below_trigger_at(872.0)
        };
        assert_eq!(placement, expected);
    }
}
