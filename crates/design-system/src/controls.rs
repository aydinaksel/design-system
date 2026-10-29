use leptos::prelude::*;

pub(crate) const CONTROL_CLASS: &str = "block w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 shadow-sm transition-colors placeholder:text-gray-400 hover:border-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:cursor-not-allowed disabled:border-gray-200 disabled:bg-gray-50 disabled:text-gray-500 aria-[invalid=true]:border-red-400 aria-[invalid=true]:focus:border-red-500 aria-[invalid=true]:focus:ring-red-500";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FieldElementIds {
    pub(crate) control: String,
    pub(crate) label: String,
    pub(crate) description: Option<String>,
}

pub(crate) fn field_control_id() -> Option<String> {
    use_context::<FieldElementIds>().map(|ids| ids.control)
}

pub(crate) fn field_label_id() -> Option<String> {
    use_context::<FieldElementIds>().map(|ids| ids.label)
}

pub(crate) fn field_description_id() -> Option<String> {
    use_context::<FieldElementIds>().and_then(|ids| ids.description)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputKind {
    #[default]
    Text,
    Decimal,
    Integer,
    Email,
    Telephone,
    WebAddress,
}

impl InputKind {
    fn input_type(self) -> &'static str {
        match self {
            Self::Text | Self::Decimal | Self::Integer => "text",
            Self::Email => "email",
            Self::Telephone => "tel",
            Self::WebAddress => "url",
        }
    }

    fn input_mode(self) -> Option<&'static str> {
        match self {
            Self::Decimal => Some("decimal"),
            Self::Integer => Some("numeric"),
            Self::Text | Self::Email | Self::Telephone | Self::WebAddress => None,
        }
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Input(
    value: RwSignal<String>,
    #[prop(optional)] kind: InputKind,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional_no_strip)] max_characters: Option<usize>,
    #[prop(optional)] autofocus: bool,
    #[prop(optional, into)] invalid: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <input
            data-autofocus=autofocus.then_some("")
            type=kind.input_type()
            inputmode=kind.input_mode()
            maxlength=max_characters.map(|limit| limit.to_string())
            id=field_control_id()
            aria-describedby=field_description_id()
            aria-invalid=move || invalid.get().to_string()
            placeholder=placeholder
            autocomplete="off"
            class=CONTROL_CLASS
            prop:value=move || value.get()
            prop:disabled=move || disabled.get()
            on:input=move |event| value.set(event_target_value(&event))
        />
    }
    .into_any()
}

const DEFAULT_TEXTAREA_ROWS: u32 = 3;

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Textarea(
    value: RwSignal<String>,
    #[prop(default = DEFAULT_TEXTAREA_ROWS)] rows: u32,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] invalid: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <textarea
            id=field_control_id()
            aria-describedby=field_description_id()
            aria-invalid=move || invalid.get().to_string()
            placeholder=placeholder
            rows=rows
            class=format!("{CONTROL_CLASS} resize-y")
            prop:value=move || value.get()
            prop:disabled=move || disabled.get()
            on:input=move |event| value.set(event_target_value(&event))
        ></textarea>
    }
    .into_any()
}

const SWITCH_TRACK_CLASS: &str = "group relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full bg-gray-200 transition-colors duration-200 ease-in-out hover:bg-gray-300 aria-checked:bg-blue-600 aria-checked:hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 disabled:cursor-not-allowed disabled:opacity-50 motion-reduce:transition-none";
const SWITCH_KNOB_CLASS: &str = "pointer-events-none inline-block size-4 translate-x-1 rounded-full bg-white shadow transition-transform duration-200 ease-in-out group-aria-checked:translate-x-6 motion-reduce:transition-none";

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Switch(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="switch"
            id=field_control_id()
            aria-describedby=field_description_id()
            aria-label=label
            aria-checked=move || checked.get().to_string()
            class=SWITCH_TRACK_CLASS
            prop:disabled=move || disabled.get()
            on:click=move |_| on_toggle.run(())
        >
            <span aria-hidden="true" class=SWITCH_KNOB_CLASS></span>
        </button>
    }
    .into_any()
}
