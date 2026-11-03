use leptos::prelude::*;

use crate::button::{button_icon_class, icon_button_class, ButtonSize, ButtonTone};
use crate::icons;
use crate::tooltip::{Tooltip, TooltipAlign};

#[cfg(feature = "hydrate")]
const COPY_FAILED_TITLE: &str = "Copy Failed";
#[cfg(feature = "hydrate")]
const COPY_FAILED_DESCRIPTION: &str = "The browser did not allow access to the clipboard.";

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn CopyButton(
    #[prop(into)] value: Signal<String>,
    #[prop(default = "Copy")] label: &'static str,
    #[prop(default = "Copied")] confirmation: &'static str,
    #[prop(default = icons::LucideGlyph::Copy)] glyph: icons::LucideGlyph,
    #[prop(optional)] tone: ButtonTone,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] tooltip_align: TooltipAlign,
) -> impl IntoView {
    view! {
        <Tooltip label=label align=tooltip_align>
            <button
                type="button"
                class=icon_button_class(tone, size)
                aria-label=label
                on:click=move |_| copy_with_toast(&value.get_untracked(), confirmation)
            >
                {icons::icon(glyph, button_icon_class(size))}
            </button>
        </Tooltip>
    }
    .into_any()
}

#[cfg(feature = "hydrate")]
fn current_address() -> String {
    web_sys::window()
        .and_then(|window| window.location().href().ok())
        .unwrap_or_default()
}

#[cfg(not(feature = "hydrate"))]
fn current_address() -> String {
    String::new()
}

#[component]
pub fn CopyLinkButton() -> impl IntoView {
    view! {
        <CopyButton
            value=Signal::derive(current_address)
            glyph=icons::LucideGlyph::Link
            label="Copy Link"
            confirmation="Link Copied"
        />
    }
    .into_any()
}

#[cfg_attr(
    not(feature = "hydrate"),
    expect(unused_variables, reason = "only a browser has a clipboard to copy to")
)]
pub fn copy_with_toast(text: &str, confirmation: &'static str) {
    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let write =
            wasm_bindgen_futures::JsFuture::from(window.navigator().clipboard().write_text(text));
        leptos::task::spawn_local(async move {
            if write.await.is_ok() {
                crate::toast::notify_success(confirmation);
            } else {
                crate::toast::notify_failure(
                    COPY_FAILED_TITLE,
                    Some(COPY_FAILED_DESCRIPTION.to_string()),
                );
            }
        });
    }
}

pub(crate) fn copy_current_address_with_toast() {
    copy_with_toast(&current_address(), "Link Copied");
}
