use leptos::prelude::*;

use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::clipboard::copy_with_toast;
use crate::icons;
use crate::tooltip::{Tooltip, TooltipAlign};

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
        let class = icon_button_class(ButtonTone::Ghost, ButtonSize::Compact);
        let glyph_view = icons::icon(glyph, "h-4 w-4 text-gray-500");
        let control = match target {
            FieldActionTarget::Link(href) => view! {
                <a href=href target="_blank" rel="noreferrer" class=class aria-label=label>
                    {glyph_view}
                </a>
            }
            .into_any(),
            FieldActionTarget::Copy { text, confirmation } => view! {
                <button
                    type="button"
                    class=class
                    aria-label=label
                    on:click=move |_| copy_with_toast(&text, confirmation)
                >
                    {glyph_view}
                </button>
            }
            .into_any(),
        };
        view! {
            <Tooltip label=label align=align>
                {control}
            </Tooltip>
        }
        .into_any()
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
    Some(
        view! { <div class="-my-2 flex shrink-0 items-center gap-0.5">{rendered}</div> }.into_any(),
    )
}
