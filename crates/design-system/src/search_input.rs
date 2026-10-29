use leptos::ev::{Event, KeyboardEvent};
use leptos::prelude::*;

use crate::icons;

const DEFAULT_PLACEHOLDER: &str = "Search";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchSize {
    #[default]
    Standard,
    Compact,
}

impl SearchSize {
    const fn input_class(self) -> &'static str {
        match self {
            Self::Standard => "block w-full rounded-md border border-gray-300 bg-white py-2 pl-9 pr-9 text-sm text-gray-900 shadow-sm transition-colors placeholder:text-gray-400 hover:border-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500",
            Self::Compact => "block w-full rounded-md border border-gray-300 bg-white py-1.5 pl-7 pr-7 text-xs text-gray-900 transition-colors placeholder:text-gray-400 hover:border-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500",
        }
    }

    const fn search_icon_class(self) -> &'static str {
        match self {
            Self::Standard => {
                "pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
            }
            Self::Compact => {
                "pointer-events-none absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-gray-400"
            }
        }
    }

    const fn clear_button_class(self) -> &'static str {
        match self {
            Self::Standard => "absolute right-2 top-1/2 -translate-y-1/2 cursor-pointer rounded p-0.5 text-gray-400 hover:text-gray-700",
            Self::Compact => "absolute right-1.5 top-1/2 -translate-y-1/2 cursor-pointer rounded p-0.5 text-gray-400 hover:text-gray-700",
        }
    }

    const fn clear_icon_class(self) -> &'static str {
        match self {
            Self::Standard => "h-4 w-4",
            Self::Compact => "h-3.5 w-3.5",
        }
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn SearchInput(
    value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional)] size: SearchSize,
    #[prop(optional, into)] on_submit: Option<Callback<()>>,
    #[prop(optional, into)] on_clear: Option<Callback<()>>,
    #[prop(optional, into)] applied: Signal<bool>,
) -> impl IntoView {
    let placeholder = placeholder.unwrap_or_else(|| DEFAULT_PLACEHOLDER.to_owned());
    let has_something_to_clear = move || !value.read().is_empty() || applied.get();
    let clear = move || {
        value.set(String::new());
        if let Some(on_clear) = on_clear {
            on_clear.run(());
        }
    };
    let on_keydown = move |event: KeyboardEvent| match event.key().as_str() {
        "Enter" => {
            if let Some(on_submit) = on_submit {
                event.prevent_default();
                on_submit.run(());
            }
        }
        "Escape" if has_something_to_clear() => {
            event.prevent_default();
            event.stop_propagation();
            clear();
        }
        _ => {}
    };

    view! {
        <div class="relative w-full">
            {icons::icon(icons::LucideGlyph::Search, size.search_icon_class())}
            <input
                type="text"
                role="searchbox"
                placeholder=placeholder.clone()
                aria-label=placeholder
                autocomplete="off"
                class=size.input_class()
                prop:value=value
                on:input=move |event: Event| value.set(event_target_value(&event))
                on:keydown=on_keydown
            />
            {move || {
                has_something_to_clear()
                    .then(|| {
                        view! {
                            <button
                                type="button"
                                title="Clear Search"
                                aria-label="Clear Search"
                                class=size.clear_button_class()
                                on:click=move |_| clear()
                            >
                                {icons::icon(icons::LucideGlyph::X, size.clear_icon_class())}
                            </button>
                        }
                    })
            }}
        </div>
    }
    .into_any()
}
