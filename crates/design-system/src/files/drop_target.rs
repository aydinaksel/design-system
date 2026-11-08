use leptos::ev::{ClipboardEvent, DragEvent};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;
use leptos_use::{use_event_listener, use_window};

const FILES_TRANSFER_TYPE: &str = "Files";

fn carries_files(event: &DragEvent) -> bool {
    event.data_transfer().is_some_and(|transfer| {
        transfer
            .types()
            .includes(&JsValue::from_str(FILES_TRANSFER_TYPE), 0)
    })
}

fn set_drop_effect(event: &DragEvent, effect: &str) {
    if let Some(transfer) = event.data_transfer() {
        transfer.set_drop_effect(effect);
    }
}

pub(super) fn files_in(list: Option<web_sys::FileList>) -> Vec<web_sys::File> {
    list.map(|list| {
        (0..list.length())
            .filter_map(|index| list.get(index))
            .collect()
    })
    .unwrap_or_default()
}

pub(super) fn offer_drop(event: &DragEvent, accepting: bool) {
    if carries_files(event) {
        event.prevent_default();
        set_drop_effect(event, if accepting { "copy" } else { "none" });
    }
}

pub(super) fn pasted_files(event: &ClipboardEvent) -> Vec<web_sys::File> {
    files_in(event.clipboard_data().and_then(|data| data.files()))
}

#[derive(Clone, Copy)]
pub(super) struct DropTarget {
    page_depth: RwSignal<u32>,
    section_depth: RwSignal<u32>,
}

impl DropTarget {
    pub(super) fn new() -> Self {
        let target = Self {
            page_depth: RwSignal::new(0),
            section_depth: RwSignal::new(0),
        };
        target.watch_page();
        target
    }

    fn watch_page(self) {
        let page_depth = self.page_depth;
        drop(use_event_listener(
            use_window(),
            leptos::ev::dragenter,
            move |event: DragEvent| {
                if carries_files(&event) {
                    page_depth.update(|depth| *depth = depth.saturating_add(1));
                }
            },
        ));
        drop(use_event_listener(
            use_window(),
            leptos::ev::dragleave,
            move |event: DragEvent| {
                if carries_files(&event) {
                    page_depth.update(|depth| *depth = depth.saturating_sub(1));
                }
            },
        ));
        drop(use_event_listener(
            use_window(),
            leptos::ev::dragover,
            move |event: DragEvent| {
                if carries_files(&event) && !event.default_prevented() {
                    event.prevent_default();
                    set_drop_effect(&event, "none");
                }
            },
        ));
        drop(use_event_listener(
            use_window(),
            leptos::ev::drop,
            move |event: DragEvent| {
                page_depth.set(0);
                if carries_files(&event) && !event.default_prevented() {
                    event.prevent_default();
                }
            },
        ));
    }

    pub(super) fn files_over_section(self) -> bool {
        self.section_depth.get() > 0
    }

    pub(super) fn files_elsewhere_on_page(self) -> bool {
        self.page_depth.get() > 0 && self.section_depth.get() == 0
    }

    pub(super) fn enter(self, event: &DragEvent) {
        if carries_files(event) {
            event.prevent_default();
            self.section_depth
                .update(|depth| *depth = depth.saturating_add(1));
        }
    }

    pub(super) fn leave(self, event: &DragEvent) {
        if carries_files(event) {
            self.section_depth
                .update(|depth| *depth = depth.saturating_sub(1));
        }
    }

    pub(super) fn dropped(self, event: &DragEvent) -> Vec<web_sys::File> {
        self.section_depth.set(0);
        if !carries_files(event) {
            return Vec::new();
        }
        event.prevent_default();
        files_in(event.data_transfer().and_then(|transfer| transfer.files()))
    }
}
