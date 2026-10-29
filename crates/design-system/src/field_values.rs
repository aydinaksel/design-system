use leptos::prelude::*;

use crate::field_actions::FieldAction;
use crate::icons;

fn present(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

#[must_use]
pub fn missing_value() -> AnyView {
    view! { <span class="text-gray-300">"\u{2014}"</span> }.into_any()
}

#[must_use]
pub fn value_skeleton() -> AnyView {
    view! {
        <span
            aria-hidden="true"
            class="inline-block h-3.5 w-40 max-w-full animate-pulse rounded bg-gray-100 align-middle"
        ></span>
    }
    .into_any()
}

#[must_use]
pub fn text_value(value: Option<String>) -> AnyView {
    match present(value) {
        Some(text) => view! { <span>{text}</span> }.into_any(),
        None => missing_value(),
    }
}

#[must_use]
pub fn multiline_value(value: Option<String>) -> AnyView {
    match present(value) {
        Some(text) => view! { <span class="whitespace-pre-wrap">{text}</span> }.into_any(),
        None => missing_value(),
    }
}

#[must_use]
pub fn partner_value(code: Option<String>, name: Option<String>, href: Option<String>) -> AnyView {
    let (code, name) = (present(code), present(name));
    if code.is_none() && name.is_none() {
        return missing_value();
    }
    let separator = (code.is_some() && name.is_some()).then_some(" \u{b7} ");
    let line = view! {
        <span class="text-gray-500">{code}</span>
        {separator}
        <span>{name}</span>
    };
    match href {
        Some(href) => view! {
            <a href=href class="text-blue-600 hover:text-blue-800 hover:underline">
                {line}
            </a>
        }
        .into_any(),
        None => line.into_any(),
    }
}

#[must_use]
pub fn address_value(value: Option<String>) -> AnyView {
    let lines = address_lines(value.unwrap_or_default().as_str());
    if lines.is_empty() {
        return missing_value();
    }
    view! { <span class="whitespace-pre-line">{lines.join("\n")}</span> }.into_any()
}

#[must_use]
pub fn address_actions(value: Option<&str>) -> Vec<FieldAction> {
    let lines = address_lines(value.unwrap_or_default());
    if lines.is_empty() {
        return Vec::new();
    }
    vec![
        FieldAction::link(
            "Open in Google Maps",
            icons::LucideGlyph::MapPin,
            map_search_url(&lines),
        ),
        FieldAction::copy("Copy Address", lines.join("\n"), "Address Copied"),
    ]
}

fn address_lines(value: &str) -> Vec<String> {
    value
        .split(['\r', '\n'])
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

fn map_search_url(lines: &[String]) -> String {
    let query = lines.join(", ");
    let mut encoded = String::with_capacity(query.len());
    for byte in query.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                let high = usize::from(byte >> 4);
                let low = usize::from(byte & 0x0F);
                encoded.push('%');
                encoded.push(char::from(HEX_DIGITS.get(high).copied().unwrap_or(b'0')));
                encoded.push(char::from(HEX_DIGITS.get(low).copied().unwrap_or(b'0')));
            }
        }
    }
    format!("https://www.google.com/maps/search/?api=1&query={encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_address_lines_into_one_map_query() {
        let lines = vec!["1200 Harbor Way".to_owned(), "Long Beach, CA".to_owned()];
        assert_eq!(
            map_search_url(&lines),
            "https://www.google.com/maps/search/?api=1&query=1200%20Harbor%20Way%2C%20Long%20Beach%2C%20CA"
        );
    }
}
