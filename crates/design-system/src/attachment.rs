use leptos::prelude::*;

use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::icons::{self, LucideGlyph};
use crate::tone::Tone;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentState {
    Idle,
    Uploading,
    Processing,
    Failed,
    #[default]
    Done,
}

impl AttachmentState {
    fn in_progress(self) -> bool {
        matches!(self, Self::Uploading | Self::Processing)
    }

    fn tone(self) -> Tone {
        match self {
            Self::Failed => Tone::Critical,
            Self::Idle | Self::Uploading | Self::Processing | Self::Done => Tone::Neutral,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentSize {
    #[default]
    Standard,
    Compact,
    Dense,
}

impl AttachmentSize {
    fn padding_class(self) -> &'static str {
        match self {
            Self::Standard => "gap-3 p-2.5 text-sm",
            Self::Compact => "gap-2.5 p-2 text-sm",
            Self::Dense => "gap-2 p-1.5 text-xs",
        }
    }

    fn media_class(self) -> &'static str {
        match self {
            Self::Standard => "size-10",
            Self::Compact => "size-8",
            Self::Dense => "size-6",
        }
    }

    fn icon_class(self) -> &'static str {
        match self {
            Self::Standard => "h-5 w-5",
            Self::Compact => "h-4 w-4",
            Self::Dense => "h-3.5 w-3.5",
        }
    }

