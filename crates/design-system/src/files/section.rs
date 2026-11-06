use leptos::ev::{BeforeUnloadEvent, ClipboardEvent, DragEvent};
use leptos::html::Input;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;
use leptos_use::{use_event_listener, use_window};

use super::display::{file_items, FileItem, FileLayout, ItemHandlers, ItemState};
use super::upload::{FileRemover, FileUploader, UploadRules};
use super::upload_queue::UploadQueue;
use super::viewer::FileViewer;
use super::StoredFile;
use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::confirm_dialog::ConfirmDialog;
use crate::icons::{self, LucideGlyph};
use crate::toast::notify_failure;

const DROP_ZONE_CLASS: &str = "flex w-full cursor-pointer flex-col items-center gap-1 rounded-surface border-2 border-dashed border-gray-300 bg-white px-6 py-8 text-center text-sm transition-colors hover:border-gray-400 focus-ring disabled:cursor-not-allowed disabled:opacity-60";
const DROP_OVERLAY_CLASS: &str = "pointer-events-none absolute inset-0 z-20 flex items-center justify-center rounded-surface border-2 border-dashed border-blue-500 bg-blue-50/90 text-sm font-medium text-blue-700";
const SECTION_CLASS: &str = "relative space-y-3 rounded-surface transition-shadow";
const PAGE_DRAG_CLASS: &str = "ring-2 ring-blue-200 ring-offset-4";
const FILES_TRANSFER_TYPE: &str = "Files";

fn carries_files(event: &DragEvent) -> bool {
    event.data_transfer().is_some_and(|transfer| {
        transfer
            .types()
            .includes(&JsValue::from_str(FILES_TRANSFER_TYPE), 0)
    })
}

fn files_in(list: Option<web_sys::FileList>) -> Vec<web_sys::File> {
    list.map(|list| {
        (0..list.length())
            .filter_map(|index| list.get(index))
            .collect()
    })
    .unwrap_or_default()
}

fn set_drop_effect(event: &DragEvent, effect: &str) {
    if let Some(transfer) = event.data_transfer() {
        transfer.set_drop_effect(effect);
    }
}

