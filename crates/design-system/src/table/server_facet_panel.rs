use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::icons;
use crate::search_input::{SearchInput, SearchSize};
use crate::typography::LABEL_CLASS;

use super::data_table::BODY_MAX_HEIGHT_CLASS;
use super::filter_option::FilterOption;
use super::url_query::UrlTableQuery;
use table_query::TableQuery;
use table_query::{FilterValue, LoadedFacetValues};

/// One facet in the left rail, naming the column the server counts.
#[derive(Clone, Copy)]
pub struct ServerFacet {
    pub column_key: &'static str,
    pub label: &'static str,
}

impl ServerFacet {
    #[must_use]
    pub const fn new(column_key: &'static str, label: &'static str) -> Self {
        Self { column_key, label }
    }
}

const SEARCH_VISIBILITY_THRESHOLD: usize = 10;
const COUNT_BADGE_CLASS: &str =
    "rounded-full bg-blue-100 px-1.5 py-0.5 text-xs font-medium text-blue-700 tabular-nums";
const HEADER_ICON_BUTTON_CLASS: &str =
    "cursor-pointer rounded p-1 text-gray-400 hover:bg-gray-200 hover:text-gray-700";

fn selected_count(query: UrlTableQuery, column_key: &str) -> usize {
    query.with(|query| {
        query
            .selected_filter_values(column_key)
            .map_or(0, BTreeSet::len)
    })
}

fn count_badge(count: usize) -> AnyView {
    if count == 0 {
        return ().into_any();
    }
    view! { <span class=COUNT_BADGE_CLASS>{count}</span> }.into_any()
}

fn clear_all_button(query: UrlTableQuery) -> AnyView {
    view! {
        <button
            type="button"
            class=move || {
                if query.with(|query| query.filtered_column_count() == 0) {
                    "hidden"
                } else {
                    "cursor-pointer text-xs text-blue-600 hover:underline"
                }
            }
            on:click=move |_| query.update(TableQuery::clear_all_column_filters)
        >
            "Clear All"
        </button>
    }
    .into_any()
}

fn facet_groups(
    facets: StoredValue<Vec<ServerFacet>>,
    query: UrlTableQuery,
    facet_values: Signal<LoadedFacetValues>,
) -> AnyView {
    let groups = move || {
        facets.with_value(|all| {
            all.iter()
                .map(|facet| {
                    view! { <ServerFacetGroup facet=*facet query=query facet_values=facet_values /> }
                })
                .collect_view()
        })
    };
    view! {
        <div class="min-h-0 flex-1 divide-y divide-gray-100 overflow-y-auto overscroll-contain [scrollbar-width:thin]">
            {groups}
        </div>
    }
    .into_any()
}

#[cfg(feature = "hydrate")]
fn lock_page_scroll(locked: bool) {
    const PHONE_SCROLL_LOCK_CLASS: &str = "max-sm:overflow-hidden";
    let Some(root) = document().document_element() else {
        return;
    };
    let class_list = root.class_list();
    let outcome = if locked {
        class_list.add_1(PHONE_SCROLL_LOCK_CLASS)
    } else {
        class_list.remove_1(PHONE_SCROLL_LOCK_CLASS)
    };
    outcome.unwrap_or_default();
}

#[cfg(not(feature = "hydrate"))]
fn lock_page_scroll(_locked: bool) {}

#[cfg(feature = "hydrate")]
fn narrow_viewport() -> bool {
    const PHONE_BREAKPOINT: &str = "(max-width: 639px)";
    web_sys::window()
        .and_then(|window| window.match_media(PHONE_BREAKPOINT).ok().flatten())
        .is_some_and(|query| query.matches())
}

#[cfg(not(feature = "hydrate"))]
fn narrow_viewport() -> bool {
    false
}

/// The facet filters of one table or calendar: a rail beside it on wide
/// screens, a full-screen sheet on phones, and the toolbar button that brings
/// either back.
#[derive(Clone, Copy)]
pub struct FacetControls {
    facets: StoredValue<Vec<ServerFacet>>,
    query: UrlTableQuery,
    facet_values: Signal<LoadedFacetValues>,
    matching_rows: Option<Signal<Option<u64>>>,
    collapsed: RwSignal<bool>,
    sheet_open: RwSignal<bool>,
}

impl FacetControls {
    /// `matching_rows` labels the phone sheet's done button with the result
    /// count.
    #[must_use]
    pub fn new(
        facets: Vec<ServerFacet>,
        query: UrlTableQuery,
        facet_values: Signal<LoadedFacetValues>,
        matching_rows: Option<Signal<Option<u64>>>,
    ) -> Self {
        Self {
            facets: StoredValue::new(facets),
            query,
            facet_values,
            matching_rows,
            collapsed: RwSignal::new(false),
            sheet_open: RwSignal::new(false),
        }
    }

