use leptos::prelude::*;
use leptos::web_sys::HtmlElement;

const SLIDE_TRANSITION: &str = "transform 250ms cubic-bezier(0.4, 0, 0.2, 1)";
const REDUCED_MOTION_QUERY: &str = "(prefers-reduced-motion: reduce)";

fn prefers_reduced_motion() -> bool {
    window()
        .match_media(REDUCED_MOTION_QUERY)
        .ok()
        .flatten()
        .is_some_and(|query| query.matches())
}

fn set_style(element: &HtmlElement, property: &str, value: &str) {
    element
        .style()
        .set_property(property, value)
        .unwrap_or_default();
}

fn commit_styles_before_transition(element: &HtmlElement) {
    element.offset_width();
}

#[derive(Clone, Copy)]
pub struct HorizontalExtent {
    left_pixels: f64,
    width_pixels: f64,
}

#[must_use]
pub fn horizontal_extent(element: &HtmlElement) -> HorizontalExtent {
    let rectangle = element.get_bounding_client_rect();
    HorizontalExtent {
        left_pixels: rectangle.left(),
        width_pixels: rectangle.width(),
    }
}

/// FLIP: <https://aerotwist.com/blog/flip-your-animations/>
pub fn slide_from(first: HorizontalExtent, next: &HtmlElement) {
    if prefers_reduced_motion() {
        return;
    }
    let last = horizontal_extent(next);
    if last.width_pixels <= 0.0 {
        return;
    }
    let offset_pixels = first.left_pixels - last.left_pixels;
    let width_ratio = first.width_pixels / last.width_pixels;
    set_style(next, "transition", "none");
    set_style(next, "transform-origin", "left");
    set_style(
        next,
        "transform",
        &format!("translateX({offset_pixels}px) scaleX({width_ratio})"),
    );
    commit_styles_before_transition(next);
    set_style(next, "transition", SLIDE_TRANSITION);
    set_style(next, "transform", "none");
}

pub fn slide_between(previous: &HtmlElement, next: &HtmlElement) {
    slide_from(horizontal_extent(previous), next);
}
