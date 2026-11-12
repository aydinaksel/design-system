use leptos::prelude::*;

use crate::element_key::ElementKey;
use crate::typography::{heading_element, HeadingLevel, LABEL_CLASS, SECTION_HEADING_ROW_CLASS};

/// A titled part of a page that is not a `FieldSection`, such as a table or a
/// set of files, with the same heading as one so the two sit together.
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn PageSection(
    #[prop(into)] title: String,
    #[prop(optional)] heading_level: HeadingLevel,
    #[prop(optional)] actions: Option<AnyView>,
    children: Children,
) -> impl IntoView {
    let heading_id = format!("page-section-{}-heading", ElementKey::next());
    let heading = heading_element(heading_level, Some(heading_id.clone()), LABEL_CLASS, title);
    view! {
        <section aria-labelledby=heading_id class="w-full">
            <div class=SECTION_HEADING_ROW_CLASS>
                {heading} <div class="ml-auto flex items-center gap-2">{actions}</div>
            </div>
            {children()}
        </section>
    }
    .into_any()
}
