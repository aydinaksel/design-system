use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use leptos::ev::{FocusEvent, MouseEvent, PointerEvent};
use leptos::prelude::*;
use leptos_use_event_listener::use_event_listener;
use leptos_use_window::use_window;

use super::display::{arrange_media, name_view, FileLayout, NAME_CLASS};
use super::StoredFile;
use crate::badge::badge;
use crate::button::{button_class, icon_button_class, ButtonSize, ButtonTone};
use crate::icons::{self, LucideGlyph};
use crate::listbox::{Listbox, ListboxOption};
use crate::toast::notify_failure;
use crate::tone::Tone;
use crate::tooltip::{OverflowTooltip, Tooltip};

pub type ArrangeFuture = Pin<Box<dyn Future<Output = Result<(), String>>>>;
pub type FileArranger = Arc<dyn Fn(Vec<Arrangement>) -> ArrangeFuture + Send + Sync>;

const KEY_ATTRIBUTE: &str = "data-file-key";
const KEY_SELECTOR: &str = "[data-file-key]";
const ARRANGE_CONTROLS_SELECTOR: &str = "button, [aria-hidden=true]";
const NO_CATEGORY_LABEL: &str = "No Type";
const ROW_CLASS: &str = "flex min-w-0 items-center gap-2 rounded-surface border bg-surface p-2";
const TILE_CLASS: &str = "flex min-w-0 flex-col overflow-hidden rounded-surface border bg-surface";
const RESTING_CLASS: &str = "border-gray-200";
const DRAGGED_CLASS: &str = "border-blue-400 opacity-60 ring-2 ring-blue-200";
const HANDLE_CLASS: &str = "flex shrink-0 cursor-grab touch-none items-center justify-center rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-700 active:cursor-grabbing";

