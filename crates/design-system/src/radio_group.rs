use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;

use crate::controls::{field_control_id, field_description_id, field_label_id};

static NEXT_RADIO_GROUP_NUMBER: AtomicU64 = AtomicU64::new(0);

const RADIO_CLASS: &str = "mt-0.5 size-4 shrink-0 cursor-pointer appearance-none rounded-full border border-gray-300 bg-white transition-all hover:border-gray-400 checked:border-[5px] checked:border-blue-600 checked:hover:border-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 disabled:cursor-not-allowed disabled:border-gray-200 disabled:bg-gray-100 disabled:checked:border-gray-300 motion-reduce:transition-none";
const OPTION_LABEL_CLASS: &str = "block text-sm font-medium text-gray-900";
const OPTION_DESCRIPTION_CLASS: &str = "block text-xs text-gray-500";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub disabled: bool,
}

impl RadioOption {
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            disabled: false,
        }
    }

    #[must_use]
    pub fn with_description(self, description: impl Into<String>) -> Self {
        Self {
            description: Some(description.into()),
            ..self
        }
    }

    #[must_use]
    pub fn disabled(self) -> Self {
        Self {
            disabled: true,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioLayout {
    #[default]
    List,
    Cards,
}

impl RadioLayout {
    const fn group_class(self) -> &'static str {
        match self {
            Self::List => "space-y-3",
            Self::Cards => "grid gap-3 sm:grid-cols-[repeat(auto-fit,minmax(11rem,1fr))]",
        }
    }

    const fn option_class(self) -> &'static str {
        match self {
            Self::List => "flex cursor-pointer items-start gap-3 has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50",
            Self::Cards => "flex cursor-pointer items-start gap-3 rounded-lg border border-gray-200 bg-white px-4 py-3 shadow-sm transition-colors hover:border-gray-300 has-[:checked]:border-blue-600 has-[:checked]:ring-1 has-[:checked]:ring-blue-600 has-[:disabled]:cursor-not-allowed has-[:disabled]:bg-gray-50 has-[:disabled]:opacity-60",
        }
    }
}

#[component]
pub fn RadioGroup(
    #[prop(into)] options: Signal<Vec<RadioOption>>,
    selected: RwSignal<String>,
    #[prop(optional)] layout: RadioLayout,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let group_number = NEXT_RADIO_GROUP_NUMBER.fetch_add(1, Ordering::Relaxed);
    let name = format!("radio-group-{group_number}");
    let labelled_by = field_label_id();
    let label = label.filter(|_| labelled_by.is_none());

    let rows = move || {
        options
            .get()
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let description_id = option
                    .description
                    .as_ref()
                    .map(|_| format!("{name}-option-{index}-description"));
                let value = option.value.clone();
                let checked_value = option.value.clone();
                let option_disabled = option.disabled;
                let description = option.description.map(|description| {
                    view! {
                        <span id=description_id.clone() class=OPTION_DESCRIPTION_CLASS>
                            {description}
                        </span>
                    }
                });
                view! {
                    <label class=layout.option_class()>
                        <input
                            type="radio"
                            name=name.clone()
                            value=option.value
                            aria-describedby=description_id.clone()
                            class=RADIO_CLASS
                            prop:checked=move || selected.with(|current| *current == checked_value)
                            prop:disabled=move || disabled.get() || option_disabled
                            on:change=move |_| selected.set(value.clone())
                        />
                        <span class="min-w-0">
                            <span class=OPTION_LABEL_CLASS>{option.label}</span>
                            {description}
                        </span>
                    </label>
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div
            role="radiogroup"
            id=field_control_id()
            aria-labelledby=labelled_by
            aria-label=label
            aria-describedby=field_description_id()
            aria-disabled=move || disabled.get().then_some("true")
            class=layout.group_class()
        >
            {rows}
        </div>
    }
    .into_any()
}