    fn groups(self) -> AnyView {
        facet_groups(self.facets, self.query, self.facet_values)
    }

    fn active_filter_count(self) -> usize {
        self.query.with(TableQuery::filtered_column_count)
    }

    /// Shown on phones always, and on wide screens while the rail is hidden.
    #[must_use]
    pub fn toggle_button(self, extra_class: &'static str) -> AnyView {
        let wrapper_class = move || {
            let visibility = if self.collapsed.get() {
                "flex"
            } else {
                "flex sm:hidden"
            };
            format!("{visibility} {extra_class}")
        };
        view! {
            <div class=wrapper_class>
                <button
                    type="button"
                    class=button_class(ButtonTone::Secondary, ButtonSize::Standard)
                    aria-haspopup="dialog"
                    on:click=move |_| {
                        if narrow_viewport() {
                            self.sheet_open.set(true);
                        } else {
                            self.collapsed.set(false);
                        }
                    }
                >
                    {icons::icon(icons::LucideGlyph::Funnel, "h-4 w-4 shrink-0 opacity-70")}
                    "Filters"
                    {move || count_badge(self.active_filter_count())}
                </button>
            </div>
        }
        .into_any()
    }

    /// The rail and the phone sheet, for the start of the row the table or
    /// calendar sits in.
    #[must_use]
    pub fn rail(self) -> AnyView {
        view! {
            {move || if self.collapsed.get() { ().into_any() } else { self.expanded_rail() }}
            {self.phone_sheet()}
        }
        .into_any()
    }

    fn expanded_rail(self) -> AnyView {
        let panel_class = format!(
            "flex {BODY_MAX_HEIGHT_CLASS} flex-col overflow-hidden rounded-lg border border-gray-200 bg-white"
        );
        view! {
            <div class="hidden sm:block sm:w-64 sm:shrink-0">
                <div class=panel_class>
                    <div class="flex shrink-0 items-center justify-between gap-2 border-b border-gray-200 bg-gray-50 py-2.5 pl-3 pr-2">
                        <h2 class=LABEL_CLASS>"Filters"</h2>
                        <div class="flex items-center gap-2">
                            {clear_all_button(self.query)}
                            <button
                                type="button"
                                class=HEADER_ICON_BUTTON_CLASS
                                title="Hide Filters"
                                aria-label="Hide Filters"
                                on:click=move |_| self.collapsed.set(true)
                            >
                                {icons::icon(icons::LucideGlyph::X, "h-4 w-4")}
                            </button>
                        </div>
                    </div>
                    {self.groups()}
                </div>
            </div>
        }
        .into_any()
    }

    fn phone_sheet(self) -> AnyView {
        let sheet_open = self.sheet_open;
        Effect::new(move |_| lock_page_scroll(sheet_open.get()));
        on_cleanup(move || lock_page_scroll(false));

        let close_button: NodeRef<leptos::html::Button> = NodeRef::new();
        Effect::new(move |_| {
            if let Some(button) = close_button.get() {
                button.focus().unwrap_or_default();
            }
        });

        let show_results_label = move || match self.matching_rows.and_then(|rows| rows.get()) {
            Some(count) => format!("Show {count} results"),
            None => "Show Results".to_string(),
        };

        (move || {
            if !sheet_open.get() {
                return ().into_any();
            }
            view! {
                <div
                    role="dialog"
                    aria-modal="true"
                    aria-label="Filters"
                    class="fixed inset-0 z-50 flex flex-col bg-white sm:hidden"
                    on:keydown=move |event| {
                        if event.key() == "Escape" {
                            sheet_open.set(false);
                        }
                    }
                >
                    <div class="flex shrink-0 items-center justify-between gap-2 border-b border-gray-200 px-4 py-3">
                        <h2 class="text-base font-semibold text-gray-900">"Filters"</h2>
                        <div class="flex items-center gap-3">
                            {clear_all_button(self.query)}
                            <button
                                type="button"
                                node_ref=close_button
                                class="cursor-pointer rounded p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-800"
                                aria-label="Close Filters"
                                on:click=move |_| sheet_open.set(false)
                            >
                                {icons::icon(icons::LucideGlyph::X, "h-5 w-5")}
                            </button>
                        </div>
                    </div>
                    {self.groups()}
                    <div class="shrink-0 border-t border-gray-200 p-4">
                        <button
                            type="button"
                            class=format!(
                                "{} w-full",
                                button_class(ButtonTone::Primary, ButtonSize::Standard),
                            )
                            on:click=move |_| sheet_open.set(false)
                        >
                            {show_results_label}
                        </button>
                    </div>
                </div>
            }
            .into_any()
        })
        .into_any()
    }
}

