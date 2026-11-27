use design_system::{
    badge, Alert, ButtonSize, ButtonTone, Card, CardGrid, CopyButton, CopyLinkButton, Disclosure,
    GalleryImage, HelpTerm, ImageGallery, ImageGrid, Meter, QrCode, QrCodeButton, ResponsiveImage,
    Tone,
};
use leptos::prelude::*;

use crate::fake_data::swatch;

const BADGE_TONES: [(Tone, &str); 5] = [
    (Tone::Neutral, "Neutral"),
    (Tone::Positive, "Positive"),
    (Tone::Informative, "Informative"),
    (Tone::Caution, "Caution"),
    (Tone::Critical, "Critical"),
];

const SWATCHES: [(&str, &str, &str); 4] = [
    ("Ivory Honed Travertine", "TRVT-024", "e9e2d0"),
    ("Black Polished Granite", "GRNT-076", "3b3b3b"),
    ("Green Brushed Slate", "SLTE-112", "8a9a7b"),
    ("Beige Tumbled Limestone", "LMST-063", "d8c3a5"),
];

pub(crate) fn badges_and_hints() -> AnyView {
    let badges = BADGE_TONES
        .iter()
        .map(|(tone, label)| badge(*label, *tone))
        .collect_view();
    view! {
        <div class="flex flex-wrap items-center gap-4">
            {badges} <span class="text-sm text-gray-700">
                <HelpTerm help="A tooltip on hover">"Hover Me"</HelpTerm>
            </span>
            <span class="inline-flex items-center gap-1 text-sm text-gray-700">
                "TRVT-048"
                <CopyButton
                    value="TRVT-048".to_string()
                    tone=ButtonTone::Ghost
                    size=ButtonSize::Dense
                />
            </span> <span class="inline-flex items-center gap-2">
                <CopyButton
                    value="TRVT-048".to_string()
                    label="Copy SKU"
                    confirmation="SKU Copied"
                />
                <CopyLinkButton />
            </span>
        </div>
    }
    .into_any()
}

const ALERT_TONES: [(Tone, &str, &str); 4] = [
    (
        Tone::Informative,
        "Container Booked",
        "Departs Mersin on 14 November.",
    ),
    (
        Tone::Positive,
        "Invoice Committed",
        "Posted to QuickFile as INV-2291.",
    ),
    (
        Tone::Caution,
        "Possible Duplicate",
        "An invoice with this number was staged yesterday.",
    ),
    (Tone::Critical, "Upload Failed", "The file is not a PDF."),
];

pub(crate) fn alerts() -> AnyView {
    let alerts = ALERT_TONES
        .iter()
        .map(|(tone, title, body)| {
            view! {
                <Alert tone=*tone title=*title>
                    <p>{*body}</p>
                </Alert>
            }
        })
        .collect_view();
    view! { <div class="space-y-3">{alerts}</div> }.into_any()
}

const METER_READINGS: [(f64, Tone, &str); 3] = [
    (42.0, Tone::Informative, "42% of container capacity"),
    (91.0, Tone::Caution, "91% of container capacity"),
    (100.0, Tone::Critical, "112% of container capacity"),
];

pub(crate) fn meters() -> AnyView {
    let meters = METER_READINGS
        .iter()
        .map(|(value, tone, label)| {
            view! {
                <div class="space-y-1">
                    <p class="text-xs text-gray-500">{*label}</p>
                    <Meter value=*value label=*label tone=*tone />
                </div>
            }
        })
        .collect_view();
    view! { <div class="max-w-md space-y-4">{meters}</div> }.into_any()
}

pub(crate) fn disclosures() -> AnyView {
    view! {
        <div class="max-w-xl space-y-4 text-sm">
            <Disclosure
                summary=|| {
                    view! {
                        <span class="font-medium text-gray-900">
                            "Why Is the Arrival Date an Estimate?"
                        </span>
                    }
                }
                open_at_start=true
            >
                <p class="text-gray-600">
                    "The carrier confirms a berth only once the vessel is two days out, so the date moves with the sailing until then."
                </p>
            </Disclosure>
            <Disclosure summary=|| {
                view! {
                    <span class="font-medium text-gray-900">
                        "How Are Landed Costs Split Across Lines?"
                    </span>
                }
            }>
                <p class="text-gray-600">
                    "Freight and duty are shared by each line's value, and handling is shared by its weight."
                </p>
            </Disclosure>
        </div>
    }
    .into_any()
}

const DELIVERY_ADDRESS: &str = "https://hermes.chichek.co.uk/deliveries/DN-10482";

pub(crate) fn qr_codes() -> AnyView {
    view! {
        <div class="flex flex-wrap items-start gap-8">
            <QrCodeButton
                address=DELIVERY_ADDRESS.to_owned()
                title="Delivery DN-10482"
                description="Scan with a phone to add photos to this delivery."
            />
            <div class="w-32">
                <QrCode value=DELIVERY_ADDRESS.to_owned() label="QR Code: Delivery DN-10482" />
            </div>
        </div>
    }
    .into_any()
}

pub(crate) fn images() -> AnyView {
    let gallery_images: Vec<GalleryImage> = SWATCHES
        .iter()
        .map(|(name, _, color_hex)| {
            let source = swatch(color_hex);
            GalleryImage {
                label: (*name).to_string(),
                thumbnail_url: source.display_url.clone(),
                display_url: source.display_url,
                source_set: source.source_set,
            }
        })
        .collect();
    let cards = SWATCHES
        .iter()
        .map(|(name, code, color_hex)| {
            let image = view! { <ResponsiveImage source=Some(swatch(color_hex)) alternative_text=*name sizes="18rem" /> }
            .into_any();
            view! {
                <Card href=format!("#{code}") image=image>
                    <p class="text-sm font-medium text-gray-900">{*name}</p>
                    <p class="mt-0.5 font-mono text-xs text-gray-500">{*code}</p>
                </Card>
            }
        })
        .collect_view();
    let missing_image =
        view! { <ResponsiveImage source=None alternative_text="No photograph" sizes="18rem" /> }
            .into_any();
    view! {
        <ImageGrid images=gallery_images.clone() />
        <div class="mt-8 grid gap-8 lg:grid-cols-2">
            <ImageGallery images=Signal::stored(gallery_images) selected_image=RwSignal::new(0) />
            <CardGrid>
                {cards} <Card href="#missing" image=missing_image>
                    <p class="text-sm font-medium text-gray-900">"Without a Photograph"</p>
                </Card>
            </CardGrid>
        </div>
    }
    .into_any()
}
