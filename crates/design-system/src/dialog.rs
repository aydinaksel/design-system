use std::sync::atomic::{AtomicU64, Ordering};

use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html::Div;
use leptos::portal::Portal;
use leptos::prelude::*;

use crate::icons;

static NEXT_DIALOG_NUMBER: AtomicU64 = AtomicU64::new(0);

const BACKDROP_CLASS: &str = "fixed inset-0 bg-gray-900/40 transition-opacity duration-200 ease-out starting:opacity-0 motion-reduce:transition-none";
const PANEL_CLASS: &str = "relative w-full rounded-lg border border-gray-200 bg-white p-5 shadow-xl transition duration-200 ease-out focus:outline-none starting:scale-95 starting:opacity-0 motion-reduce:transition-none";
const CLOSE_BUTTON_CLASS: &str =
    "cursor-pointer rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl DialogSize {
    const fn width_class(self) -> &'static str {
        match self {
            Self::Small => "max-w-sm",
            Self::Medium => "max-w-md",
            Self::Large => "max-w-2xl",
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
    use std::cell::RefCell;

    use leptos::prelude::document;
    use leptos::wasm_bindgen::JsCast;
    use leptos::web_sys::{Element, HtmlElement};

    const FOCUSABLE_SELECTOR: &str = "a[href], button:not([disabled]), input:not([disabled]):not([type=hidden]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";
    const AUTOFOCUS_SELECTOR: &str = "[data-autofocus]";

    thread_local! {
        static OPEN_DIALOGS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    }

    pub(super) fn opened(number: u64) {
        let first = OPEN_DIALOGS.with(|open| {
            let mut open = open.borrow_mut();
            open.push(number);
            open.len() == 1
        });
        if first {
            lock_page_scroll(true);
        }
    }

    pub(super) fn closed(number: u64) {
        let none_left = OPEN_DIALOGS.with(|open| {
            let mut open = open.borrow_mut();
            open.retain(|other| *other != number);
            open.is_empty()
        });
        if none_left {
            lock_page_scroll(false);
        }
    }

    pub(super) fn is_topmost(number: u64) -> bool {
        OPEN_DIALOGS.with(|open| open.borrow().last() == Some(&number))
    }

    fn lock_page_scroll(locked: bool) {
        let Some(body) = document().body() else {
            return;
        };
        let style = body.style();
        if locked {
            style.set_property("overflow", "hidden").unwrap_or_default();
        } else {
            style.remove_property("overflow").unwrap_or_default();
        }
    }

    pub(super) fn focused_element() -> Option<Element> {
        document().active_element()
    }

    pub(super) fn focus_initial(panel: &HtmlElement) {
        let marked = panel
            .query_selector(AUTOFOCUS_SELECTOR)
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<HtmlElement>().ok());
        marked
            .unwrap_or_else(|| panel.clone())
            .focus()
            .unwrap_or_default();
    }

    pub(super) fn restore_focus(element: Option<Element>) {
        if let Some(element) = element.and_then(|element| element.dyn_into::<HtmlElement>().ok()) {
            element.focus().unwrap_or_default();
        }
    }

    fn focusable_elements(panel: &HtmlElement) -> Vec<HtmlElement> {
        let Ok(found) = panel.query_selector_all(FOCUSABLE_SELECTOR) else {
            return Vec::new();
        };
        (0..found.length())
            .filter_map(|index| found.get(index))
            .filter_map(|node| node.dyn_into::<HtmlElement>().ok())
            .filter(|element| element.offset_width() > 0 || element.offset_height() > 0)
            .collect()
    }

    pub(super) fn wrap_focus(panel: &HtmlElement, backwards: bool) -> bool {
        let focusables = focusable_elements(panel);
        let (Some(first), Some(last)) = (focusables.first(), focusables.last()) else {
            return true;
        };
        let active = document().active_element();
        let is_active = |element: &HtmlElement| {
            active
                .as_ref()
                .is_some_and(|active| active.is_same_node(Some(element)))
        };
        let wrap_to = if backwards && (is_active(first) || is_active(panel)) {
            last
        } else if !backwards && is_active(last) {
            first
        } else {
            return false;
        };
        wrap_to.focus().unwrap_or_default();
        true
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one widget: the focus handling, dismissal and markup share the dialog's state"
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
    let dialog_number = NEXT_DIALOG_NUMBER.fetch_add(1, Ordering::Relaxed);
    let title_id = format!("dialog-{dialog_number}-title");
    let description_id = description
        .is_some()
        .then(|| format!("dialog-{dialog_number}-description"));
    let panel: NodeRef<Div> = NodeRef::new();
    let container: NodeRef<Div> = NodeRef::new();
    provide_context(DialogClose(on_close));

    #[cfg(feature = "hydrate")]
    {
        let previous_focus = StoredValue::new_local(browser::focused_element());
        browser::opened(dialog_number);
        request_animation_frame(move || {
            if let Some(element) = panel.get_untracked() {
                browser::focus_initial(&element);
            }
        });
        let escape = window_event_listener(leptos::ev::keydown, move |event| {
            if event.key() != "Escape" || event.default_prevented() {
                return;
            }
            if browser::is_topmost(dialog_number) {
                event.prevent_default();
                on_close.run(());
            }
        });
        on_cleanup(move || {
            escape.remove();
            browser::closed(dialog_number);
            browser::restore_focus(previous_focus.get_value());
        });
    }

    let on_keydown = move |event: KeyboardEvent| {
        #[cfg(feature = "hydrate")]
        if event.key() == "Tab"
            && panel
                .get_untracked()
                .is_some_and(|element| browser::wrap_focus(&element, event.shift_key()))
        {
            event.prevent_default();
        }
        #[cfg(not(feature = "hydrate"))]
        drop(event);
    };
    let on_click = move |event: MouseEvent| {
        let outside_panel =
            event
                .target()
                .zip(container.get_untracked())
                .is_some_and(|(target, element)| {
                    let element: &web_sys::EventTarget = element.as_ref();
                    &target == element
                });
        if outside_panel {
            on_close.run(());
        }
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
        {children()}
    }
    .into_any();
    let content = StoredValue::new_local(Some(content));
    let title_id = StoredValue::new(title_id);
    let description_id = StoredValue::new(description_id);

    view! {
        <Portal>
            <div class="relative z-50">
                <div class=BACKDROP_CLASS aria-hidden="true"></div>
                <div class="fixed inset-0 w-screen overflow-y-auto">
                    <div
                        node_ref=container
                        class="flex min-h-full items-center justify-center p-4"
                        on:click=on_click
                    >
                        <div
                            node_ref=panel
                            role=if alert { "alertdialog" } else { "dialog" }
                            aria-modal="true"
                            aria-labelledby=title_id.get_value()
                            aria-describedby=description_id.get_value()
                            tabindex="-1"
                            class=format!("{PANEL_CLASS} {}", size.width_class())
                            on:keydown=on_keydown
                        >
                            {content.try_update_value(Option::take).flatten()}
                        </div>
                    </div>
                </div>
            </div>
        </Portal>
    }
    .into_any()
}
