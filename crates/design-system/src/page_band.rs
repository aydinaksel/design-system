use leptos::context::Provider;
use leptos::prelude::*;

use crate::layout::PAGE_WIDTH_CLASS;
use crate::tabs::TabsOnSharedBorder;

const BAND_CLASS: &str = "border-b border-edge";
const BAND_CONTENT_CLASS: &str = "pt-8";
const BODY_CLASS: &str = "py-6";

pub(crate) fn banded_page(header: AnyView, tabs: Option<ViewFn>, body: AnyView) -> AnyView {
    let tabs =
        tabs.map(|tabs| view! { <Provider value=TabsOnSharedBorder>{tabs.run()}</Provider> });
    view! {
        <div class=BAND_CLASS>
            <div class=format!("{PAGE_WIDTH_CLASS} {BAND_CONTENT_CLASS}")>{header} {tabs}</div>
        </div>
        <div class=format!("{PAGE_WIDTH_CLASS} {BODY_CLASS}")>{body}</div>
    }
    .into_any()
}
