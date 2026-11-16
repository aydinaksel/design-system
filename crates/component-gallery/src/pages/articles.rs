use design_system::{MarkdownArticle, PageHeader};
use leptos::prelude::*;

use super::section;

pub const RECEIVE_SCREEN_PATH: &str = "/gallery-files/receive-screen.svg";
const SAMPLE_ARTICLE: &str = include_str!("../../assets/receiving-a-delivery.md");

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ArticlesPage() -> impl IntoView {
    view! {
        <main class="mx-auto max-w-screen-2xl space-y-10 px-6 py-8">
            <PageHeader
                title="Articles"
                description="Help docs and how-tos written in plain CommonMark."
            />
            {section(
                "Help Article",
                view! { <MarkdownArticle text=SAMPLE_ARTICLE.to_owned() /> }.into_any(),
            )}
        </main>
    }
    .into_any()
}
