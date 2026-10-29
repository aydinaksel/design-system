use leptos::prelude::*;
use leptos_use::{use_clipboard, UseClipboardReturn};

use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::icons;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn CopyIconButton(#[prop(into)] value: Signal<String>) -> impl IntoView {
    let UseClipboardReturn { copied, copy, .. } = use_clipboard();

    view! {
        <button
            type="button"
            class="shrink-0 cursor-pointer hover:opacity-70"
            title="Copy"
            on:click={
                let copy = copy.clone();
                move |_| copy(&value.get_untracked())
            }
        >
            {move || {
                if copied.get() {
                    icons::labelled_icon(
                        icons::LucideGlyph::Check,
                        "block h-3.5 w-3.5 text-green-600",
                        "Copied",
                    )
                } else {
                    icons::icon(icons::LucideGlyph::Copy, "block h-3.5 w-3.5")
                }
            }}
        </button>
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

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn CopyButton(
    #[prop(into)] value: Signal<String>,
    glyph: icons::LucideGlyph,
    label: &'static str,
    copied_label: &'static str,
) -> impl IntoView {
    let UseClipboardReturn { copied, copy, .. } = use_clipboard();
    let current_label = move || if copied.get() { copied_label } else { label };

    view! {
        <button
            type="button"
            class=icon_button_class(ButtonTone::Secondary, ButtonSize::Standard)
            title=current_label
            aria-label=current_label
            on:click=move |_| copy(&value.get_untracked())
        >
            {move || {
                if copied.get() {
                    icons::icon(icons::LucideGlyph::Check, "h-4 w-4 text-green-600")
                } else {
                    icons::icon(glyph, "h-4 w-4 opacity-70")
                }
            }}
        </button>
    }
    .into_any()
}

#[component]
pub fn CopyLinkButton() -> impl IntoView {
    view! {
        <CopyButton
            value=Signal::derive(current_address)
            glyph=icons::LucideGlyph::Link
            label="Copy Link"
            copied_label="Link Copied"
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
        drop(window.navigator().clipboard().write_text(text));
        crate::toast::notify_success(confirmation);
    }
}

pub(crate) fn copy_current_address_with_toast() {
    copy_with_toast(&current_address(), "Link Copied");
}