#[component]
fn ServerFacetGroup(
    facet: ServerFacet,
    query: UrlTableQuery,
    facet_values: Signal<LoadedFacetValues>,
) -> impl IntoView {
    let open =
        RwSignal::new(query.with_untracked(|query| query.is_column_filtered(facet.column_key)));
    let value_search = RwSignal::new(String::new());

    let offered = move || -> Vec<FilterValue> {
        match facet_values.get() {
            Some(Ok(all)) => all
                .into_iter()
                .find(|values| values.column_key == facet.column_key)
                .map(|values| values.values)
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    };

    let value_list = move || {
        let all_values: BTreeSet<String> = offered().into_iter().map(|value| value.value).collect();
        let search_lower = value_search.get().to_lowercase();
        offered()
            .into_iter()
            .filter(|value| {
                search_lower.is_empty() || value.label().to_lowercase().contains(&search_lower)
            })
            .map(|offered_value| {
                let value = offered_value.value.clone();
                let label = offered_value.label().to_string();
                let row_count = offered_value.row_count;
                let checked = query.with(|query| {
                    query
                        .selected_filter_values(facet.column_key)
                        .is_some_and(|set| set.contains(&value))
                });
                let all_values = all_values.clone();
                let on_toggle = move |()| {
                    let mut selected = query.with_untracked(|query| {
                        query
                            .selected_filter_values(facet.column_key)
                            .cloned()
                            .unwrap_or_default()
                    });
                    if !selected.remove(&value) {
                        selected.insert(value.clone());
                    }
                    query.update(|query| {
                        if selected.is_empty() || selected == all_values {
                            query.clear_column_filter(facet.column_key);
                        } else {
                            query.set_column_filter(facet.column_key, selected);
                        }
                    });
                };
                view! { <FilterOption label=label row_count=row_count checked=checked on_toggle=on_toggle /> }
            })
            .collect_view()
    };

    let body = move || {
        if !open.get() {
            return ().into_any();
        }
        if facet_values.get().is_none() {
            return view! {
                <div class="flex justify-center pb-3 pt-1" role="status" aria-label="Loading">
                    {icons::spinner("h-4 w-4 text-gray-400")}
                </div>
            }
            .into_any();
        }
        let search_box = if offered().len() > SEARCH_VISIBILITY_THRESHOLD {
            view! {
                <div class="px-1.5 pb-1.5">
                    <SearchInput value=value_search size=SearchSize::Compact />
                </div>
            }
            .into_any()
        } else {
            ().into_any()
        };
        view! {
            <div class="pb-2">
                {search_box}
                <div class="max-h-56 overflow-y-auto overscroll-contain px-1.5 [scrollbar-width:thin]">
                    {value_list}
                </div>
            </div>
        }
        .into_any()
    };

    view! {
        <div>
            <button
                type="button"
                class="flex w-full cursor-pointer items-center gap-1.5 px-2 py-2 text-left text-sm font-medium text-gray-800 hover:bg-gray-50"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| open.update(|is_open| *is_open = !*is_open)
            >
                {move || {
                    let rotation = if open.get() { "rotate-90" } else { "" };
                    icons::icon(
                        icons::LucideGlyph::ChevronRight,
                        format!("h-4 w-4 shrink-0 text-gray-400 transition-transform {rotation}"),
                    )
                }}
                <span class="min-w-0 flex-1 truncate">{facet.label}</span>
                {move || count_badge(selected_count(query, facet.column_key))}
            </button>
            {body}
        </div>
    }
    .into_any()
}

/// Keeps a facet rail's values in step with the table's search and filters.
pub fn facet_values_loader<LoadFn, LoadFuture, LoadError>(
    load: LoadFn,
    query: UrlTableQuery,
) -> Signal<LoadedFacetValues>
where
    LoadFn: Fn(TableQuery) -> LoadFuture + Clone + Send + Sync + 'static,
    LoadFuture: std::future::Future<Output = Result<Vec<table_query::FacetValues>, LoadError>>
        + Send
        + 'static,
    LoadError: ToString,
{
    let values = LocalResource::new(move || {
        let load = load.clone();
        let filters = query.with(TableQuery::filters_only);
        async move { load(filters).await.map_err(|error| error.to_string()) }
    });
    let latest: RwSignal<LoadedFacetValues> = RwSignal::new(None);
    Effect::new(move |_| {
        latest.set(values.try_get().flatten());
    });
    latest.into()
}
