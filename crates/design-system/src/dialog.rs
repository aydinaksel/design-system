use std::sync::Mutex;

use leptos::context::Provider;
use leptos::ev::{Event, MouseEvent};
use leptos::prelude::*;

use crate::element_key::ElementKey;
use crate::icons;

const PANEL_CLASS: &str = "w-full rounded-xl border border-edge bg-surface p-5 text-gray-900 shadow-xl transition-[opacity,scale] duration-200 ease-out backdrop:bg-gray-900/40 focus:outline-none starting:scale-95 starting:opacity-0 motion-reduce:transition-none";
const CLOSE_BUTTON_CLASS: &str =
    "cursor-pointer rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    Small,
    #[default]
    Medium,
    Large,
    FullScreen,
}

impl DialogSize {
    const fn panel_class(self) -> &'static str {
        match self {
            Self::Small => "max-w-sm",
            Self::Medium => "max-w-md",
            Self::Large => "max-w-2xl",
            Self::FullScreen => "flex h-[calc(100dvh-2rem)] max-w-none flex-col",
        }
    }
}

#[derive(Clone, Copy)]
struct DialogClose(Callback<()>);

#[must_use]
pub fn use_dialog_close() -> Option<Callback<()>> {
    use_context::<DialogClose>().map(|DialogClose(close)| close)
}

#[cfg(feature = "hydrate")]
mod browser {
    use leptos::wasm_bindgen::JsCast;
    use leptos::web_sys::{EventTarget, HtmlDialogElement, HtmlElement, MouseEvent};

    const AUTOFOCUS_SELECTOR: &str = "[data-autofocus]";

    pub(super) fn open_modal(dialog: &HtmlDialogElement) {
        if !dialog.open() {
            dialog.show_modal().unwrap_or_default();
        }
        let marked = dialog
            .query_selector(AUTOFOCUS_SELECTOR)
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<HtmlElement>().ok());
        if let Some(element) = marked {
            element.focus().unwrap_or_default();
        }
    }

    pub(super) fn clicked_the_backdrop(dialog: &HtmlDialogElement, event: &MouseEvent) -> bool {
        let dialog_target: &EventTarget = dialog.as_ref();
        let on_the_dialog_itself = event
            .target()
            .is_some_and(|target| &target == dialog_target);
        let bounds = dialog.get_bounding_client_rect();
        let x = f64::from(event.client_x());
        let y = f64::from(event.client_y());
        let outside =
            x < bounds.left() || x > bounds.right() || y < bounds.top() || y > bounds.bottom();
        on_the_dialog_itself && outside
    }
}

pub(crate) fn show_modal_while_mounted(
    #[cfg_attr(
        not(feature = "hydrate"),
        expect(
            unused_variables,
            reason = "the server renders a dialog closed and only the browser opens it"
        )
    )]
    dialog: NodeRef<leptos::html::Dialog>,
) {
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            if let Some(element) = dialog.get() {
                browser::open_modal(&element);
            }
        });
        on_cleanup(move || {
            if let Some(element) = dialog.try_get_untracked().flatten() {
                element.close();
            }
        });
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Dialog(
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] description: Option<Signal<String>>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(optional)] size: DialogSize,
    #[prop(optional)] alert: bool,
    children: Children,
) -> impl IntoView {
    let dialog_number = ElementKey::next();
    let title_id = format!("dialog-{dialog_number}-title");
    let description_id = description
        .is_some()
        .then(|| format!("dialog-{dialog_number}-description"));
    let dialog: NodeRef<leptos::html::Dialog> = NodeRef::new();
    show_modal_while_mounted(dialog);

    let on_cancel = move |event: Event| {
        event.prevent_default();
        on_close.run(());
    };
    let on_click = move |event: MouseEvent| {
        #[cfg(feature = "hydrate")]
        if dialog
            .get_untracked()
            .is_some_and(|element| browser::clicked_the_backdrop(&element, &event))
        {
            on_close.run(());
        }
        #[cfg(not(feature = "hydrate"))]
        drop(event);
    };

    let close_button = (!alert).then(|| {
        view! {
            <button
                type="button"
                class=CLOSE_BUTTON_CLASS
                aria-label="Close"
                on:click=move |_| on_close.run(())
            >
                {icons::icon(icons::LucideGlyph::X, "h-4 w-4")}
            </button>
        }
    });
    let description_view = description.map(|description| {
        view! {
            <p id=description_id.clone() class="mt-1 text-sm text-gray-600">
                {move || description.get()}
            </p>
        }
    });
    let content = view! {
        <div class="mb-4">
            <div class="flex items-start justify-between gap-4">
                <h2 id=title_id.clone() class="text-base font-semibold text-gray-900">
                    {move || title.get()}
                </h2>
                {close_button}
            </div>
            {description_view}
        </div>
        <Provider value=DialogClose(on_close)>{children()}</Provider>
    }
    .into_any();
    let content = StoredValue::new(Mutex::new(Some(content)));
    let title_id = StoredValue::new(title_id);
    let description_id = StoredValue::new(description_id);

    view! {
        <dialog
            node_ref=dialog
            role=alert.then_some("alertdialog")
            aria-labelledby=title_id.get_value()
            aria-describedby=description_id.get_value()
            class=format!("{PANEL_CLASS} {}", size.panel_class())
            on:cancel=on_cancel
            on:click=on_click
        >
            {content
                .with_value(|content| {
                    content.lock().ok().and_then(|mut content| content.take())
                })}
        </dialog>
    }
    .into_any()
}
