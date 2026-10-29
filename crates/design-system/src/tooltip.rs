use leptos::prelude::*;

const TOOLTIP_CLASS: &str = "pointer-events-none absolute bottom-full z-50 mb-1.5 scale-95 whitespace-nowrap rounded-md bg-gray-900 px-3 py-1.5 text-xs text-white opacity-0 shadow-md transition delay-0 duration-100 group-hover/tooltip:scale-100 group-hover/tooltip:opacity-100 group-hover/tooltip:delay-300 group-has-focus-visible/tooltip:scale-100 group-has-focus-visible/tooltip:opacity-100 group-has-focus-visible/tooltip:delay-0";

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
    label: &'static str,
    #[prop(optional)] align: TooltipAlign,
    children: Children,
) -> impl IntoView {
    view! {
        <span class="group/tooltip relative inline-flex">
            {children()} <span role="tooltip" class=format!("{TOOLTIP_CLASS} {}", align.class())>
                {label}
            </span>
        </span>
    }
    .into_any()
}
