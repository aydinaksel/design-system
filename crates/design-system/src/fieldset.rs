use leptos::prelude::*;

use crate::controls::FieldElementIds;
use crate::element_key::ElementKey;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Fieldset(
    #[prop(optional, into)] legend: Option<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let fieldset_number = ElementKey::next();
    let legend_id = legend
        .as_ref()
        .map(|_| format!("fieldset-{fieldset_number}-legend"));
    let data_disabled = move || disabled.get().then_some("");
    let legend_view = legend.map(|legend| {
        view! {
            <div
                id=legend_id.clone()
                data-disabled=data_disabled
                class="text-sm font-semibold text-gray-900 data-disabled:opacity-50"
            >
                {legend}
            </div>
        }
    });
    view! {
        <fieldset
            aria-labelledby=legend_id.clone()
            prop:disabled=move || disabled.get()
            data-disabled=data_disabled
            class="min-w-0 space-y-5 motion-safe:transition-opacity data-disabled:opacity-60"
        >
            {legend_view}
            {children()}
        </fieldset>
    }
    .into_any()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Field(
    #[prop(into)] label: String,
    #[prop(optional, into)] description: Option<ViewFn>,
    children: Children,
) -> impl IntoView {
    let field_number = ElementKey::next();
    let control_id = format!("fieldset-field-{field_number}");
    let label_id = format!("fieldset-field-{field_number}-label");
    let description_id = description
        .as_ref()
        .map(|_| format!("fieldset-field-{field_number}-description"));
    provide_context(FieldElementIds {
        control: control_id.clone(),
        label: label_id.clone(),
        description: description_id.clone(),
        invalid: Signal::default(),
    });
    let description_view = description.map(|description| {
        view! {
            <div id=description_id class="mt-1.5 text-xs text-gray-500">
                {description.run()}
            </div>
        }
    });
    view! {
        <div>
            <label id=label_id for=control_id class="block text-sm font-medium text-gray-900">
                {label}
            </label>
            <div class="mt-1.5">{children()}</div>
            {description_view}
        </div>
    }
    .into_any()
}
