use leptos::prelude::*;

use crate::icons::{self, LucideGlyph};
use crate::tabs::{choice_navigator, selected_key, ChoiceTransition};
use crate::tooltip::Tooltip;

pub const MODE_PARAMETER: &str = "mode";

/// One way of presenting a page's data, offered as an icon in a segmented
/// switch. The first mode is the default and keeps the URL free of its
/// parameter.
#[derive(Clone, Copy)]
pub struct PageMode {
    pub key: &'static str,
    pub label: &'static str,
    pub glyph: LucideGlyph,
}

impl PageMode {
    #[must_use]
    pub const fn new(key: &'static str, label: &'static str, glyph: LucideGlyph) -> Self {
        Self { key, label, glyph }
    }
}

#[must_use]
pub fn selected_mode(modes: &'static [PageMode]) -> Memo<&'static str> {
    selected_key(MODE_PARAMETER, modes.iter().map(|mode| mode.key).collect())
}

fn corner_class(index: usize, count: usize) -> &'static str {
    let is_first = index == 0;
    let is_last = index.saturating_add(1) == count;
    match (is_first, is_last) {
        (true, true) => "rounded-[0.3rem]",
        (true, false) => "rounded-l-[0.3rem]",
        (false, true) => "rounded-r-[0.3rem]",
        (false, false) => "",
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ModeSwitch(
    modes: &'static [PageMode],
    #[prop(optional)] transition: Option<ChoiceTransition>,
) -> impl IntoView {
    let selected = selected_mode(modes);
    let default_key = modes.first().map_or("", |mode| mode.key);
    let choose = choice_navigator(MODE_PARAMETER, default_key, transition);
    let count = modes.len();

    let segments = modes
        .iter()
        .enumerate()
        .map(|(index, mode)| {
            let PageMode { key, label, glyph } = *mode;
            let choose = choose.clone();
            let corners = corner_class(index, count);
            let is_selected = move || selected.get() == key;
            view! {
                <Tooltip label=label>
                    <button
                        type="button"
                        aria-label=label
                        aria-pressed=move || is_selected().to_string()
                        class=move || {
                            let tone = if is_selected() {
                                "bg-blue-50 text-blue-700"
                            } else {
                                "bg-white text-gray-500 hover:bg-gray-50 hover:text-gray-900"
                            };
                            format!("flex cursor-pointer items-center p-2 {corners} {tone}")
                        }
                        on:click=move |_| choose(key)
                    >
                        {icons::icon(glyph, "h-5 w-5 shrink-0")}
                    </button>
                </Tooltip>
            }
        })
        .collect_view();

    view! {
        <div class="inline-flex divide-x divide-gray-300 rounded-md border border-gray-300 shadow-sm">
            {segments}
        </div>
    }
    .into_any()
}
