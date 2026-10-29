use leptos::prelude::*;
use leptos_router::components::A;

const IMAGE_CLASS: &str = "aspect-square w-full bg-gray-100 object-cover";

/// The widths of one photograph a browser picks between.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponsiveSource {
    pub display_url: String,
    pub source_set: String,
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ResponsiveImage(
    source: Option<ResponsiveSource>,
    #[prop(into)] alternative_text: String,
    sizes: &'static str,
) -> impl IntoView {
    match source {
        Some(ResponsiveSource {
            display_url,
            source_set,
        }) => view! {
            <img
                class=IMAGE_CLASS
                src=display_url
                srcset=source_set
                sizes=sizes
                alt=alternative_text
                loading="lazy"
                decoding="async"
            />
        }
        .into_any(),
        None => view! { <div class=IMAGE_CLASS role="img" aria-label=alternative_text></div> }
            .into_any(),
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn CardGrid(#[prop(optional)] class: &'static str, children: Children) -> impl IntoView {
    view! {
        <ul class=format!(
            "grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 {class}",
        )>{children()}</ul>
    }
    .into_any()
}

/// One linked tile of a [`CardGrid`]: a square image over a padded body.
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Card(#[prop(into)] href: String, image: AnyView, children: Children) -> impl IntoView {
    view! {
        <li class="overflow-hidden rounded-lg border border-gray-200 bg-white transition hover:border-gray-300">
            <A href=href>{image} <div class="p-4">{children()}</div></A>
        </li>
    }
    .into_any()
}