    fn action_size(self) -> ButtonSize {
        match self {
            Self::Standard | Self::Compact => ButtonSize::Compact,
            Self::Dense => ButtonSize::Dense,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttachmentMedia {
    Icon(LucideGlyph),
    Image {
        source: String,
        alternative_text: String,
    },
}

#[derive(Clone)]
pub struct AttachmentAction {
    label: String,
    glyph: LucideGlyph,
    on_click: Callback<()>,
    danger: bool,
}

impl AttachmentAction {
    pub fn new(
        label: impl Into<String>,
        glyph: LucideGlyph,
        on_click: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            glyph,
            on_click: Callback::new(move |()| on_click()),
            danger: false,
        }
    }

    #[must_use]
    pub fn danger(self) -> Self {
        Self {
            danger: true,
            ..self
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttachmentOpen {
    Link(String),
    NewTab(String),
}

fn root_class(
    size: AttachmentSize,
    orientation: AttachmentOrientation,
    state: AttachmentState,
) -> String {
    let layout = match orientation {
        AttachmentOrientation::Horizontal => "flex-row items-center",
        AttachmentOrientation::Vertical => "flex-col items-stretch",
    };
    let border = match state.tone() {
        Tone::Critical => "border-red-200 bg-red-50/40",
        Tone::Neutral | Tone::Informative | Tone::Positive | Tone::Caution => {
            "border-gray-200 bg-white hover:border-gray-300"
        }
    };
    format!(
        "group/attachment relative flex min-w-0 rounded-lg border transition-colors {layout} {border} {}",
        size.padding_class(),
    )
}

fn media_view(
    media: AttachmentMedia,
    size: AttachmentSize,
    orientation: AttachmentOrientation,
    state: Signal<AttachmentState>,
) -> AnyView {
    let frame = match orientation {
        AttachmentOrientation::Horizontal => size.media_class(),
        AttachmentOrientation::Vertical => "aspect-video w-full",
    };
    match media {
        AttachmentMedia::Image {
            source,
            alternative_text,
        } => view! {
            <span class=format!("block shrink-0 overflow-hidden rounded-md bg-gray-100 {frame}")>
                <img src=source alt=alternative_text class="h-full w-full object-cover" />
            </span>
        }
        .into_any(),
        AttachmentMedia::Icon(glyph) => {
            let icon_class = move || {
                if state.get().in_progress() {
                    format!("animate-spin {}", size.icon_class())
                } else {
                    size.icon_class().to_owned()
                }
            };
            let shown_glyph = move || {
                if state.get().in_progress() {
                    LucideGlyph::LoaderCircle
                } else {
                    glyph
                }
            };
            view! {
                <span
                    aria-hidden="true"
                    class=move || {
                        format!(
                            "flex shrink-0 items-center justify-center rounded-md bg-gray-100 {frame} {}",
                            state.get().tone().icon_class(),
                        )
                    }
                >
                    {move || icons::icon(shown_glyph(), icon_class())}
                </span>
            }
            .into_any()
        }
    }
}

fn open_view(open: Option<AttachmentOpen>, title: &str) -> Option<AnyView> {
    let label = format!("Open {title}");
    let overlay_class = "absolute inset-0 z-0 rounded-lg focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500";
    open.map(|open| match open {
        AttachmentOpen::Link(href) => {
            view! { <a href=href aria-label=label class=overlay_class></a> }.into_any()
        }
        AttachmentOpen::NewTab(href) => view! { <a href=href target="_blank" rel="noopener" aria-label=label class=overlay_class></a> }
        .into_any(),
    })
}

fn actions_view(actions: Vec<AttachmentAction>, size: AttachmentSize) -> Option<AnyView> {
    if actions.is_empty() {
        return None;
    }
    let buttons = actions
        .into_iter()
        .map(|action| {
            let tone = if action.danger {
                ButtonTone::DangerOutline
            } else {
                ButtonTone::Ghost
            };
            let on_click = action.on_click;
            view! {
                <button
                    type="button"
                    class=icon_button_class(tone, size.action_size())
                    aria-label=action.label.clone()
                    title=action.label
                    on:click=move |_| on_click.run(())
                >
                    {icons::icon(action.glyph, "h-4 w-4")}
                </button>
            }
        })
        .collect_view();
    Some(
        view! { <div class="relative z-10 flex shrink-0 items-center gap-1">{buttons}</div> }
            .into_any(),
    )
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Attachment(
    #[prop(into)] title: String,
    media: AttachmentMedia,
    #[prop(optional, into)] description: MaybeProp<String>,
    #[prop(into, default = Signal::stored(AttachmentState::Done))] state: Signal<AttachmentState>,
    #[prop(optional)] size: AttachmentSize,
    #[prop(optional)] orientation: AttachmentOrientation,
    #[prop(optional)] open: Option<AttachmentOpen>,
    #[prop(optional)] actions: Vec<AttachmentAction>,
) -> impl IntoView {
    let open_overlay = open_view(open, &title);
    let title_text = title.clone();
    let actions = actions_view(actions, size);
    let title_class = move || {
        if state.get().in_progress() {
            "block truncate font-medium text-shimmer motion-safe:animate-shimmer"
        } else {
            "block truncate font-medium text-gray-900"
        }
    };
    let description_class = move || {
        format!(
            "block truncate text-xs {}",
            match state.get().tone() {
                Tone::Critical => Tone::Critical.text_class(),
                Tone::Neutral | Tone::Informative | Tone::Positive | Tone::Caution => {
                    "text-gray-500"
                }
            },
        )
    };
    view! {
        <div
            class=move || root_class(size, orientation, state.get())
            aria-busy=move || state.get().in_progress().to_string()
        >
            {open_overlay}
            {media_view(media, size, orientation, state)}
            <div class="pointer-events-none relative min-w-0 flex-1">
                <span class=title_class title=title_text>
                    {title}
                </span>
                {move || {
                    description
                        .get()
                        .map(|text| view! { <span class=description_class>{text}</span> })
                }}
            </div>
            {actions}
        </div>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn AttachmentGroup(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div
            role="group"
            aria-label=label
            class="flex snap-x snap-mandatory gap-2 overflow-x-auto pb-1 [mask-image:linear-gradient(to_right,transparent,black_12px,black_calc(100%-12px),transparent)] *:w-64 *:shrink-0 *:snap-start"
        >
            {children()}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_uploading_and_processing_count_as_in_progress() {
        assert!(AttachmentState::Uploading.in_progress());
        assert!(AttachmentState::Processing.in_progress());
        assert!(!AttachmentState::Idle.in_progress());
        assert!(!AttachmentState::Failed.in_progress());
        assert!(!AttachmentState::Done.in_progress());
    }

    #[test]
    fn a_failure_reads_as_critical() {
        assert_eq!(AttachmentState::Failed.tone(), Tone::Critical);
        assert_eq!(AttachmentState::Done.tone(), Tone::Neutral);
    }
}
