use lepticons::{Icon, LucideGlyph};
use leptos::prelude::*;

const BOX_CLASS: &str = "peer size-4 cursor-pointer appearance-none rounded border border-control-edge bg-surface transition-colors hover:border-control-edge-hover checked:border-blue-600 checked:bg-blue-600 checked:hover:border-blue-700 checked:hover:bg-blue-700 focus-ring disabled:cursor-not-allowed disabled:border-gray-200 disabled:bg-gray-100 disabled:checked:border-control-edge disabled:checked:bg-gray-300";
const TICK_CLASS: &str =
    "pointer-events-none absolute size-3 text-white invisible peer-checked:visible";
const TICK_STROKE_WIDTH: &str = "3.5";
const MARK_CLASS: &str =
    "inline-flex size-4 shrink-0 items-center justify-center rounded border transition-colors";
const UNCHECKED_MARK_CLASS: &str = "border-control-edge bg-surface";
const CHECKED_MARK_CLASS: &str = "border-blue-600 bg-blue-600";
const MARK_TICK_CLASS: &str = "size-3 text-white";
const MIXED_BAR_CLASS: &str = "h-0.5 w-2 rounded-full bg-white";
const CHECK_BUTTON_CLASS: &str =
    "inline-flex size-4 cursor-pointer items-center justify-center rounded align-middle focus-ring";

pub(crate) fn checkbox_mark(checked: Signal<bool>) -> AnyView {
    let class = move || {
        let tone = if checked.get() {
            CHECKED_MARK_CLASS
        } else {
            UNCHECKED_MARK_CLASS
        };
        format!("{MARK_CLASS} {tone}")
    };
    let tick = move || {
        checked.get().then(|| {
            view! { <Icon glyph=LucideGlyph::Check class=MARK_TICK_CLASS stroke_width=TICK_STROKE_WIDTH /> }
        })
    };
    view! {
        <span aria-hidden="true" class=class>
            {tick}
        </span>
    }
    .into_any()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CheckState {
    Unchecked,
    Mixed,
    Checked,
}

impl CheckState {
    pub(crate) fn of(checked: bool) -> Self {
        if checked {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }

    fn spoken(self) -> &'static str {
        match self {
            Self::Unchecked => "false",
            Self::Mixed => "mixed",
            Self::Checked => "true",
        }
    }

    fn mark_class(self) -> &'static str {
        match self {
            Self::Unchecked => UNCHECKED_MARK_CLASS,
            Self::Mixed | Self::Checked => CHECKED_MARK_CLASS,
        }
    }

    fn mark(self) -> Option<AnyView> {
        match self {
            Self::Unchecked => None,
            Self::Mixed => Some(view! { <span class=MIXED_BAR_CLASS></span> }.into_any()),
            Self::Checked => Some(
                view! { <Icon glyph=LucideGlyph::Check class=MARK_TICK_CLASS stroke_width=TICK_STROKE_WIDTH /> }
                    .into_any(),
            ),
        }
    }
}

pub(crate) fn check_button(
    state: Signal<CheckState>,
    label: &'static str,
    on_click: impl Fn(leptos::ev::MouseEvent) + 'static,
) -> AnyView {
    view! {
        <button
            type="button"
            role="checkbox"
            aria-checked=move || state.get().spoken()
            aria-label=label
            class=CHECK_BUTTON_CLASS
            on:click=on_click
        >
            <span
                aria-hidden="true"
                class=move || {
                    format!("pointer-events-none {MARK_CLASS} {}", state.get().mark_class())
                }
            >
                {move || state.get().mark()}
            </span>
        </button>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <span class="relative inline-flex size-4 shrink-0 items-center justify-center">
            <input
                type="checkbox"
                class=BOX_CLASS
                prop:checked=move || checked.get()
                prop:disabled=move || disabled.get()
                on:change=move |_| on_toggle.run(())
            />
            <Icon glyph=LucideGlyph::Check class=TICK_CLASS stroke_width=TICK_STROKE_WIDTH />
        </span>
    }
    .into_any()
}
