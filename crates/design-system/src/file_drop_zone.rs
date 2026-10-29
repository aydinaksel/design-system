use leptos::ev::DragEvent;
use leptos::html;
use leptos::prelude::*;
use web_sys::{File, FileList};

use crate::icons::{self, LucideGlyph};

const IDLE_CLASS: &str = "border-gray-300 bg-white hover:border-gray-400";
const DRAGGING_CLASS: &str = "border-blue-500 bg-blue-50";
const DISABLED_CLASS: &str = "cursor-not-allowed opacity-60";

fn files_in(list: Option<FileList>) -> Vec<File> {
    list.map(|list| {
        (0..list.length())
            .filter_map(|index| list.get(index))
            .collect()
    })
    .unwrap_or_default()
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn FileDropZone(
    #[prop(into)] on_files: Callback<Vec<File>>,
    #[prop(into)] label: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] accept: Option<String>,
    #[prop(optional)] multiple: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let dragging = RwSignal::new(false);
    let input: NodeRef<html::Input> = NodeRef::new();
    let deliver = move |files: Vec<File>| {
        if !files.is_empty() && !disabled.get_untracked() {
            on_files.run(files);
        }
    };
    let zone_class = move || {
        let state = if disabled.get() {
            DISABLED_CLASS
        } else if dragging.get() {
            DRAGGING_CLASS
        } else {
            IDLE_CLASS
        };
        format!(
            "flex cursor-pointer flex-col items-center gap-1 rounded-lg border-2 border-dashed px-6 py-6 text-center text-sm transition-colors focus-within:ring-2 focus-within:ring-blue-500 {state}"
        )
    };
    let description_view = description
        .map(|description| view! { <span class="text-xs text-gray-500">{description}</span> });

    view! {
        <div class="space-y-3">
            <label
                class=zone_class
                on:dragover=move |event: DragEvent| {
                    event.prevent_default();
                    dragging.set(true);
                }
                on:dragleave=move |_| dragging.set(false)
                on:drop=move |event: DragEvent| {
                    event.prevent_default();
                    dragging.set(false);
                    deliver(files_in(event.data_transfer().and_then(|transfer| transfer.files())));
                }
            >
                {icons::icon(LucideGlyph::Upload, "h-5 w-5 text-gray-400")}
                <span class="font-medium text-gray-700">{label}</span>
                {description_view}
                <input
                    node_ref=input
                    type="file"
                    class="sr-only"
                    accept=accept
                    multiple=multiple
                    prop:disabled=move || disabled.get()
                    on:change=move |_| {
                        let Some(element) = input.get() else {
                            return;
                        };
                        deliver(files_in(element.files()));
                        element.set_value("");
                    }
                />
            </label>
            {children.map(|children| children())}
        </div>
    }
    .into_any()
}
