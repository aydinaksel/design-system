use leptos::prelude::*;

const BUTTON_GROUP_CLASS: &str = "inline-flex flex-wrap items-stretch [&>*]:rounded-none [&>*:first-child]:rounded-l-md [&>*:last-child]:rounded-r-md [&>*:not(:first-child)]:-ml-px [&>:is(a,button):hover]:z-10 [&>:is(a,button):focus-visible]:z-10";

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ButtonGroup(children: Children) -> impl IntoView {
    view! {
        <div role="group" class=BUTTON_GROUP_CLASS>
            {children()}
        </div>
    }
    .into_any()
}
