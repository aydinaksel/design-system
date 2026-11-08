use leptos::ev::{BeforeUnloadEvent, ClipboardEvent, DragEvent};
use leptos::html::Input;
use leptos::prelude::*;
use leptos_use_event_listener::use_event_listener;
use leptos_use_window::use_window;

use super::arrange::{
    arrange_items, arrange_toolbar, arranged, Arranging, FileArranger, FileCategory,
};
use super::display::{file_items, list_class, FileItem, FileLayout, ItemHandlers, ItemState};
use super::drop_target::{files_in, offer_drop, pasted_files, DropTarget};
use super::removal::Removals;
use super::toolbar::{category_chips, drop_zone_view, toolbar_view};
use super::upload::{FileRemover, FileUploader, UploadRules};
use super::upload_queue::UploadQueue;
use super::viewer::FileViewer;
use super::StoredFile;

const DROP_OVERLAY_CLASS: &str = "pointer-events-none absolute inset-0 z-20 flex items-center justify-center rounded-surface border-2 border-dashed border-blue-500 bg-blue-50/90 text-sm font-medium text-blue-700";
const SECTION_CLASS: &str = "relative space-y-3 rounded-surface transition-shadow";
const PAGE_DRAG_CLASS: &str = "ring-2 ring-blue-200 ring-offset-4";
const CAMERA_ACCEPT: &str = "image/*";
const REAR_CAMERA: &str = "environment";

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
pub(super) struct Presentation {
    pub(super) primary: bool,
    pub(super) categories: StoredValue<Vec<FileCategory>>,
    pub(super) filter: RwSignal<Option<String>>,
}

impl Presentation {
    fn category_label(self, key: &str) -> Option<String> {
        self.categories.with_value(|categories| {
            categories
                .iter()
                .find(|category| category.key == key)
                .map(|category| category.label.clone())
        })
    }

