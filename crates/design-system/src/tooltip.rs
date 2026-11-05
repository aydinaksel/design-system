use leptos::prelude::*;

use crate::element_key::ElementKey;

const TOOLTIP_CLASS: &str = "pointer-events-none absolute bottom-full z-50 mb-1.5 w-max max-w-xs scale-95 whitespace-normal rounded-md bg-gray-900 px-3 py-1.5 text-xs text-white opacity-0 shadow-md transition delay-0 duration-100 group-hover/tooltip:scale-100 group-hover/tooltip:opacity-100 group-hover/tooltip:delay-300 group-has-focus-visible/tooltip:scale-100 group-has-focus-visible/tooltip:opacity-100 group-has-focus-visible/tooltip:delay-0";
const HELP_TERM_CLASS: &str = "cursor-help rounded-sm underline decoration-gray-300 decoration-dotted underline-offset-4 focus-ring";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipAlign {
    #[default]
    Center,
    End,
}

impl TooltipAlign {
    const fn class(self) -> &'static str {
        match self {
            Self::Center => "left-1/2 -translate-x-1/2",
            Self::End => "right-0 origin-bottom-right",
        }
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Tooltip(
    #[prop(into)] label: String,
    #[prop(optional)] align: TooltipAlign,
    #[prop(optional, into)] id: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span class="group/tooltip relative inline-flex">
            {children()}
            <span id=id role="tooltip" class=format!("{TOOLTIP_CLASS} {}", align.class())>
                {label}
            </span>
        </span>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn HelpTerm(
    #[prop(into)] help: String,
    #[prop(optional, into)] help_id: Option<String>,
    children: Children,
) -> impl IntoView {
    let help_id = help_id.unwrap_or_else(|| format!("help-term-{}", ElementKey::next()));
    view! {
        <Tooltip label=help id=help_id.clone()>
            <span tabindex="0" aria-describedby=help_id class=HELP_TERM_CLASS>
                {children()}
            </span>
        </Tooltip>
    }
    .into_any()
}