pub fn file_arranger<ArrangeFn, Arrange, ArrangeError>(arrange: ArrangeFn) -> FileArranger
where
    ArrangeFn: Fn(Vec<Arrangement>) -> Arrange + Send + Sync + 'static,
    Arrange: Future<Output = Result<(), ArrangeError>> + 'static,
    ArrangeError: ToString,
{
    Arc::new(move |arrangement| {
        let saving = arrange(arrangement);
        Box::pin(async move { saving.await.map_err(|error| error.to_string()) })
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileCategory {
    pub key: String,
    pub label: String,
}

impl FileCategory {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrangement {
    pub key: String,
    pub category: Option<String>,
}

pub(super) fn arranged(files: Vec<StoredFile>, arrangement: &[Arrangement]) -> Vec<StoredFile> {
    let mut remaining = files;
    let mut ordered: Vec<StoredFile> = Vec::with_capacity(remaining.len());
    for placed in arrangement {
        if let Some(position) = remaining.iter().position(|file| file.key == placed.key) {
            let mut file = remaining.remove(position);
            file.category.clone_from(&placed.category);
            ordered.push(file);
        }
    }
    ordered.extend(remaining);
    ordered
}

pub(super) fn moved<Item>(items: &mut Vec<Item>, from: usize, to: usize) -> bool {
    if from == to || from >= items.len() || to >= items.len() {
        return false;
    }
    let item = items.remove(from);
    items.insert(to, item);
    true
}

fn coordinate(pixels: i32) -> Option<f32> {
    i16::try_from(pixels).ok().map(f32::from)
}

fn key_under_pointer(event: &PointerEvent) -> Option<String> {
    let (x, y) = (coordinate(event.client_x())?, coordinate(event.client_y())?);
    document()
        .element_from_point(x, y)?
        .closest(KEY_SELECTOR)
        .ok()
        .flatten()?
        .get_attribute(KEY_ATTRIBUTE)
}

#[derive(Clone, Copy)]
pub(super) struct Arranging {
    arranger: StoredValue<Option<FileArranger>>,
    draft: RwSignal<Option<Vec<StoredFile>>>,
    pub(super) saved: RwSignal<Option<Vec<Arrangement>>>,
    saving: RwSignal<bool>,
    dragging: RwSignal<Option<String>>,
    pub(super) announcement: RwSignal<String>,
}

impl Arranging {
    pub(super) fn new(arranger: Option<FileArranger>) -> Self {
        Self {
            arranger: StoredValue::new(arranger),
            draft: RwSignal::new(None),
            saved: RwSignal::new(None),
            saving: RwSignal::new(false),
            dragging: RwSignal::new(None),
            announcement: RwSignal::new(String::new()),
        }
    }

    pub(super) fn enabled(self) -> bool {
        self.arranger.with_value(Option::is_some)
    }

    pub(super) fn is_active(self) -> bool {
        self.draft.with(Option::is_some)
    }

    pub(super) fn is_active_untracked(self) -> bool {
        self.draft.with_untracked(Option::is_some)
    }

    pub(super) fn begin(self, files: Vec<StoredFile>) {
        self.draft.set(Some(files));
        self.announcement
            .set("Arranging. Drag files or use the arrow buttons, then save.".to_owned());
    }

    fn cancel(self) {
        self.draft.set(None);
        self.dragging.set(None);
        self.announcement.set("Arranging cancelled.".to_owned());
    }

    fn position(self, key: &str) -> Option<usize> {
        self.draft
            .with(|draft| draft.as_ref()?.iter().position(|file| file.key == key))
    }

    fn count(self) -> usize {
        self.draft.with(|draft| draft.as_ref().map_or(0, Vec::len))
    }

    fn move_to(self, key: &str, target: usize) -> bool {
        let mut name = String::new();
        let mut moved_any = false;
        self.draft.update(|draft| {
            let Some(draft) = draft else {
                return;
            };
            let Some(from) = draft.iter().position(|file| file.key == key) else {
                return;
            };
            name = draft
                .get(from)
                .map(|file| file.name.clone())
                .unwrap_or_default();
            moved_any = moved(draft, from, target);
        });
        if moved_any {
            self.announce_position(&name, target);
        }
        moved_any
    }

    fn announce_position(self, name: &str, index: usize) {
        self.announcement.set(format!(
            "Moved {name} to position {} of {}.",
            index.saturating_add(1),
            self.count()
        ));
    }

    fn step(self, key: &str, later: bool) {
        let Some(from) = self.position(key) else {
            return;
        };
        let target = if later {
            from.checked_add(1)
        } else {
            from.checked_sub(1)
        };
        if let Some(target) = target {
            self.move_to(key, target);
        }
    }

    fn make_primary(self, key: &str) {
        if self.move_to(key, 0) {
            let name = self
                .draft
                .with_untracked(|draft| {
                    draft
                        .as_ref()
                        .and_then(|draft| draft.first().map(|file| file.name.clone()))
                })
                .unwrap_or_default();
            self.announcement
                .set(format!("{name} is now the primary file."));
        }
    }

    fn set_category(self, key: &str, category: Option<String>) {
        self.draft.update(|draft| {
            let file = draft
                .as_mut()
                .and_then(|draft| draft.iter_mut().find(|file| file.key == key));
            if let Some(file) = file {
                file.category = category;
            }
        });
    }

    fn start_drag(self, key: &str, event: &PointerEvent) {
        event.prevent_default();
        self.dragging.set(Some(key.to_owned()));
    }

    pub(super) fn follow_pointer(self) {
        drop(use_event_listener(
            use_window(),
            leptos::ev::pointermove,
            move |event: PointerEvent| {
                self.drag_to_pointer(&event);
            },
        ));
        drop(use_event_listener(
            use_window(),
            leptos::ev::pointerup,
            move |_| {
                self.end_drag();
            },
        ));
        drop(use_event_listener(
            use_window(),
            leptos::ev::pointercancel,
            move |_| {
                self.end_drag();
            },
        ));
    }

    fn drag_to_pointer(self, event: &PointerEvent) {
        let Some(dragged) = self.dragging.get_untracked() else {
            return;
        };
        let target = key_under_pointer(event)
            .filter(|key| *key != dragged)
            .and_then(|key| self.position(&key));
        if let Some(target) = target {
            self.move_to(&dragged, target);
        }
    }

    fn end_drag(self) {
        self.dragging.set(None);
    }

    fn save(self) {
        let Some(draft) = self.draft.get_untracked() else {
            return;
        };
        let arrangement: Vec<Arrangement> = draft
            .into_iter()
            .map(|file| Arrangement {
                key: file.key,
                category: file.category,
            })
            .collect();
        let Some(saving) = self.arranger.with_value(|arranger| {
            arranger
                .as_ref()
                .map(|arrange| arrange(arrangement.clone()))
        }) else {
            return;
        };
        self.saving.set(true);
        leptos::task::spawn_local(async move {
            match saving.await {
                Ok(()) => {
                    self.saved.try_set(Some(arrangement));
                    self.draft.try_set(None);
                    self.announcement.try_set("Order saved.".to_owned());
                }
                Err(reason) => notify_failure("Couldn't Save the Order", Some(reason)),
            }
            self.saving.try_set(false);
        });
    }
}

pub(super) fn arrange_toolbar(arranging: Arranging) -> AnyView {
    let saving = arranging.saving;
    view! {
        <div class="flex flex-wrap items-center justify-between gap-3">
            <p class="text-xs text-gray-500">"Drag files into order, or use the arrow buttons."</p>
            <div class="flex items-center gap-2">
                <button
                    type="button"
                    class=format!(
                        "{} disabled:cursor-not-allowed",
                        button_class(ButtonTone::Secondary, ButtonSize::Compact),
                    )
                    prop:disabled=move || saving.get()
                    on:click=move |_| arranging.cancel()
                >
                    "Cancel"
                </button>
                <button
                    type="button"
                    class=format!(
                        "{} disabled:cursor-not-allowed",
                        button_class(ButtonTone::Primary, ButtonSize::Compact),
                    )
                    prop:disabled=move || saving.get()
                    on:click=move |_| arranging.save()
                >
                    {move || saving.get().then(|| icons::spinner("h-4 w-4"))}
                    "Save Order"
                </button>
            </div>
        </div>
    }
    .into_any()
}

fn step_button(
    arranging: Arranging,
    key: String,
    file_name: &str,
    tooltip: &'static str,
    glyph: LucideGlyph,
    available: Signal<bool>,
    action: fn(Arranging, &str),
) -> AnyView {
    let label = format!("{tooltip}: {file_name}");
    view! {
        <Tooltip label=tooltip>
            <button
                type="button"
                aria-label=label
                class=format!(
                    "{} disabled:cursor-not-allowed disabled:opacity-40",
                    icon_button_class(ButtonTone::Secondary, ButtonSize::Compact),
                )
                prop:disabled=move || !available.get()
                on:click=move |_| action(arranging, &key)
            >
                {icons::icon(glyph, "h-4 w-4")}
            </button>
        </Tooltip>
    }
    .into_any()
}

fn category_picker(
    arranging: Arranging,
    file: &StoredFile,
    categories: &[FileCategory],
) -> Option<AnyView> {
    if categories.is_empty() {
        return None;
    }
    let options: Vec<ListboxOption> = std::iter::once(ListboxOption::new("", NO_CATEGORY_LABEL))
        .chain(
            categories
                .iter()
                .map(|category| ListboxOption::new(category.key.clone(), category.label.clone())),
        )
        .collect();
    let selected = RwSignal::new(file.category.clone().unwrap_or_default());
    let label = format!("Type of {}", file.name);
    let key = file.key.clone();
    Effect::new(move |previous: Option<String>| {
        let chosen = selected.get();
        if previous.is_some_and(|previous| previous != chosen) {
            arranging.set_category(
                &key,
                Some(chosen.clone()).filter(|chosen| !chosen.is_empty()),
            );
        }
        chosen
    });
    Some(view! { <Listbox options=Signal::stored(options) selected label /> }.into_any())
}

struct ItemButtons {
    arranging: Arranging,
    key: String,
    file_name: String,
    layout: FileLayout,
    show_primary: bool,
    is_first: Signal<bool>,
    is_last: Signal<bool>,
}

impl ItemButtons {
    fn view(self) -> AnyView {
        let Self {
            arranging,
            key,
            file_name,
            layout,
            show_primary,
            is_first,
            is_last,
        } = self;
        let (earlier, later) = match layout {
            FileLayout::List => (LucideGlyph::ArrowUp, LucideGlyph::ArrowDown),
            FileLayout::Grid => (LucideGlyph::ArrowLeft, LucideGlyph::ArrowRight),
        };
        let not_first = Signal::derive(move || !is_first.get());
        let not_last = Signal::derive(move || !is_last.get());
        let make_primary = show_primary.then(|| {
            step_button(
                arranging,
                key.clone(),
                &file_name,
                "Make Primary",
                LucideGlyph::Star,
                not_first,
                Arranging::make_primary,
            )
        });
        view! {
            <span class="flex shrink-0 items-center gap-1">
                {step_button(
                    arranging,
                    key.clone(),
                    &file_name,
                    "Move Earlier",
                    earlier,
                    not_first,
                    |arranging, key| arranging.step(key, false),
                )}
                {step_button(
                    arranging,
                    key.clone(),
                    &file_name,
                    "Move Later",
                    later,
                    not_last,
                    |arranging, key| arranging.step(key, true),
                )} {make_primary}
            </span>
        }
        .into_any()
    }
}

fn arrange_item(
    arranging: Arranging,
    file: &StoredFile,
    layout: FileLayout,
    show_primary: bool,
    categories: &[FileCategory],
) -> AnyView {
    let key = file.key.clone();
    let position_key = key.clone();
    let position = Signal::derive(move || arranging.position(&position_key));
    let is_first = Signal::derive(move || position.get() == Some(0));
    let is_last = Signal::derive(move || {
        position
            .get()
            .is_some_and(|index| index.saturating_add(1) >= arranging.count())
    });
    let dragged_key = key.clone();
    let border = move || {
        if arranging.dragging.get().as_deref() == Some(dragged_key.as_str()) {
            DRAGGED_CLASS
        } else {
            RESTING_CLASS
        }
    };
    let drag_key = key.clone();
    let handle = view! {
        <span
            class=HANDLE_CLASS
            aria-hidden="true"
            on:pointerdown=move |event: PointerEvent| arranging.start_drag(&drag_key, &event)
        >
            {icons::icon(LucideGlyph::GripVertical, "h-4 w-4")}
        </span>
    };
    let primary_badge =
        move || (show_primary && is_first.get()).then(|| badge("Primary", Tone::Informative));
    let buttons = ItemButtons {
        arranging,
        key: key.clone(),
        file_name: file.name.clone(),
        layout,
        show_primary,
        is_first,
        is_last,
    }
    .view();
    let picker = category_picker(arranging, file, categories);
    let name_tooltip = OverflowTooltip::ignoring(ARRANGE_CONTROLS_SELECTOR);
    let name = name_view(file, NAME_CLASS, name_tooltip.anchor);
    let name_bubble = name_tooltip.bubble(&file.name);
    match layout {
        FileLayout::List => view! {
            <li
                data-file-key=key
                class=move || format!("{ROW_CLASS} {}", border())
                on:mouseover=move |event: MouseEvent| name_tooltip.pointer_over(&event)
                on:mouseleave=move |_| name_tooltip.hide()
                on:focusin=move |event: FocusEvent| name_tooltip.focused(&event)
                on:focusout=move |_| name_tooltip.hide()
            >
                {name_bubble}
                {handle}
                {arrange_media(file, layout)}
                <span class="flex min-w-0 flex-1 items-center gap-2 text-sm">
                    {name} {primary_badge}
                </span>
                <span class="w-36 shrink-0">{picker}</span>
                {buttons}
            </li>
        }
        .into_any(),
        FileLayout::Grid => view! {
            <li
                data-file-key=key
                class=move || format!("{TILE_CLASS} {}", border())
                on:mouseover=move |event: MouseEvent| name_tooltip.pointer_over(&event)
                on:mouseleave=move |_| name_tooltip.hide()
                on:focusin=move |event: FocusEvent| name_tooltip.focused(&event)
                on:focusout=move |_| name_tooltip.hide()
            >
                {name_bubble}
                <span class="relative">
                    {arrange_media(file, layout)}
                    <span class="absolute left-2 top-2">{primary_badge}</span>
                    <span class="absolute right-2 top-2 rounded-md bg-surface/90 shadow-sm">
                        {handle}
                    </span>
                </span>
                <span class="flex min-w-0 flex-col gap-2 p-2">
                    <span class="min-w-0 text-sm">{name}</span>
                    {picker}
                    {buttons}
                </span>
            </li>
        }
        .into_any(),
    }
}

pub(super) fn arrange_items(
    arranging: Arranging,
    layout: FileLayout,
    label: String,
    show_primary: bool,
    categories: StoredValue<Vec<FileCategory>>,
    list_class: &'static str,
) -> AnyView {
    let draft = arranging.draft;
    view! {
        <ul aria-label=label class=list_class>
            <For
                each=move || draft.get().unwrap_or_default()
                key=|file| file.key.clone()
                children=move |file| {
                    categories
                        .with_value(|categories| {
                            arrange_item(arranging, &file, layout, show_primary, categories)
                        })
                }
            />
        </ul>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(key: &str) -> StoredFile {
        StoredFile::new(key, format!("{key}.jpg"), format!("/{key}.jpg"))
    }

    fn keys(files: &[StoredFile]) -> Vec<&str> {
        files.iter().map(|file| file.key.as_str()).collect()
    }

    #[test]
    fn moving_takes_an_item_out_and_puts_it_at_the_target() {
        let mut items = vec!["a", "b", "c", "d"];
        assert!(moved(&mut items, 3, 0));
        assert_eq!(items, ["d", "a", "b", "c"]);
        assert!(moved(&mut items, 0, 2));
        assert_eq!(items, ["a", "b", "d", "c"]);
    }

    #[test]
    fn moving_out_of_range_or_in_place_changes_nothing() {
        let mut items = vec!["a", "b"];
        assert!(!moved(&mut items, 0, 2));
        assert!(!moved(&mut items, 5, 0));
        assert!(!moved(&mut items, 1, 1));
        assert_eq!(items, ["a", "b"]);
    }

    #[test]
    fn a_saved_arrangement_orders_and_labels_files_and_keeps_newcomers_last() {
        let files = vec![file("a"), file("b"), file("c"), file("new")];
        let arrangement = vec![
            Arrangement {
                key: "c".to_owned(),
                category: Some("swatch".to_owned()),
            },
            Arrangement {
                key: "a".to_owned(),
                category: None,
            },
            Arrangement {
                key: "gone".to_owned(),
                category: None,
            },
            Arrangement {
                key: "b".to_owned(),
                category: None,
            },
        ];
        let ordered = arranged(files, &arrangement);
        assert_eq!(keys(&ordered), ["c", "a", "b", "new"]);
        assert_eq!(
            ordered.first().and_then(|file| file.category.as_deref()),
            Some("swatch")
        );
    }

    #[test]
    fn a_pointer_outside_the_coordinate_range_finds_nothing() {
        assert_eq!(coordinate(120), Some(120.0));
        assert_eq!(coordinate(i32::MAX), None);
    }
}