    fn shows(self, file: &StoredFile) -> bool {
        self.filter.with(|filter| {
            filter
                .as_ref()
                .is_none_or(|category| file.category.as_ref() == Some(category))
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct SectionState {
    files: Signal<Vec<StoredFile>>,
    pub(super) queue: UploadQueue,
    removals: Removals,
    pub(super) arranging: Arranging,
    pub(super) presentation: Presentation,
}

impl SectionState {
    fn defer_to_fresh_files(self) {
        let files = self.files;
        Effect::new(move |previous: Option<()>| {
            files.track();
            if previous.is_some() {
                self.arranging.saved.set(None);
                self.removals.forget_removed();
            }
        });
    }

    pub(super) fn stored_in_order(self) -> Vec<StoredFile> {
        let removed = self.removals.removed_keys();
        let visible: Vec<StoredFile> = self
            .files
            .get()
            .into_iter()
            .filter(|file| !removed.contains(&file.key))
            .collect();
        self.arranging.saved.with(|saved| match saved {
            Some(arrangement) => arranged(visible, arrangement),
            None => visible,
        })
    }

    fn stored_item(self, file: StoredFile, primary_key: Option<&str>) -> FileItem {
        let removing = self.removals.is_removing(&file.key);
        let category_label = file
            .category
            .as_deref()
            .and_then(|key| self.presentation.category_label(key));
        FileItem {
            state: if removing {
                ItemState::Removing
            } else {
                ItemState::Stored
            },
            can_remove: self.removals.allowed() && !removing,
            primary: primary_key == Some(file.key.as_str()),
            category_label,
            ..FileItem::stored(file)
        }
    }

    fn items(self) -> Vec<FileItem> {
        let removed = self.removals.removed_keys();
        let stored = self.stored_in_order();
        let primary_key = self
            .presentation
            .primary
            .then(|| stored.first().map(|file| file.key.clone()))
            .flatten();
        let stored_keys: Vec<String> = stored.iter().map(|file| file.key.clone()).collect();
        let stored_items: Vec<FileItem> = stored
            .into_iter()
            .filter(|file| self.presentation.shows(file))
            .map(|file| self.stored_item(file, primary_key.as_deref()))
            .collect();
        let local_items = self
            .queue
            .items(&stored_keys)
            .into_iter()
            .filter(|item| !removed.contains(&item.file.key));
        stored_items.into_iter().chain(local_items).collect()
    }

    fn has_anything(self) -> bool {
        !self.stored_in_order().is_empty() || !self.queue.is_empty()
    }

    pub(super) fn category_counts(self) -> Vec<(FileCategory, usize)> {
        let stored = self.stored_in_order();
        self.presentation.categories.with_value(|categories| {
            categories
                .iter()
                .map(|category| {
                    let count = stored
                        .iter()
                        .filter(|file| file.category.as_deref() == Some(category.key.as_str()))
                        .count();
                    (category.clone(), count)
                })
                .filter(|(_, count)| *count > 0)
                .collect()
        })
    }

    fn handlers(self, items: Signal<Vec<FileItem>>) -> ItemHandlers {
        let remove = move |key: String| {
            if self.queue.dismiss(&key) {
                return;
            }
            let stored = items.with_untracked(|items| {
                stored_files(items).into_iter().find(|file| file.key == key)
            });
            if let Some(file) = stored {
                self.removals.ask(file);
            }
        };
        ItemHandlers {
            retry: Some(Callback::new(move |key: String| self.queue.retry(&key))),
            remove: Some(Callback::new(remove)),
        }
    }
}

fn stored_files(items: &[FileItem]) -> Vec<StoredFile> {
    items
        .iter()
        .filter(|item| item.state == ItemState::Stored)
        .map(|item| item.file.clone())
        .collect()
}

struct Picker {
    input: NodeRef<Input>,
    accept: Option<String>,
    multiple: bool,
    capture: Option<&'static str>,
}

impl Picker {
    fn opener(&self) -> Callback<()> {
        let input = self.input;
        Callback::new(move |()| {
            if let Some(element) = input.get_untracked() {
                element.click();
            }
        })
    }
}

fn hidden_picker(
    picker: Picker,
    deliver: impl Fn(Vec<web_sys::File>) + Send + Sync + 'static,
) -> AnyView {
    let Picker {
        input,
        accept,
        multiple,
        capture,
    } = picker;
    view! {
        <input
            node_ref=input
            type="file"
            class="sr-only"
            tabindex="-1"
            aria-hidden="true"
            accept=accept
            multiple=multiple
            capture=capture
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

struct ContentParts {
    state: SectionState,
    items: Signal<Vec<FileItem>>,
    layout: FileLayout,
    label: String,
    hint: String,
    disabled: Signal<bool>,
    open_picker: Callback<()>,
    take_photo: Option<Callback<()>>,
    open_key: RwSignal<Option<String>>,
}

fn section_content(parts: ContentParts) -> AnyView {
    let ContentParts {
        state,
        items,
        layout,
        label,
        hint,
        disabled,
        open_picker,
        take_photo,
        open_key,
    } = parts;
    let presentation = state.presentation;
    let handlers = state.handlers(items);
    let content = move || {
        if state.arranging.is_active() {
            return view! {
                {arrange_toolbar(state.arranging)}
                {arrange_items(
                    state.arranging,
                    layout,
                    label.clone(),
                    presentation.primary,
                    presentation.categories,
                    list_class(layout),
                )}
            }
            .into_any();
        }
        if !state.has_anything() {
            return drop_zone_view(hint.clone(), disabled, open_picker, take_photo);
        }
        view! {
            {toolbar_view(hint.clone(), &state, disabled, open_picker, take_photo)}
            {category_chips(&state)}
            {file_items(items, layout, label.clone(), open_key, handlers)}
        }
        .into_any()
    };
    view! { {content} }.into_any()
}

#[component]
pub fn FileSection(
    #[prop(into)] files: Signal<Vec<StoredFile>>,
    #[prop(into)] label: String,
    #[prop(optional)] layout: FileLayout,
    #[prop(optional)] rules: UploadRules,
    upload: FileUploader,
    #[prop(optional)] remove: Option<FileRemover>,
    #[prop(optional)] arrange: Option<FileArranger>,
    #[prop(optional)] primary: bool,
    #[prop(optional)] categories: Vec<FileCategory>,
    #[prop(optional)] camera: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let hint = rules.hint();
    let file_picker = Picker {
        input: NodeRef::new(),
        accept: rules.accept_attribute(),
        multiple: rules.multiple(),
        capture: None,
    };
    let camera_picker = (camera && rules.accepts_photos()).then(|| Picker {
        input: NodeRef::new(),
        accept: Some(CAMERA_ACCEPT.to_owned()),
        multiple: false,
        capture: Some(REAR_CAMERA),
    });
    let state = SectionState {
        files,
        queue: UploadQueue::new(upload, rules),
        removals: Removals::new(remove),
        arranging: Arranging::new(arrange),
        presentation: Presentation {
            primary,
            categories: StoredValue::new(categories),
            filter: RwSignal::new(None),
        },
    };
    state.defer_to_fresh_files();
    state.arranging.follow_pointer();
    warn_before_leaving(state.queue);
    let drop_target = DropTarget::new();
    let open_key: RwSignal<Option<String>> = RwSignal::new(None);

    let items = Signal::derive(move || state.items());
    let viewable = Signal::derive(move || items.with(|items| stored_files(items)));
    let open_picker = file_picker.opener();
    let take_photo = camera_picker.as_ref().map(Picker::opener);
    let accepting = move || !disabled.get() && !state.arranging.is_active();
    let deliver = move |files: Vec<web_sys::File>| {
        let accepting_now = !disabled.get_untracked() && !state.arranging.is_active_untracked();
        if !files.is_empty() && accepting_now {
            state.queue.add(files);
        }
    };
    let section_class = move || {
        let highlight = drop_target.files_elsewhere_on_page() && accepting();
        format!(
            "{SECTION_CLASS} {}",
            if highlight { PAGE_DRAG_CLASS } else { "" }
        )
    };
    let dragging_over = move || drop_target.files_over_section() && accepting();
    let content = section_content(ContentParts {
        state,
        items,
        layout,
        label,
        hint,
        disabled,
        open_picker,
        take_photo,
        open_key,
    });

    view! {
        <div
            class=section_class
            on:dragenter=move |event: DragEvent| drop_target.enter(&event)
            on:dragover=move |event: DragEvent| offer_drop(&event, accepting())
            on:dragleave=move |event: DragEvent| drop_target.leave(&event)
            on:drop=move |event: DragEvent| deliver(drop_target.dropped(&event))
            on:paste=move |event: ClipboardEvent| {
                let pasted = pasted_files(&event);
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
            <p class="sr-only" aria-live="polite">
                {move || state.arranging.announcement.get()}
            </p>
            {hidden_picker(file_picker, deliver)}
            {camera_picker.map(|picker| hidden_picker(picker, deliver))}
            <FileViewer files=viewable open_key />
            {state.removals.dialog(state.queue)}
        </div>
    }
    .into_any()
}
