use leptos::prelude::*;
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode as EncodedQrCode};

use crate::button::{button_class, button_icon_class, ButtonSize, ButtonTone};
use crate::clipboard::CopyButton;
use crate::dialog::{Dialog, DialogSize};
use crate::icons::{self, LucideGlyph};

const DARK_MODULE_COLOR: &str = "#111827";
const LIGHT_MODULE_COLOR: &str = "#ffffff";
const SVG_START: &str = "<svg";
const SCALABLE_CLASS: &str = "[&>svg]:block [&>svg]:h-auto [&>svg]:w-full";
const TOO_LONG_MESSAGE: &str = "This link is too long to fit in a QR code.";
const DEFAULT_BUTTON_LABEL: &str = "Show QR Code";

#[must_use]
pub fn qr_code_svg(text: &str) -> Option<String> {
    let encoded = EncodedQrCode::with_error_correction_level(text.as_bytes(), EcLevel::M).ok()?;
    let rendered = encoded
        .render()
        .quiet_zone(true)
        .dark_color(svg::Color(DARK_MODULE_COLOR))
        .light_color(svg::Color(LIGHT_MODULE_COLOR))
        .build();
    let start = rendered.find(SVG_START)?;
    rendered.get(start..).map(str::to_owned)
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn QrCode(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] label: String,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let drawing = Memo::new(move |_| qr_code_svg(&value.get()));
    let class = format!("{SCALABLE_CLASS} {class}");
    move || drawing_view(drawing.get(), &label, &class)
}

fn drawing_view(drawing: Option<String>, label: &str, class: &str) -> AnyView {
    let Some(markup) = drawing else {
        return view! { <p class="text-sm text-red-700">{TOO_LONG_MESSAGE}</p> }.into_any();
    };
    let drawing = view! {
        <div role="img" aria-label=label.to_owned() class=class.to_owned() inner_html=markup></div>
    };
    drawing.into_any()
}

#[component]
pub fn QrCodeButton(
    #[prop(into)] address: Signal<String>,
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(default = DEFAULT_BUTTON_LABEL)] label: &'static str,
    #[prop(optional)] tone: ButtonTone,
    #[prop(optional)] size: ButtonSize,
    #[prop(into, default = Signal::stored(false))] disabled: Signal<bool>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let close = Callback::new(move |()| open.set(false));
    let title = Signal::stored(title);
    let description = description.map(Signal::stored);
    let dialog = move || {
        open.get().then(|| {
            let contents = qr_dialog_contents(address, title);
            match description {
                Some(description) => view! {
                    <Dialog title description on_close=close size=DialogSize::Small>
                        {contents}
                    </Dialog>
                }
                .into_any(),
                None => view! {
                    <Dialog title on_close=close size=DialogSize::Small>
                        {contents}
                    </Dialog>
                }
                .into_any(),
            }
        })
    };
    view! {
        <button
            type="button"
            class=button_class(tone, size)
            prop:disabled=move || disabled.get()
            on:click=move |_| open.set(true)
        >
            {icons::icon(LucideGlyph::QrCode, button_icon_class(size))}
            {label}
        </button>
        {dialog}
    }
    .into_any()
}

fn qr_dialog_contents(address: Signal<String>, title: Signal<String>) -> AnyView {
    let label = format!("QR Code: {}", title.get_untracked());
    view! {
        <div class="flex flex-col items-center gap-4">
            <QrCode value=address label class="w-full max-w-64" />
            <div class="flex w-full items-center gap-2 rounded-md bg-gray-50 py-1.5 pl-3 pr-1.5">
                <span class="min-w-0 flex-1 break-all font-mono text-xs text-gray-600">
                    {move || address.get()}
                </span>
                <CopyButton
                    value=address
                    label="Copy Link"
                    confirmation="Link Copied"
                    glyph=LucideGlyph::Link
                    tone=ButtonTone::Ghost
                    size=ButtonSize::Dense
                />
            </div>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_draws_as_a_scalable_svg() {
        let drawing =
            qr_code_svg("https://hermes.chichek.co.uk/deliveries/DN-10482").unwrap_or_default();
        assert!(drawing.starts_with(SVG_START));
        assert!(drawing.contains("viewBox"));
        assert!(drawing.contains(DARK_MODULE_COLOR));
    }

    #[test]
    fn text_too_long_for_any_qr_code_draws_nothing() {
        assert_eq!(qr_code_svg(&"a".repeat(8000)), None);
    }
}
