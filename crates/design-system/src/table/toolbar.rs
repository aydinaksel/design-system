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
const TOOLBAR_CLASS: &str = "mb-4 flex flex-col gap-3";
const TABS_CLASS: &str = "border-b border-edge";
const CONTROLS_ROW_CLASS: &str = "flex flex-wrap items-center justify-between gap-2";
const NARROWING_CONTROLS_CLASS: &str = "flex min-w-0 flex-1 flex-wrap items-center gap-2";
const DISPLAY_CONTROLS_CLASS: &str = "ml-auto flex flex-wrap items-center gap-2";

pub(super) struct TableToolbar {
    pub views: Option<&'static [PageView]>,
    pub filter_toggle: Option<AnyView>,
    pub facets: Option<AnyView>,
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
            facets,
            controls,
            ..
        } = self;
        let narrows = filter_toggle.is_some() || search_box.is_some() || facets.is_some();
        let controls_row = (narrows || controls.is_some()).then(|| {
            let display_controls = controls
                .map(|controls| view! { <div class=DISPLAY_CONTROLS_CLASS>{controls}</div> });
            view! {
                <div class=CONTROLS_ROW_CLASS>
                    <div class=NARROWING_CONTROLS_CLASS>{filter_toggle} {search_box} {facets}</div>
                    {display_controls}
                </div>
            }
        });
        let tabs = views.map(|views| {
            view! {
                <div class=TABS_CLASS>
                    <Provider value=TabsOnSharedBorder>
                        <ViewTabs views=views />
                    </Provider>
                </div>
            }
        });
        if tabs.is_none() && controls_row.is_none() {
            return ().into_any();
        }
        view! { <div class=TOOLBAR_CLASS>{tabs} {controls_row}</div> }.into_any()
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
                let searching = RwSignal::new(false);
                Effect::watch(
                    move || pending.get(),
                    move |loading, _, _| {
                        if !loading {
                            searching.set(false);
                        }
                    },
                    false,
                );
                let publish = move |search: String| {
                    let unchanged = query.with_untracked(|query| query.search == search);
                    if !unchanged {
                        searching.set(true);
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
                                pending=searching
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