fn watch_page_drags(page_drag_depth: RwSignal<u32>) {
    drop(use_event_listener(
        use_window(),
        leptos::ev::dragenter,
        move |event: DragEvent| {
            if carries_files(&event) {
                page_drag_depth.update(|depth| *depth = depth.saturating_add(1));
            }
        },
    ));
    drop(use_event_listener(
        use_window(),
        leptos::ev::dragleave,
        move |event: DragEvent| {
            if carries_files(&event) {
                page_drag_depth.update(|depth| *depth = depth.saturating_sub(1));
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
            page_drag_depth.set(0);
            if carries_files(&event) && !event.default_prevented() {
                event.prevent_default();
            }
        },
    ));
}

fn warn_before_leaving(queue: UploadQueue) {
    drop(use_event_listener(
        use_window(),
        leptos::ev::beforeunload,
        move |event: BeforeUnloadEvent| {
            if queue.has_unfinished() {
                event.prevent_default();
                event.set_return_value("");
            }
        },
    ));
}

#[derive(Clone, Copy)]
struct Removals {
    remover: StoredValue<Option<FileRemover>>,
    removing: RwSignal<Vec<String>>,
    removed: RwSignal<Vec<String>>,
    confirming: RwSignal<Option<StoredFile>>,
    confirm_open: RwSignal<bool>,
}

impl Removals {
    fn new(remover: Option<FileRemover>) -> Self {
        Self {
            remover: StoredValue::new(remover),
            removing: RwSignal::new(Vec::new()),
            removed: RwSignal::new(Vec::new()),
            confirming: RwSignal::new(None),
            confirm_open: RwSignal::new(false),
        }
    }

    fn allowed(self) -> bool {
        self.remover.with_value(Option::is_some)
    }

    fn ask(self, file: StoredFile) {
        self.confirming.set(Some(file));
        self.confirm_open.set(true);
    }

    fn confirmed(self, queue: UploadQueue) {
        let Some(file) = self.confirming.get_untracked() else {
            return;
        };
        let Some(removal) = self
            .remover
            .with_value(|remover| remover.as_ref().map(|remove| remove(file.clone())))
        else {
            return;
        };
        let key = file.key;
        let name = file.name;
        self.removing.update(|keys| keys.push(key.clone()));
        leptos::task::spawn_local(async move {
            let outcome = removal.await;
            self.removing
                .try_update(|keys| keys.retain(|removing| *removing != key));
            match outcome {
                Ok(()) => {
                    self.removed.try_update(|keys| keys.push(key.clone()));
                    queue.forget_uploaded(&key);
                }
                Err(reason) => notify_failure(format!("Couldn't Remove {name}"), Some(reason)),
            }
        });
    }
}

fn section_items(
    files: Signal<Vec<StoredFile>>,
    queue: UploadQueue,
    removals: Removals,
) -> Vec<FileItem> {
    let removed = removals.removed.get();
    let removing = removals.removing.get();
    let can_remove = removals.allowed();
    let stored: Vec<StoredFile> = files
        .get()
        .into_iter()
        .filter(|file| !removed.contains(&file.key))
        .collect();
    let stored_keys: Vec<String> = stored.iter().map(|file| file.key.clone()).collect();
    stored
        .into_iter()
        .map(FileItem::stored)
        .chain(queue.items(&stored_keys))
        .filter(|item| !removed.contains(&item.file.key))
        .map(|item| match item.state {
            ItemState::Stored if removing.contains(&item.file.key) => FileItem {
                state: ItemState::Removing,
                ..item
            },
            ItemState::Stored => FileItem { can_remove, ..item },
            ItemState::Waiting
            | ItemState::Uploading
            | ItemState::Removing
            | ItemState::Failed(_) => item,
        })
        .collect()
}

fn drop_zone_view(hint: String, disabled: Signal<bool>, open_picker: Callback<()>) -> AnyView {
    view! {
        <button
            type="button"
            class=DROP_ZONE_CLASS
            prop:disabled=move || disabled.get()
            on:click=move |_| open_picker.run(())
        >
            {icons::icon(LucideGlyph::Upload, "h-5 w-5 text-gray-400")}
            <span class="font-medium text-gray-700">"Drop Files Here, or Browse"</span>
            <span class="text-xs text-gray-500">{hint}</span>
        </button>
    }
    .into_any()
}

fn toolbar_view(hint: String, disabled: Signal<bool>, open_picker: Callback<()>) -> AnyView {
    view! {
        <div class="flex items-center justify-between gap-3">
            <p class="text-xs text-gray-500">{hint}</p>
            <button
                type="button"
                class=format!(
                    "{} disabled:cursor-not-allowed",
                    button_class(ButtonTone::Secondary, ButtonSize::Compact),
                )
                prop:disabled=move || disabled.get()
                on:click=move |_| open_picker.run(())
            >
                {icons::icon(LucideGlyph::Upload, "h-4 w-4 shrink-0 opacity-70")}
                "Add Files"
            </button>
        </div>
    }
    .into_any()
}

fn removal_dialog(removals: Removals, queue: UploadQueue) -> AnyView {
    let name = move || {
        removals
            .confirming
            .get()
            .map(|file| file.name)
            .unwrap_or_default()
    };
    view! {
        <ConfirmDialog
            open=removals.confirm_open
            title=Signal::derive(move || format!("Remove {}?", name()))
            body="It will be removed for everyone who can see this record."
            confirm_label="Remove"
            tone=ButtonTone::Danger
            on_confirm=Callback::new(move |()| removals.confirmed(queue))
        />
    }
    .into_any()
}

fn stored_files(items: &[FileItem]) -> Vec<StoredFile> {
    items
        .iter()
        .filter(|item| item.state == ItemState::Stored)
        .map(|item| item.file.clone())
        .collect()
}

fn item_handlers(
    queue: UploadQueue,
    removals: Removals,
    items: Signal<Vec<FileItem>>,
) -> ItemHandlers {
    let remove = move |key: String| {
        if queue.dismiss(&key) {
            return;
        }
        let stored = items
            .with_untracked(|items| stored_files(items).into_iter().find(|file| file.key == key));
        if let Some(file) = stored {
            removals.ask(file);
        }
    };
    ItemHandlers {
        retry: Some(Callback::new(move |key: String| queue.retry(&key))),
        remove: Some(Callback::new(remove)),
    }
}

fn hidden_picker(
    input: NodeRef<Input>,
    accept: Option<String>,
    multiple: bool,
    deliver: impl Fn(Vec<web_sys::File>) + Send + Sync + 'static,
) -> AnyView {
    view! {
        <input
            node_ref=input
            type="file"
            class="sr-only"
            tabindex="-1"
            aria-hidden="true"
            accept=accept
            multiple=multiple
            on:change=move |_| {
                let Some(element) = input.get() else {
                    return;
                };
                deliver(files_in(element.files()));
                element.set_value("");
            }
        />
    }
    .into_any()
}

#[component]
pub fn FileSection(
    #[prop(into)] files: Signal<Vec<StoredFile>>,
    #[prop(into)] label: String,
    #[prop(optional)] layout: FileLayout,
    #[prop(optional)] rules: UploadRules,
    upload: FileUploader,
    #[prop(optional)] remove: Option<FileRemover>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let hint = rules.hint();
    let accept = rules.accept_attribute();
    let multiple = rules.multiple();
    let queue = UploadQueue::new(upload, rules);
    let removals = Removals::new(remove);
    let open_key: RwSignal<Option<String>> = RwSignal::new(None);
    let page_drag_depth = RwSignal::new(0_u32);
    let section_drag_depth = RwSignal::new(0_u32);
    let input: NodeRef<Input> = NodeRef::new();
    watch_page_drags(page_drag_depth);
    warn_before_leaving(queue);

    let items = Signal::derive(move || section_items(files, queue, removals));
    let viewable = Signal::derive(move || items.with(|items| stored_files(items)));
    let handlers = item_handlers(queue, removals, items);
    let open_picker = Callback::new(move |()| {
        if let Some(element) = input.get_untracked() {
            element.click();
        }
    });
    let deliver = move |files: Vec<web_sys::File>| {
        if !files.is_empty() && !disabled.get_untracked() {
            queue.add(files);
        }
    };
    let section_class = move || {
        let page_drag =
            page_drag_depth.get() > 0 && section_drag_depth.get() == 0 && !disabled.get();
        format!(
            "{SECTION_CLASS} {}",
            if page_drag { PAGE_DRAG_CLASS } else { "" }
        )
    };
    let dragging_over = move || section_drag_depth.get() > 0 && !disabled.get();
    let toolbar_hint = hint.clone();
    let content = move || {
        if items.with(Vec::is_empty) {
            return drop_zone_view(hint.clone(), disabled, open_picker);
        }
        view! {
            {toolbar_view(toolbar_hint.clone(), disabled, open_picker)}
            {file_items(items, layout, label.clone(), open_key, handlers)}
        }
        .into_any()
    };

    view! {
        <div
            class=section_class
            on:dragenter=move |event: DragEvent| {
                if carries_files(&event) {
                    event.prevent_default();
                    section_drag_depth.update(|depth| *depth = depth.saturating_add(1));
                }
            }
            on:dragover=move |event: DragEvent| {
                if carries_files(&event) {
                    event.prevent_default();
                    set_drop_effect(&event, if disabled.get_untracked() { "none" } else { "copy" });
                }
            }
            on:dragleave=move |event: DragEvent| {
                if carries_files(&event) {
                    section_drag_depth.update(|depth| *depth = depth.saturating_sub(1));
                }
            }
            on:drop=move |event: DragEvent| {
                section_drag_depth.set(0);
                if carries_files(&event) {
                    event.prevent_default();
                    deliver(files_in(event.data_transfer().and_then(|transfer| transfer.files())));
                }
            }
            on:paste=move |event: ClipboardEvent| {
                let pasted = files_in(event.clipboard_data().and_then(|data| data.files()));
                if !pasted.is_empty() {
                    event.prevent_default();
                    deliver(pasted);
                }
            }
        >
            {content}
            <Show when=dragging_over>
                <div class=DROP_OVERLAY_CLASS>"Drop to Attach"</div>
            </Show>
            {hidden_picker(input, accept, multiple, deliver)}
            <FileViewer files=viewable open_key />
            {removal_dialog(removals, queue)}
        </div>
    }
    .into_any()
}
