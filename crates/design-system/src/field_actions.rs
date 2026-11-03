use leptos::prelude::*;

use crate::button::{button_icon_class, icon_button_class, ButtonSize, ButtonTone};
use crate::clipboard::CopyButton;
use crate::icons;
use crate::tooltip::{Tooltip, TooltipAlign};

const ACTION_TONE: ButtonTone = ButtonTone::Ghost;
const ACTION_SIZE: ButtonSize = ButtonSize::Compact;

enum FieldActionTarget {
    Link(String),
    Copy {
        text: String,
        confirmation: &'static str,
    },
}

pub struct FieldAction {
    label: &'static str,
    glyph: icons::LucideGlyph,
    target: FieldActionTarget,
}

impl FieldAction {
    #[must_use]
    pub fn link(label: &'static str, glyph: icons::LucideGlyph, href: impl Into<String>) -> Self {
        Self {
            label,
            glyph,
            target: FieldActionTarget::Link(href.into()),
        }
    }

    #[must_use]
    pub fn copy(label: &'static str, text: impl Into<String>, confirmation: &'static str) -> Self {
        Self {
            label,
            glyph: icons::LucideGlyph::Copy,
            target: FieldActionTarget::Copy {
                text: text.into(),
                confirmation,
            },
        }
    }

    fn render(self, align: TooltipAlign) -> AnyView {
        let Self {
            label,
            glyph,
            target,
        } = self;
        match target {
            FieldActionTarget::Link(href) => view! {
                <Tooltip label=label align=align>
                    <a
                        href=href
                        target="_blank"
                        rel="noreferrer"
                        class=icon_button_class(ACTION_TONE, ACTION_SIZE)
                        aria-label=label
                    >
                        {icons::icon(glyph, button_icon_class(ACTION_SIZE))}
                    </a>
                </Tooltip>
            }
            .into_any(),
            FieldActionTarget::Copy { text, confirmation } => view! {
                <CopyButton
                    value=text
                    label=label
                    confirmation=confirmation
                    glyph=glyph
                    tone=ACTION_TONE
                    size=ACTION_SIZE
                    tooltip_align=align
                />
            }
            .into_any(),
        }
    }
}

pub(crate) fn field_actions_view(actions: Vec<FieldAction>) -> Option<AnyView> {
    let last = actions.len().checked_sub(1)?;
    let rendered = actions
        .into_iter()
        .enumerate()
        .map(|(position, action)| {
            let align = if position == last {
                TooltipAlign::End
            } else {
                TooltipAlign::Center
            };
            action.render(align)
        })
        .collect::<Vec<_>>();
    Some(view! { <div class="flex shrink-0 items-center gap-0.5">{rendered}</div> }.into_any())
}
