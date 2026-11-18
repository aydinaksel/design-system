use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::button::{button_class, button_icon_class, ButtonSize, ButtonTone};
use crate::filter_chip::SELECTED_COUNT_CLASS;
use crate::icons;
use crate::multi_select_list::{
    MultiSelectList, MultiSelectOption, OPTION_END_INSET_CLASS, SCROLLBAR_GUTTER_CLASS,
};
use crate::tooltip::Tooltip;
use crate::typography::LABEL_CLASS;

use super::data_table::{
    BODY_MAX_HEIGHT_CLASS, HEADER_ROW_HEIGHT_CLASS, HEADER_RULE_CLASS, ROW_HEIGHT_CLASS,
};
use super::url_query::UrlTableQuery;
use table_query::LoadedFacetValues;
use table_query::TableQuery;

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

const HEADER_ICON_BUTTON_CLASS: &str =
    "cursor-pointer rounded p-1 text-gray-400 hover:bg-gray-200 hover:text-gray-700";
const CLEAR_ALL_SIZE: ButtonSize = ButtonSize::Dense;

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
    view! { <span class=SELECTED_COUNT_CLASS>{count}</span> }.into_any()
}

fn clear_all_button(query: UrlTableQuery) -> AnyView {
    let button = move || {
        view! {
            <button
                type="button"
                class=button_class(ButtonTone::Subtle, CLEAR_ALL_SIZE)
                on:click=move |_| query.update(TableQuery::clear_all_column_filters)
            >
                {icons::icon(icons::LucideGlyph::X, button_icon_class(CLEAR_ALL_SIZE))}
                "Clear All"
            </button>
        }
    };
    (move || (query.with(TableQuery::filtered_column_count) > 0).then(button)).into_any()
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
        <div class="min-h-0 flex-1 divide-y divide-divider overflow-y-auto overscroll-contain [scrollbar-width:thin]">
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
            "flex {BODY_MAX_HEIGHT_CLASS} flex-col overflow-hidden rounded-xl border border-edge bg-surface"
        );
        let header_class = format!(
            "flex {HEADER_ROW_HEIGHT_CLASS} shrink-0 items-center justify-between gap-2 bg-gray-50 pl-3 pr-2 {HEADER_RULE_CLASS}"
        );
        view! {
            <div class="hidden sm:block sm:w-64 sm:shrink-0">
                <div class=panel_class>
                    <div class=header_class>
                        <h2 class=LABEL_CLASS>"Filters"</h2>
                        <div class="flex items-center gap-2">
                            {clear_all_button(self.query)} <Tooltip label="Hide Filters">
                                <button
                                    type="button"
                                    class=HEADER_ICON_BUTTON_CLASS
                                    aria-label="Hide Filters"
                                    on:click=move |_| self.collapsed.set(true)
                                >
                                    {icons::icon(icons::LucideGlyph::PanelLeftClose, "h-4 w-4")}
                                </button>
                            </Tooltip>
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
                    class="fixed inset-0 z-50 flex flex-col bg-surface sm:hidden"
                    on:keydown=move |event| {
                        if event.key() == "Escape" {
                            sheet_open.set(false);
                        }
                    }
                >
                    <div class="flex shrink-0 items-center justify-between gap-2 border-b border-edge px-4 py-3">
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
                    <div class="shrink-0 border-t border-edge p-4">
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

fn offered_options(facet_values: &LoadedFacetValues, column_key: &str) -> Vec<MultiSelectOption> {
    let Some(Ok(all)) = facet_values else {
        return Vec::new();
    };
    all.iter()
        .find(|values| values.column_key == column_key)
        .map(|values| {
            values
                .values
                .iter()
                .map(|offered| MultiSelectOption {
                    value: offered.value.clone(),
                    label: offered.label().to_string(),
                    row_count: Some(offered.row_count),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[component]
fn ServerFacetGroup(
    facet: ServerFacet,
    query: UrlTableQuery,
    facet_values: Signal<LoadedFacetValues>,
) -> impl IntoView {
    let column_key = facet.column_key;
    let open = RwSignal::new(query.with_untracked(|query| query.is_column_filtered(column_key)));
    let loaded = Memo::new(move |_| facet_values.with(Option::is_some));
    let options =
        Memo::new(move |_| facet_values.with(|values| offered_options(values, column_key)));
    let selected = Signal::derive(move || {
        query.with(|query| {
            query
                .selected_filter_values(column_key)
                .cloned()
                .unwrap_or_default()
        })
    });
    let on_toggle = Callback::new(move |value: String| {
        let every_value: BTreeSet<String> =
            options.with_untracked(|all| all.iter().map(|option| option.value.clone()).collect());
        let mut chosen = selected.get_untracked();
        if !chosen.remove(&value) {
            chosen.insert(value);
        }
        query.update(|query| {
            if chosen.is_empty() || chosen == every_value {
                query.clear_column_filter(column_key);
            } else {
                query.set_column_filter(column_key, chosen);
            }
        });
    });
    let on_clear =
        Callback::new(move |()| query.update(|query| query.clear_column_filter(column_key)));

    let body = move || {
        if !open.get() {
            return ().into_any();
        }
        if !loaded.get() {
            return view! {
                <div class="flex justify-center pb-3 pt-1" role="status" aria-label="Loading">
                    {icons::spinner("h-4 w-4 text-gray-400")}
                </div>
            }
            .into_any();
        }
        view! {
            <div class="pb-1">
                <MultiSelectList
                    options=options
                    selected=selected
                    on_toggle=on_toggle
                    on_clear=on_clear
                    label=facet.label
                />
            </div>
        }
        .into_any()
    };

    view! {
        <div>
            <button
                type="button"
                class=format!(
                    "flex {ROW_HEIGHT_CLASS} w-full cursor-pointer items-center gap-1.5 overflow-hidden pl-2 {OPTION_END_INSET_CLASS} {SCROLLBAR_GUTTER_CLASS} text-left text-sm font-medium text-gray-800 hover:bg-gray-50",
                )
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
                {move || count_badge(selected_count(query, column_key))}
            </button>
            {body}
        </div>
    }
    .into_any()
}

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
