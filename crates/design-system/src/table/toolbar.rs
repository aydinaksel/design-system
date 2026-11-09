use leptos::context::Provider;
use leptos::prelude::*;

use leptos_use_fn_debounce_throttle::use_debounce_fn_with_arg;

use crate::search_input::SearchInput;
use crate::tabs::{PageView, TabsOnSharedBorder, ViewTabs};

use super::data_table::Search;
use super::url_query::UrlTableQuery;

const DEFAULT_SEARCH_PLACEHOLDER: &str = "Search";
const TYPING_PAUSE_MILLISECONDS: f64 = 300.0;
const SEARCH_BOX_CLASS: &str = "min-w-0 flex-1 sm:w-80 sm:flex-none";
const ROW_WITH_VIEWS_CLASS: &str = "mb-4 flex flex-wrap-reverse items-start justify-between gap-x-6 gap-y-2 border-b border-gray-200";
const ROW_WITHOUT_VIEWS_CLASS: &str = "mb-4 flex";
const CONTROLS_CLASS: &str =
    "flex w-full flex-wrap items-center justify-end gap-2 sm:ml-auto sm:w-auto";
const CONTROLS_ABOVE_SHARED_BORDER_CLASS: &str =
    "flex w-full flex-wrap items-center justify-end gap-2 pb-2 sm:ml-auto sm:w-auto";

pub(super) struct TableToolbar {
    pub views: Option<&'static [PageView]>,
    pub filter_toggle: Option<AnyView>,
    pub controls: Option<AnyView>,
    pub search: Search,
    pub search_placeholder: String,
    pub query: Option<UrlTableQuery>,
    pub search_text: RwSignal<String>,
    pub pending: Signal<bool>,
}

impl TableToolbar {
    pub(super) fn render(self) -> AnyView {
        let search_box = self.search_box();
        let Self {
            views,
            filter_toggle,
            controls,
            ..
        } = self;
        if views.is_none() && filter_toggle.is_none() && controls.is_none() && search_box.is_none()
        {
            return ().into_any();
        }
        match views {
            Some(views) => view! {
                <div class=ROW_WITH_VIEWS_CLASS>
                    <Provider value=TabsOnSharedBorder>
                        <ViewTabs views=views />
                    </Provider>
                    <div class=CONTROLS_ABOVE_SHARED_BORDER_CLASS>
                        {filter_toggle} {search_box} {controls}
                    </div>
                </div>
            }
            .into_any(),
            None => view! {
                <div class=ROW_WITHOUT_VIEWS_CLASS>
                    <div class=CONTROLS_CLASS>{filter_toggle} {search_box} {controls}</div>
                </div>
            }
            .into_any(),
        }
    }

    fn placeholder(&self) -> String {
        if self.search_placeholder.is_empty() {
            DEFAULT_SEARCH_PLACEHOLDER.to_string()
        } else {
            self.search_placeholder.clone()
        }
    }

    fn search_box(&self) -> Option<AnyView> {
        let placeholder = self.placeholder();
        let search_text = self.search_text;
        let pending = self.pending;
        match (self.search, self.query) {
            (Search::Off, _) => None,
            (_, None) => Some(
                view! {
                    <div class=SEARCH_BOX_CLASS>
                        <SearchInput value=search_text placeholder=placeholder />
                    </div>
                }
                .into_any(),
            ),
            (_, Some(query)) => {
                let publish = move |search: String| {
                    let unchanged = query.with_untracked(|query| query.search == search);
                    if !unchanged {
                        query.update(|query| query.set_search(search));
                    }
                };
                let publish_after_typing_pauses =
                    use_debounce_fn_with_arg(publish, TYPING_PAUSE_MILLISECONDS);
                Effect::watch(
                    move || search_text.get(),
                    move |search, _, _| {
                        publish_after_typing_pauses(search.clone());
                    },
                    false,
                );
                Some(
                    view! {
                        <div class=SEARCH_BOX_CLASS>
                            <SearchInput
                                value=search_text
                                placeholder=placeholder
                                pending=pending
                                on_submit=Callback::new(move |()| {
                                    publish(search_text.get_untracked());
                                })
                                on_clear=Callback::new(move |()| publish(String::new()))
                                applied=Signal::derive(move || {
                                    query.with(|query| !query.search.is_empty())
                                })
                            />
                        </div>
                    }
                    .into_any(),
                )
            }
        }
    }
}
