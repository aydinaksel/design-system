use leptos::prelude::*;

use crate::icons;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GalleryImage {
    pub label: String,
    pub thumbnail_url: String,
    pub display_url: String,
    pub source_set: String,
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ImageGallery(
    images: Signal<Vec<GalleryImage>>,
    selected_image: RwSignal<usize>,
) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let handler = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |event: web_sys::KeyboardEvent| match event.key().as_str() {
                "ArrowLeft" => {
                    let current = selected_image.get_untracked();
                    if current > 0 {
                        selected_image.set(current.saturating_sub(1));
                    }
                }
                "ArrowRight" => {
                    let current = selected_image.get_untracked();
                    let count = images.get_untracked().len();
                    if current.saturating_add(1) < count {
                        selected_image.set(current.saturating_add(1));
                    }
                }
                _ => {}
            },
        );
        if let Some(window) = web_sys::window() {
            #[expect(
                clippy::let_underscore_must_use,
                reason = "without the listener the shortcut is inert, which is the pre-existing behaviour"
            )]
            let _ = window
                .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
        }
        handler.forget();
    }

    view! {
        <div>
            {move || {
                let views = images.get();
                if views.is_empty() {
                    return empty_gallery();
                }
                let selected = selected_image.get();
                let current_index = if selected < views.len() { selected } else { 0 };
                let Some(current) = views.get(current_index).cloned() else {
                    return ().into_any();
                };
                view! {
                    <div>
                        <GalleryStage
                            image=current
                            position=current_index
                            count=views.len()
                            selected_image=selected_image
                        />
                        <GalleryThumbnails
                            images=views
                            current_index=current_index
                            selected_image=selected_image
                        />
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
    .into_any()
}

fn empty_gallery() -> AnyView {
    view! {
        <div class="aspect-square w-full rounded-lg border border-gray-200 bg-gray-100 flex items-center justify-center">
            <span class="text-sm text-gray-400">"No images"</span>
        </div>
    }
    .into_any()
}

#[component]
fn GalleryStage(
    image: GalleryImage,
    position: usize,
    count: usize,
    selected_image: RwSignal<usize>,
) -> impl IntoView {
    let can_go_previous = position > 0;
    let can_go_next = position.saturating_add(1) < count;
    let caption = format!(
        "{} ({} / {})",
        image.label,
        position.saturating_add(1),
        count
    );

    view! {
        <div class="relative aspect-square w-full rounded-lg border border-gray-200 overflow-hidden bg-white">
            <img
                src=image.display_url
                srcset=image.source_set
                sizes="(min-width: 1024px) 32rem, 100vw"
                class="h-full w-full object-contain"
            />
            <Show when=move || { count > 1 }>
                <div>
                    <button
                        class="absolute left-2 top-1/2 -translate-y-1/2 rounded-full bg-white/80 p-2 shadow hover:bg-white disabled:opacity-30 disabled:cursor-not-allowed"
                        disabled=!can_go_previous
                        on:click=move |_| selected_image.set(position.saturating_sub(1))
                    >
                        {icons::icon(icons::LucideGlyph::ChevronLeft, "h-5 w-5")}
                    </button>
                    <button
                        class="absolute right-2 top-1/2 -translate-y-1/2 rounded-full bg-white/80 p-2 shadow hover:bg-white disabled:opacity-30 disabled:cursor-not-allowed"
                        disabled=!can_go_next
                        on:click=move |_| selected_image.set(position.saturating_add(1))
                    >
                        {icons::icon(icons::LucideGlyph::ChevronRight, "h-5 w-5")}
                    </button>
                    <div class="absolute bottom-3 left-1/2 -translate-x-1/2 rounded-full bg-black/50 px-3 py-1 text-xs text-white">
                        {caption.clone()}
                    </div>
                </div>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn GalleryThumbnails(
    images: Vec<GalleryImage>,
    current_index: usize,
    selected_image: RwSignal<usize>,
) -> impl IntoView {
    if images.len() <= 1 {
        return ().into_any();
    }
    let thumbnails = images
        .into_iter()
        .enumerate()
        .map(|(index, image)| {
            let border_class = if index == current_index {
                "ring-2 ring-blue-500"
            } else {
                "ring-1 ring-gray-200 hover:ring-gray-400"
            };
            view! {
                <button
                    class=format!(
                        "shrink-0 w-16 h-16 rounded overflow-hidden bg-white cursor-pointer {border_class}",
                    )
                    title=image.label
                    on:click=move |_| selected_image.set(index)
                >
                    <img
                        src=image.thumbnail_url
                        class="h-full w-full object-cover"
                        loading="lazy"
                    />
                </button>
            }
                .into_any()
        })
        .collect::<Vec<AnyView>>();

    view! { <div class="mt-3 flex gap-2 overflow-x-auto pb-1">{thumbnails}</div> }.into_any()
}

fn step_backward(position: Option<usize>) -> Option<usize> {
    position.map(|index| index.saturating_sub(1))
}

fn step_forward(position: Option<usize>, count: usize) -> Option<usize> {
    position.map(|index| index.saturating_add(1).min(count.saturating_sub(1)))
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ImageGrid(#[prop(into)] images: Signal<Vec<GalleryImage>>) -> impl IntoView {
    let enlarged: RwSignal<Option<usize>> = RwSignal::new(None);

    let keyboard = window_event_listener(leptos::ev::keydown, move |event| {
        if enlarged.get_untracked().is_none() {
            return;
        }
        let count = images.with_untracked(Vec::len);
        match event.key().as_str() {
            "Escape" => enlarged.set(None),
            "ArrowLeft" => enlarged.update(|position| *position = step_backward(*position)),
            "ArrowRight" => enlarged.update(|position| *position = step_forward(*position, count)),
            _ => {}
        }
    });
    on_cleanup(move || keyboard.remove());

    view! {
        {move || {
            let views = images.get();
            if views.is_empty() {
                return view! { <p class="py-8 text-center text-sm text-gray-500">"No images"</p> }
                    .into_any();
            }
            view! {
                <ul class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-6">
                    {views
                        .into_iter()
                        .enumerate()
                        .map(|(index, image)| {
                            view! {
                                <li>
                                    <button
                                        type="button"
                                        class="group block w-full cursor-pointer text-left"
                                        on:click=move |_| enlarged.set(Some(index))
                                    >
                                        <div class="aspect-square overflow-hidden rounded-lg border border-gray-200 bg-white group-hover:border-gray-400">
                                            <img
                                                src=image.display_url
                                                srcset=image.source_set
                                                sizes="(min-width: 1536px) 16vw, (min-width: 1024px) 25vw, (min-width: 640px) 33vw, 50vw"
                                                alt=image.label.clone()
                                                class="h-full w-full object-contain"
                                                loading="lazy"
                                            />
                                        </div>
                                        <p class="mt-1.5 truncate text-sm text-gray-600">
                                            {image.label}
                                        </p>
                                    </button>
                                </li>
                            }
                        })
                        .collect::<Vec<_>>()}
                </ul>
            }
                .into_any()
        }}
        {move || {
            let position = enlarged.get()?;
            let views = images.get();
            let image = views.get(position).cloned()?;
            Some(
                view! {
                    <EnlargedImage
                        image=image
                        position=position
                        count=views.len()
                        enlarged=enlarged
                    />
                },
            )
        }}
    }
    .into_any()
}

#[component]
fn EnlargedImage(
    image: GalleryImage,
    position: usize,
    count: usize,
    enlarged: RwSignal<Option<usize>>,
) -> impl IntoView {
    let can_go_previous = position > 0;
    let can_go_next = position.saturating_add(1) < count;
    let caption = format!(
        "{} ({} / {})",
        image.label,
        position.saturating_add(1),
        count
    );
    let dialog_label = image.label.clone();
    let navigation_class = "absolute top-1/2 -translate-y-1/2 cursor-pointer rounded-full bg-white/80 p-2 shadow hover:bg-white disabled:cursor-not-allowed disabled:opacity-30";

    view! {
        <div
            role="dialog"
            aria-modal="true"
            aria-label=dialog_label
            class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900/90 p-4 sm:p-12"
            on:click=move |_| enlarged.set(None)
        >
            <img
                src=image.display_url
                srcset=image.source_set
                sizes="100vw"
                alt=image.label
                class="max-h-full max-w-full object-contain"
                on:click=|event| event.stop_propagation()
            />
            <button
                type="button"
                class="absolute right-4 top-4 cursor-pointer rounded-full bg-white/80 p-2 shadow hover:bg-white"
                aria-label="Close"
                title="Close"
                on:click=move |_| enlarged.set(None)
            >
                {icons::icon(icons::LucideGlyph::X, "h-5 w-5")}
            </button>
            <button
                type="button"
                class=format!("left-4 {navigation_class}")
                aria-label="Previous Image"
                disabled=!can_go_previous
                on:click=move |event| {
                    event.stop_propagation();
                    enlarged.set(Some(position.saturating_sub(1)));
                }
            >
                {icons::icon(icons::LucideGlyph::ChevronLeft, "h-5 w-5")}
            </button>
            <button
                type="button"
                class=format!("right-4 {navigation_class}")
                aria-label="Next Image"
                disabled=!can_go_next
                on:click=move |event| {
                    event.stop_propagation();
                    enlarged.set(Some(position.saturating_add(1)));
                }
            >
                {icons::icon(icons::LucideGlyph::ChevronRight, "h-5 w-5")}
            </button>
            <div class="absolute bottom-4 left-1/2 -translate-x-1/2 rounded-full bg-black/50 px-3 py-1 text-sm text-white">
                {caption}
            </div>
        </div>
    }
    .into_any()
}
