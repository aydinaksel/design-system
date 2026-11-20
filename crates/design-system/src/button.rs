#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonTone {
    Primary,
    #[default]
    Secondary,
    Danger,
    DangerOutline,
    Ghost,
    Subtle,
}

impl ButtonTone {
    fn class(self) -> &'static str {
        match self {
            ButtonTone::Primary => {
                "border border-transparent bg-blue-600 text-white shadow-sm hover:bg-blue-700"
            }
            ButtonTone::Secondary => {
                "border border-control-edge bg-surface text-gray-700 shadow-sm hover:bg-gray-50"
            }
            ButtonTone::Danger => {
                "border border-transparent bg-red-600 text-white shadow-sm hover:bg-red-700"
            }
            ButtonTone::DangerOutline => {
                "border border-red-300 bg-surface text-red-600 shadow-sm hover:bg-red-50"
            }
            ButtonTone::Ghost => {
                "border border-transparent bg-transparent text-gray-400 hover:bg-gray-100 hover:text-gray-600"
            }
            ButtonTone::Subtle => {
                "border border-transparent bg-transparent text-gray-600 hover:bg-gray-100 hover:text-gray-900"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Standard,
    Compact,
    Dense,
}

impl ButtonSize {
    pub(crate) fn class(self) -> &'static str {
        match self {
            ButtonSize::Standard => "h-9 gap-1.5 px-3 text-sm",
            ButtonSize::Compact => "h-8 gap-1.5 px-2.5 text-sm",
            ButtonSize::Dense => "gap-1 px-2.5 py-1 text-xs",
        }
    }

    fn square_class(self) -> &'static str {
        match self {
            ButtonSize::Standard => "size-9 shrink-0 text-sm",
            ButtonSize::Compact => "size-8 shrink-0 text-sm",
            ButtonSize::Dense => "size-[1.625rem] shrink-0 text-xs",
        }
    }
}

pub(crate) const SHARED_CLASS: &str = "inline-flex items-center justify-center whitespace-nowrap rounded-lg font-medium transition-colors cursor-pointer focus-ring disabled:opacity-60 disabled:cursor-not-allowed aria-disabled:opacity-60 aria-disabled:cursor-not-allowed";

#[must_use]
pub fn button_class(tone: ButtonTone, size: ButtonSize) -> String {
    format!("{SHARED_CLASS} {} {}", size.class(), tone.class())
}

#[must_use]
pub fn icon_button_class(tone: ButtonTone, size: ButtonSize) -> String {
    format!("{SHARED_CLASS} {} {}", size.square_class(), tone.class())
}

#[must_use]
pub fn button_icon_class(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Standard | ButtonSize::Compact => "h-4 w-4 shrink-0 opacity-70",
        ButtonSize::Dense => "h-3.5 w-3.5 shrink-0 opacity-70",
    }
}

fn leading_icon(
    glyph: Option<crate::icons::LucideGlyph>,
    size: ButtonSize,
    busy: leptos::prelude::Signal<bool>,
) -> impl Fn() -> Option<leptos::prelude::AnyView> {
    use leptos::prelude::Get;
    move || {
        if busy.get() {
            Some(crate::icons::spinner(button_icon_class(size)))
        } else {
            glyph.map(|glyph| crate::icons::icon(glyph, button_icon_class(size)))
        }
    }
}

mod components {
    use leptos::ev::MouseEvent;
    use leptos::prelude::*;

    use super::{
        button_class, button_icon_class, icon_button_class, leading_icon, ButtonSize, ButtonTone,
    };
    use crate::icons::{self, LucideGlyph};
    use crate::tooltip::Tooltip;

    /// A button with a label. `busy` shows a spinner in place of the icon and
    /// ignores clicks for as long as the work the button started is running.
    #[component]
    pub fn Button(
        #[prop(optional)] tone: ButtonTone,
        #[prop(optional)] size: ButtonSize,
        #[prop(optional_no_strip)] glyph: Option<LucideGlyph>,
        #[prop(optional, into)] disabled: Signal<bool>,
        #[prop(optional, into)] busy: Signal<bool>,
        #[prop(optional)] submits_form: bool,
        #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
        children: Children,
    ) -> impl IntoView {
        view! {
            <button
                type=if submits_form { "submit" } else { "button" }
                class=button_class(tone, size)
                prop:disabled=move || disabled.get() || busy.get()
                aria-busy=move || busy.get().then_some("true")
                on:click=move |event: MouseEvent| {
                    if let Some(on_click) = on_click {
                        on_click.run(event);
                    }
                }
            >
                {leading_icon(glyph, size, busy)}
                {children()}
            </button>
        }
        .into_any()
    }

    /// A button that is only an icon. `label` names it for assistive
    /// technology and is shown as its tooltip.
    #[component]
    pub fn IconButton(
        glyph: LucideGlyph,
        #[prop(into)] label: String,
        #[prop(default = ButtonTone::Ghost)] tone: ButtonTone,
        #[prop(optional)] size: ButtonSize,
        #[prop(optional, into)] disabled: Signal<bool>,
        #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    ) -> impl IntoView {
        view! {
            <Tooltip label=label.clone()>
                <button
                    type="button"
                    aria-label=label
                    class=icon_button_class(tone, size)
                    prop:disabled=move || disabled.get()
                    on:click=move |event: MouseEvent| {
                        if let Some(on_click) = on_click {
                            on_click.run(event);
                        }
                    }
                >
                    {icons::icon(glyph, button_icon_class(size))}
                </button>
            </Tooltip>
        }
        .into_any()
    }

    /// A link drawn as a button, for an action that is a place to go.
    #[component]
    pub fn LinkButton(
        #[prop(into)] href: Signal<String>,
        #[prop(optional)] tone: ButtonTone,
        #[prop(optional)] size: ButtonSize,
        #[prop(optional_no_strip)] glyph: Option<LucideGlyph>,
        #[prop(optional)] opens_new_tab: bool,
        #[prop(optional)] downloads: bool,
        children: Children,
    ) -> impl IntoView {
        view! {
            <a
                href=move || href.get()
                class=button_class(tone, size)
                target=opens_new_tab.then_some("_blank")
                rel=opens_new_tab.then_some("noopener noreferrer")
                download=downloads.then_some("")
            >
                {glyph.map(|glyph| icons::icon(glyph, button_icon_class(size)))}
                {children()}
            </a>
        }
        .into_any()
    }
}

pub use components::{Button, IconButton, LinkButton};
