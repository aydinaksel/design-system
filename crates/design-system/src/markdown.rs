use leptos::prelude::*;
use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag, TagEnd};

const MARKDOWN_CLASS: &str = "space-y-2 text-sm text-gray-800 [&_a]:text-blue-600 [&_a]:underline [&_blockquote]:border-l-2 [&_blockquote]:border-edge [&_blockquote]:pl-3 [&_blockquote]:text-gray-600 [&_code]:rounded [&_code]:bg-gray-100 [&_code]:px-1 [&_h1]:font-semibold [&_h2]:font-semibold [&_h3]:font-semibold [&_ol]:list-decimal [&_ol]:pl-5 [&_strong]:font-semibold [&_ul]:list-disc [&_ul]:pl-5";
const ARTICLE_CLASS: &str = "max-w-prose text-base leading-7 text-gray-800 [&>*+*]:mt-4 [&>h1]:text-2xl [&>h1]:font-bold [&>h1]:tracking-tight [&>h1]:text-gray-900 [&>h2]:mt-10 [&>h2]:text-lg [&>h2]:font-semibold [&>h2]:text-gray-900 [&>h3]:mt-8 [&>h3]:font-semibold [&>h3]:text-gray-900 [&_a]:text-blue-600 [&_a]:underline [&_strong]:font-semibold [&_strong]:text-gray-900 [&_ul]:list-disc [&_ul]:pl-6 [&_ol]:list-decimal [&_ol]:pl-6 [&_li]:mt-1 [&_li]:pl-1 [&_blockquote]:border-l-2 [&_blockquote]:border-edge [&_blockquote]:pl-4 [&_blockquote]:text-gray-600 [&_code]:rounded [&_code]:bg-gray-100 [&_code]:px-1 [&_code]:text-sm [&_pre]:overflow-x-auto [&_pre]:rounded-lg [&_pre]:border [&_pre]:border-edge [&_pre]:bg-gray-50 [&_pre]:p-4 [&_pre]:leading-6 [&_pre_code]:bg-transparent [&_pre_code]:p-0 [&_img]:mx-auto [&_img]:my-6 [&_img]:block [&_img]:h-auto [&_img]:max-w-full [&_img]:rounded-lg [&_img]:border [&_img]:border-edge [&_hr]:my-10 [&_hr]:border-edge";
const SAFE_LINK_SCHEMES: [&str; 3] = ["https:", "http:", "mailto:"];
const NEUTRAL_LINK: &str = "#";

fn is_safe_link(destination: &str) -> bool {
    let lowered = destination.trim().to_lowercase();
    let has_scheme = lowered
        .split_once(':')
        .is_some_and(|(scheme, _)| !scheme.contains(['/', '?', '#']));
    !has_scheme
        || SAFE_LINK_SCHEMES
            .iter()
            .any(|scheme| lowered.starts_with(scheme))
}

fn safe_event(event: Event<'_>) -> Event<'_> {
    match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) if !is_safe_link(&dest_url) => Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::Borrowed(NEUTRAL_LINK),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) if !is_safe_link(&dest_url) => Event::Start(Tag::Image {
            link_type,
            dest_url: CowStr::Borrowed(NEUTRAL_LINK),
            title,
            id,
        }),
        other => other,
    }
}

#[must_use]
pub fn markdown_html(text: &str) -> String {
    let parser = Parser::new_ext(text, Options::ENABLE_STRIKETHROUGH).map(safe_event);
    let mut rendered = String::new();
    html::push_html(&mut rendered, parser);
    rendered
}

const fn ends_a_block(tag: TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::Item
            | TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
    )
}

#[must_use]
pub fn markdown_plain_text(text: &str) -> String {
    let mut words = String::new();
    for event in Parser::new_ext(text, Options::ENABLE_STRIKETHROUGH) {
        match event {
            Event::Text(part) | Event::Code(part) => words.push_str(&part),
            Event::SoftBreak | Event::HardBreak => words.push(' '),
            Event::End(tag) if ends_a_block(tag) => words.push(' '),
            _ => {}
        }
    }
    words.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn MarkdownText(#[prop(into)] text: Signal<String>) -> impl IntoView {
    view! { <div class=MARKDOWN_CLASS inner_html=move || markdown_html(&text.get())></div> }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn MarkdownArticle(#[prop(into)] text: Signal<String>) -> impl IntoView {
    view! { <article class=ARTICLE_CLASS inner_html=move || markdown_html(&text.get())></article> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_everyday_formatting() {
        assert_eq!(
            markdown_html("Supplier confirmed **13 crates**."),
            "<p>Supplier confirmed <strong>13 crates</strong>.</p>\n"
        );
        assert!(markdown_html("- one\n- two").contains("<ul>"));
    }

    #[test]
    fn flattens_markdown_into_one_line_of_text() {
        assert_eq!(
            markdown_plain_text(
                "Supplier confirmed **13 crates**.\n\n- Updated line B\n- Told `Tom`"
            ),
            "Supplier confirmed 13 crates. Updated line B Told Tom"
        );
    }

    #[test]
    fn shows_raw_html_as_text() {
        let rendered = markdown_html("<script>alert(1)</script>");
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("&lt;script&gt;"));
    }

    #[test]
    fn neutralises_links_that_could_run_code() {
        assert!(markdown_html("[click](javascript:alert(1))").contains("href=\"#\""));
        assert!(markdown_html("[click]( JavaScript:alert(1))").contains("href=\"#\""));
        assert!(
            markdown_html("[site](https://example.com)").contains("href=\"https://example.com\"")
        );
        assert!(markdown_html("[page](/quotes/4)").contains("href=\"/quotes/4\""));
    }
}
