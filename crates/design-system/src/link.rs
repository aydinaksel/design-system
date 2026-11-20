use leptos::prelude::*;

pub const LINK_CLASS: &str = "text-blue-600 hover:text-blue-800 hover:underline";

/// A text link in the running text of a page, a field or a table cell.
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Link(
    #[prop(into)] href: Signal<String>,
    #[prop(optional)] opens_new_tab: bool,
    #[prop(optional)] downloads: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            href=move || href.get()
            class=LINK_CLASS
            target=opens_new_tab.then_some("_blank")
            rel=opens_new_tab.then_some("noopener noreferrer")
            download=downloads.then_some("")
        >
            {children()}
        </a>
    }
    .into_any()
}
