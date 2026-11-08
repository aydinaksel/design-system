use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use leptos::ev::{Event, KeyboardEvent, MouseEvent};
use leptos::html::Input;
use leptos::prelude::*;

use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::controls::{field_control_id, field_description_id, with_field_error, CONTROL_CLASS};
use crate::element_key::ElementKey;
use crate::icons;
use crate::listbox::{LISTBOX_ANCHORING, LISTBOX_PANEL_CLASS, LISTBOX_PANEL_MAXIMUM_HEIGHT_PIXELS};
use crate::menu::{moved_position, Movement};
use crate::placement::{AnchoredPanel, Anchoring};
use crate::tooltip::Tooltip;

const ANCHORING: Anchoring = Anchoring {
    reserved_height_pixels: LISTBOX_PANEL_MAXIMUM_HEIGHT_PIXELS,
    ..LISTBOX_ANCHORING
};
const DEBOUNCE_MILLISECONDS: f64 = 300.0;
const MINIMUM_SEARCH_CHARS: usize = 2;
const OPTION_CLASS: &str =
    "flex cursor-pointer items-baseline justify-between gap-4 rounded-item px-3 py-1.5 text-gray-700";
const HIGHLIGHTED_OPTION_CLASS: &str = "flex cursor-pointer items-baseline justify-between gap-4 rounded-item bg-gray-100 px-3 py-1.5 text-gray-900";
const MESSAGE_CLASS: &str = "px-3 py-1.5 text-gray-500";
const GROUP_HEADING_CLASS: &str = "px-3 pb-1 pt-1.5 text-xs font-medium text-gray-500";
const GROUP_SEPARATOR_CLASS: &str = "my-1 h-px bg-gray-200";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxOption {
    pub value: String,
    pub label: String,
    pub detail: Option<String>,
}

impl ComboboxOption {
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            detail: None,
        }
    }

    #[must_use]
    pub fn with_detail(self, detail: impl Into<String>) -> Self {
        Self {
            detail: Some(detail.into()),
            ..self
        }
    }

    #[must_use]
    pub fn display(&self) -> String {
        selection_display(&self.value, &self.label)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxGroup {
    heading: Option<String>,
    options: Vec<ComboboxOption>,
}

impl ComboboxGroup {
    #[must_use]
    pub fn new(options: Vec<ComboboxOption>) -> Self {
        Self {
            heading: None,
            options,
        }
    }

    #[must_use]
    pub fn with_heading(self, heading: impl Into<String>) -> Self {
        Self {
            heading: Some(heading.into()),
            ..self
        }
    }

    fn matching(&self, search: &str, display: ComboboxDisplay) -> Self {
        Self {
            heading: self.heading.clone(),
            options: matching_options(&self.options, search, display),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComboboxDisplay {
    #[default]
    ValueAndLabel,
    Label,
}

impl ComboboxDisplay {
    fn selection(self, value: &str, label: &str) -> String {
        match self {
            Self::ValueAndLabel => selection_display(value, label),
            Self::Label if value.is_empty() => String::new(),
            Self::Label => label.to_owned(),
        }
    }

    fn shows_value(self) -> bool {
        matches!(self, Self::ValueAndLabel)
    }
}

fn selection_display(value: &str, label: &str) -> String {
    if value.is_empty() {
        String::new()
    } else {
        format!("{value} - {label}")
    }
}

type OptionsFuture = Pin<Box<dyn Future<Output = Result<Vec<ComboboxOption>, String>> + Send>>;
type SearchLoader = Arc<dyn Fn(String) -> OptionsFuture + Send + Sync>;
type SuggestionsLoader = Arc<dyn Fn() -> OptionsFuture + Send + Sync>;

fn boxed_options<LoadFuture, LoadError>(pending: LoadFuture) -> OptionsFuture
where
    LoadFuture: Future<Output = Result<Vec<ComboboxOption>, LoadError>> + Send + 'static,
    LoadError: ToString,
{
    Box::pin(async move { pending.await.map_err(|error| error.to_string()) })
}

struct Suggestions {
    heading: String,
    load: SuggestionsLoader,
}

pub struct ComboboxSearch {
    search: SearchLoader,
    suggestions: Option<Suggestions>,
}

impl ComboboxSearch {
    pub fn new<SearchFn, SearchFuture, SearchError>(search: SearchFn) -> Self
    where
        SearchFn: Fn(String) -> SearchFuture + Send + Sync + 'static,
        SearchFuture: Future<Output = Result<Vec<ComboboxOption>, SearchError>> + Send + 'static,
        SearchError: ToString,
    {
        Self {
            search: Arc::new(move |text| boxed_options(search(text))),
            suggestions: None,
        }
    }

    #[must_use]
    pub fn with_suggestions<SuggestFn, SuggestFuture, SuggestError>(
        self,
        heading: impl Into<String>,
        suggest: SuggestFn,
    ) -> Self
    where
        SuggestFn: Fn() -> SuggestFuture + Send + Sync + 'static,
        SuggestFuture: Future<Output = Result<Vec<ComboboxOption>, SuggestError>> + Send + 'static,
        SuggestError: ToString,
    {
        Self {
            suggestions: Some(Suggestions {
                heading: heading.into(),
                load: Arc::new(move || boxed_options(suggest())),
            }),
            ..self
        }
    }

    fn lookup(&self, text: &str) -> Lookup {
        let length = text.trim().chars().count();
        if length == 0 {
            return self
                .suggestions
                .as_ref()
                .map_or(Lookup::Nothing, |suggestions| Lookup::Pending {
                    pending: (suggestions.load)(),
                    suggesting: true,
                });
        }
        if length < MINIMUM_SEARCH_CHARS {
            return Lookup::Nothing;
        }
        Lookup::Pending {
            pending: (self.search)(text.to_owned()),
            suggesting: false,
        }
    }
}

pub enum ComboboxSource {
    List(Vec<ComboboxOption>),
    Groups(Vec<ComboboxGroup>),
    Search(ComboboxSearch),
}

impl From<ComboboxSearch> for ComboboxSource {
    fn from(search: ComboboxSearch) -> Self {
        Self::Search(search)
    }
}

impl From<Vec<ComboboxOption>> for ComboboxSource {
    fn from(options: Vec<ComboboxOption>) -> Self {
        Self::List(options)
    }
}

impl From<Vec<ComboboxGroup>> for ComboboxSource {
    fn from(groups: Vec<ComboboxGroup>) -> Self {
        Self::Groups(groups)
    }
}

impl ComboboxSource {
    fn lookup(&self, text: &str, display: ComboboxDisplay) -> Lookup {
        match self {
            Self::List(options) => Lookup::Ready(vec![ComboboxGroup::new(matching_options(
                options, text, display,
            ))]),
            Self::Groups(groups) => Lookup::Ready(
                groups
                    .iter()
                    .map(|group| group.matching(text, display))
                    .filter(|group| !group.options.is_empty())
                    .collect(),
            ),
            Self::Search(search) => search.lookup(text),
        }
    }

    fn opens_on_focus(&self) -> bool {
        match self {
            Self::List(_) | Self::Groups(_) => true,
            Self::Search(search) => search.suggestions.is_some(),
        }
    }

    fn filters_locally(&self) -> bool {
        matches!(self, Self::List(_) | Self::Groups(_))
    }

    fn suggestions_heading(&self) -> Option<String> {
        match self {
            Self::List(_) | Self::Groups(_) => None,
            Self::Search(search) => search
                .suggestions
                .as_ref()
                .map(|suggestions| suggestions.heading.clone()),
        }
    }
}

enum Lookup {
    Nothing,
    Ready(Vec<ComboboxGroup>),
    Pending {
        pending: OptionsFuture,
        suggesting: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MatchStrength {
    ExactValue,
    Prefix,
    Contained,
}

fn match_strength(
    option: &ComboboxOption,
    needle: &str,
    display: ComboboxDisplay,
) -> Option<MatchStrength> {
    let value = if display.shows_value() {
        option.value.to_lowercase()
    } else {
        String::new()
    };
    let label = option.label.to_lowercase();
    if !value.is_empty() && value == needle {
        Some(MatchStrength::ExactValue)
    } else if (!value.is_empty() && value.starts_with(needle)) || label.starts_with(needle) {
        Some(MatchStrength::Prefix)
    } else if value.contains(needle) || label.contains(needle) {
        Some(MatchStrength::Contained)
    } else {
        None
    }
}

fn matching_options(
    options: &[ComboboxOption],
    search: &str,
    display: ComboboxDisplay,
) -> Vec<ComboboxOption> {
    let needle = search.trim().to_lowercase();
    if needle.is_empty() {
        return options.to_vec();
    }
    let mut matches: Vec<(MatchStrength, &ComboboxOption)> = options
        .iter()
        .filter_map(|option| {
            match_strength(option, &needle, display).map(|strength| (strength, option))
        })
        .collect();
    matches.sort_by_key(|(strength, _)| *strength);
    matches
        .into_iter()
        .map(|(_, option)| option.clone())
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Results {
    Idle,
    Searching,
    Loaded(Vec<ComboboxGroup>),
}

fn lacks_options(groups: &[ComboboxGroup]) -> bool {
    groups.iter().all(|group| group.options.is_empty())
}

fn grouped_options(groups: &[ComboboxGroup]) -> Vec<ComboboxOption> {
    groups
        .iter()
        .flat_map(|group| group.options.iter().cloned())
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Abandoned {
    ClearSelection,
    RestoreSelection,
}

fn abandoned(query: &str) -> Abandoned {
    if query.trim().is_empty() {
        Abandoned::ClearSelection
    } else {
        Abandoned::RestoreSelection
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one widget: the debounced search, the key handling, and the listbox belong together"
)]
#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Combobox(
    selected_value: RwSignal<String>,
    selected_label: RwSignal<String>,
    #[prop(into)] source: ComboboxSource,
    #[prop(into)] placeholder: String,
    #[prop(into)] empty_message: String,
    #[prop(into)] clear_label: String,
    #[prop(optional, into)] invalid: Signal<bool>,
    #[prop(optional)] display: ComboboxDisplay,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let invalid = with_field_error(invalid);
    let combobox_number = ElementKey::next();
    let listbox_id = format!("combobox-{combobox_number}-listbox");
    let option_id = move |index: usize| format!("combobox-{combobox_number}-option-{index}");

    let selection_text = move || {
        display.selection(
            &selected_value.get_untracked(),
            &selected_label.get_untracked(),
        )
    };
    let query = RwSignal::new(selection_text());
    let results = RwSignal::new(Results::Idle);
    let showing_suggestions = RwSignal::new(false);
    let open = RwSignal::new(false);
    let highlighted: RwSignal<Option<usize>> = RwSignal::new(None);
    let latest_request = StoredValue::new(0_u64);
    let panel = AnchoredPanel::new();
    let input: NodeRef<Input> = NodeRef::new();
    panel.follow(
        open.into(),
        move || input.get_untracked().map(Into::into),
        ANCHORING,
    );

    Effect::new(move |_| {
        let selection = display.selection(&selected_value.get(), &selected_label.get());
        query.set(selection);
    });

    let options = Memo::new(move |_| match results.get() {
        Results::Loaded(groups) => grouped_options(&groups),
        Results::Idle | Results::Searching => Vec::new(),
    });

    let show = move || open.set(true);
    let close = move || {
        open.set(false);
        highlighted.set(None);
        panel.unplace();
    };

    let opens_on_focus = source.opens_on_focus();
    let filters_locally = source.filters_locally();
    let suggestions_heading = StoredValue::new(source.suggestions_heading());
    let source = StoredValue::new(source);
    let search = move |search_text: String| {
        let request = latest_request.get_value().wrapping_add(1);
        latest_request.set_value(request);
        let loaded = move |loaded: Vec<ComboboxGroup>| {
            highlighted.set((!lacks_options(&loaded)).then_some(0));
            results.set(Results::Loaded(loaded));
        };
        match source.with_value(|source| source.lookup(&search_text, display)) {
            Lookup::Nothing => {
                results.set(Results::Idle);
                close();
            }
            Lookup::Ready(options) => {
                showing_suggestions.set(false);
                loaded(options);
                show();
            }
            Lookup::Pending {
                pending,
                suggesting,
            } => {
                showing_suggestions.set(suggesting);
                results.set(Results::Searching);
                show();
                let heading = suggesting
                    .then(|| suggestions_heading.get_value())
                    .flatten();
                leptos::task::spawn_local(async move {
                    let options = pending.await.unwrap_or_default();
                    if latest_request.get_value() == request {
                        loaded(vec![ComboboxGroup { heading, options }]);
                    }
                });
            }
        }
    };
    let debounced_search = leptos_use::use_debounce_fn_with_arg(search, DEBOUNCE_MILLISECONDS);
    let search_as_typed = move |search_text: String| {
        if filters_locally {
            search(search_text);
        } else {
            debounced_search(search_text);
        }
    };

    let select = move |option: ComboboxOption| {
        query.set(option.display());
        selected_value.set(option.value);
        selected_label.set(option.label);
        results.set(Results::Idle);
        close();
    };
    let clear_selection = move || {
        selected_value.set(String::new());
        selected_label.set(String::new());
        query.set(String::new());
        results.set(Results::Idle);
        close();
    };
    let restore_selection = move || {
        query.set(selection_text());
        results.set(Results::Idle);
        close();
    };
    let leave = move || match abandoned(&query.get_untracked()) {
        Abandoned::ClearSelection => clear_selection(),
        Abandoned::RestoreSelection => restore_selection(),
    };
    let highlighted_option = move || {
        highlighted
            .get_untracked()
            .and_then(|index| options.with_untracked(|list| list.get(index).cloned()))
    };

    let browsing_text = move || {
        let text = query.get_untracked();
        if text == selection_text() {
            String::new()
        } else {
            text
        }
    };

    let on_keydown = move |event: KeyboardEvent| {
        let option_count = options.with(Vec::len);
        let is_open = open.get_untracked();
        let movement = match event.key().as_str() {
            "ArrowDown" => Some(Movement::Next),
            "ArrowUp" => Some(Movement::Previous),
            "Home" | "PageUp" if is_open => Some(Movement::First),
            "End" | "PageDown" if is_open => Some(Movement::Last),
            _ => None,
        };
        if let Some(movement) = movement {
            event.prevent_default();
            if option_count == 0 {
                if !is_open {
                    search(browsing_text());
                }
                return;
            }
            show();
            highlighted.update(|current| {
                *current = moved_position(*current, option_count, movement);
            });
            return;
        }
        match event.key().as_str() {
            "Enter" if is_open => {
                event.prevent_default();
                event.stop_propagation();
                if let Some(option) = highlighted_option() {
                    select(option);
                }
            }
            "Escape" if is_open || query.get_untracked() != selection_text() => {
                event.prevent_default();
                event.stop_propagation();
                restore_selection();
            }
            "Tab" if is_open => match highlighted_option() {
                Some(option) => select(option),
                None => close(),
            },
            _ => {}
        }
    };

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let Some(index) = highlighted.get() else {
            return;
        };
        if !open.get() {
            return;
        }
        if let Some(element) = document().get_element_by_id(&option_id(index)) {
            let scrolling = web_sys::ScrollIntoViewOptions::new();
            scrolling.set_block(web_sys::ScrollLogicalPosition::Nearest);
            element.scroll_into_view_with_scroll_into_view_options(&scrolling);
        }
    });

    let listbox_id_for_input = listbox_id.clone();
    let listbox_id = StoredValue::new(listbox_id);
    let empty_message = StoredValue::new(empty_message);

    let option_rows = move |first_index: usize, loaded: Vec<ComboboxOption>| {
        loaded
            .into_iter()
            .enumerate()
            .map(move |(position, option)| {
                let index = first_index.saturating_add(position);
                let is_highlighted = move || highlighted.get() == Some(index);
                let display_value = display.shows_value().then(|| {
                    view! {
                        <span class="font-medium text-gray-900">{option.value.clone()}</span>
                        <span class="text-gray-500">" - "</span>
                    }
                });
                let display_label = option.label.clone();
                let detail = option.detail.clone();
                view! {
                    <div
                        id=option_id(index)
                        role="option"
                        aria-selected=move || is_highlighted().to_string()
                        class=move || {
                            if is_highlighted() { HIGHLIGHTED_OPTION_CLASS } else { OPTION_CLASS }
                        }
                        on:mousemove=move |_| {
                            if highlighted.get_untracked() != Some(index) {
                                highlighted.set(Some(index));
                            }
                        }
                        on:click=move |_| select(option.clone())
                    >
                        <span class="min-w-0 truncate">
                            {display_value} <span>{display_label}</span>
                        </span>
                        {detail
                            .map(|detail| {
                                view! {
                                    <span class="shrink-0 text-xs tabular-nums text-gray-500">
                                        {detail}
                                    </span>
                                }
                            })}
                    </div>
                }
            })
            .collect::<Vec<_>>()
    };
    let group_views = move |groups: Vec<ComboboxGroup>| {
        let mut first_index = 0;
        groups
            .into_iter()
            .enumerate()
            .map(|(position, group)| {
                let separator = (position > 0).then(|| {
                    view! { <div aria-hidden="true" class=GROUP_SEPARATOR_CLASS></div> }
                });
                let heading_view = group.heading.clone().map(|heading| {
                    view! {
                        <div aria-hidden="true" class=GROUP_HEADING_CLASS>
                            {heading}
                        </div>
                    }
                });
                let option_count = group.options.len();
                let rows = option_rows(first_index, group.options);
                first_index = first_index.saturating_add(option_count);
                view! {
                    {separator}
                    <div role=group.heading.is_some().then_some("group") aria-label=group.heading>
                        {heading_view}
                        {rows}
                    </div>
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="relative">
            <div class="flex gap-2">
                <input
                    type="text"
                    node_ref=input
                    id=field_control_id()
                    aria-describedby=field_description_id()
                    aria-label=label
                    role="combobox"
                    aria-autocomplete="list"
                    aria-expanded=move || open.get().to_string()
                    aria-controls=move || open.get().then(|| listbox_id_for_input.clone())
                    aria-activedescendant=move || {
                        open.get().then(|| highlighted.get().map(option_id)).flatten()
                    }
                    aria-invalid=move || invalid.get().to_string()
                    placeholder=placeholder
                    autocomplete="off"
                    class=CONTROL_CLASS
                    prop:value=query
                    on:input=move |event: Event| {
                        let value = event_target_value(&event);
                        query.set(value.clone());
                        search_as_typed(value);
                    }
                    on:keydown=on_keydown
                    on:focus=move |_| {
                        if options.with_untracked(|list| !list.is_empty()) {
                            show();
                        } else if opens_on_focus && query.get_untracked().trim().is_empty() {
                            search(String::new());
                        }
                    }
                    on:blur=move |_| leave()
                />
                {move || {
                    (!selected_value.get().is_empty())
                        .then(|| {
                            let clear_label = clear_label.clone();
                            view! {
                                <Tooltip label=clear_label.clone()>
                                    <button
                                        type="button"
                                        class=icon_button_class(
                                            ButtonTone::Ghost,
                                            ButtonSize::Compact,
                                        )
                                        aria-label=clear_label
                                        on:click=move |_| clear_selection()
                                    >
                                        {icons::icon(icons::LucideGlyph::X, "h-4 w-4")}
                                    </button>
                                </Tooltip>
                            }
                        })
                }}
            </div>
            {move || {
                if !open.get() {
                    return ().into_any();
                }
                let rows = match results.get() {
                    Results::Idle => return ().into_any(),
                    Results::Searching => {
                        view! {
                            <div class=format!(
                                "flex items-center gap-2 {MESSAGE_CLASS}",
                            )>{icons::spinner("h-4 w-4")} "Searching\u{2026}"</div>
                        }
                            .into_any()
                    }
                    Results::Loaded(
                        groups,
                    ) if lacks_options(&groups) && showing_suggestions.get() => {
                        return ().into_any();
                    }
                    Results::Loaded(groups) if lacks_options(&groups) => {
                        view! { <div class=MESSAGE_CLASS>{empty_message.get_value()}</div> }
                            .into_any()
                    }
                    Results::Loaded(groups) => group_views(groups).into_any(),
                };
                view! {
                    <div
                        id=listbox_id.get_value()
                        role="listbox"
                        node_ref=panel.node
                        class=LISTBOX_PANEL_CLASS
                        style=move || panel.style()
                        on:mousedown=|event: MouseEvent| event.prevent_default()
                    >
                        {rows}
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Vec<ComboboxOption> {
        vec![
            ComboboxOption::new("US", "United States"),
            ComboboxOption::new("TR", "Turkey"),
            ComboboxOption::new("IT", "Italy"),
        ]
    }

    fn server_search() -> ComboboxSearch {
        ComboboxSearch::new(|_search: String| async {
            Ok::<Vec<ComboboxOption>, String>(Vec::new())
        })
    }

    #[test]
    fn shows_a_selection_as_value_and_label() {
        assert_eq!(selection_display("C1000", "Acme"), "C1000 - Acme");
        assert_eq!(selection_display("", ""), "");
    }

    #[test]
    fn ranks_an_exact_code_ahead_of_names_that_merely_contain_it() {
        let states = vec![
            ComboboxOption::new("AS", "American Samoa"),
            ComboboxOption::new("CA", "California"),
            ComboboxOption::new("NC", "North Carolina"),
        ];
        let codes: Vec<String> = matching_options(&states, "ca", ComboboxDisplay::ValueAndLabel)
            .into_iter()
            .map(|option| option.value)
            .collect();
        assert_eq!(codes, ["CA", "AS", "NC"]);
    }

    #[test]
    fn an_emptied_input_clears_the_selection() {
        assert_eq!(abandoned("   "), Abandoned::ClearSelection);
    }

    #[test]
    fn leftover_text_puts_the_selection_back() {
        assert_eq!(abandoned("trav"), Abandoned::RestoreSelection);
    }

    #[test]
    fn searches_the_server_from_two_characters() {
        let source = ComboboxSource::from(server_search());
        assert!(matches!(
            source.lookup("", ComboboxDisplay::ValueAndLabel),
            Lookup::Nothing
        ));
        assert!(matches!(
            source.lookup("t", ComboboxDisplay::ValueAndLabel),
            Lookup::Nothing
        ));
        assert!(matches!(
            source.lookup("tr", ComboboxDisplay::ValueAndLabel),
            Lookup::Pending {
                suggesting: false,
                ..
            }
        ));
    }

    #[test]
    fn offers_suggestions_before_anything_is_typed() {
        let source = ComboboxSource::from(server_search().with_suggestions("Recent", || async {
            Ok::<Vec<ComboboxOption>, String>(Vec::new())
        }));
        assert!(source.opens_on_focus());
        assert!(matches!(
            source.lookup(" ", ComboboxDisplay::ValueAndLabel),
            Lookup::Pending {
                suggesting: true,
                ..
            }
        ));
        assert!(matches!(
            source.lookup("t", ComboboxDisplay::ValueAndLabel),
            Lookup::Nothing
        ));
    }

    #[test]
    fn a_list_filters_without_asking_the_server() {
        let source = ComboboxSource::from(options());
        assert!(source.opens_on_focus());
        assert!(
            matches!(source.lookup("it", ComboboxDisplay::ValueAndLabel), Lookup::Ready(found) if grouped_options(&found).len() == 2)
        );
    }

    #[test]
    fn a_fixed_list_matches_code_or_name_ignoring_case() {
        let codes: Vec<String> = matching_options(&options(), "it", ComboboxDisplay::ValueAndLabel)
            .into_iter()
            .map(|option| option.value)
            .collect();
        assert_eq!(codes, vec!["IT", "US"]);
    }

    #[test]
    fn a_fixed_list_offers_everything_for_an_empty_search() {
        assert_eq!(
            matching_options(&options(), " ", ComboboxDisplay::ValueAndLabel).len(),
            3
        );
    }

    fn recent_then_everything() -> ComboboxSource {
        ComboboxSource::from(vec![
            ComboboxGroup::new(vec![ComboboxOption::new("TR", "Turkey")]).with_heading("Recent"),
            ComboboxGroup::new(options()),
        ])
    }

    fn looked_up_values(source: &ComboboxSource, text: &str) -> Vec<Vec<String>> {
        let Lookup::Ready(groups) = source.lookup(text, ComboboxDisplay::ValueAndLabel) else {
            panic!("a grouped list answers without loading");
        };
        groups
            .iter()
            .map(|group| {
                group
                    .options
                    .iter()
                    .map(|option| option.value.clone())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn groups_keep_a_recent_option_in_the_full_list_too() {
        assert_eq!(
            looked_up_values(&recent_then_everything(), ""),
            vec![vec!["TR"], vec!["US", "TR", "IT"]]
        );
    }

    #[test]
    fn groups_filter_separately_and_drop_groups_with_no_match() {
        let source = recent_then_everything();
        assert_eq!(
            looked_up_values(&source, "tur"),
            vec![vec!["TR"], vec!["TR"]]
        );
        assert_eq!(looked_up_values(&source, "ital"), vec![vec!["IT"]]);
    }

    #[test]
    fn groups_number_their_options_straight_through() {
        let Lookup::Ready(groups) = recent_then_everything().lookup("", ComboboxDisplay::Label)
        else {
            panic!("a grouped list answers without loading");
        };
        assert_eq!(grouped_options(&groups).len(), 4);
    }

    #[test]
    fn label_display_shows_only_the_label() {
        assert_eq!(ComboboxDisplay::Label.selection("42", "Acme"), "Acme");
        assert_eq!(ComboboxDisplay::Label.selection("", "Acme"), "");
    }

    #[test]
    fn label_display_ignores_the_value_when_matching() {
        let businesses = vec![
            ComboboxOption::new("12", "Acme Stone"),
            ComboboxOption::new("7", "Studio 12"),
        ];
        let labels: Vec<String> = matching_options(&businesses, "12", ComboboxDisplay::Label)
            .into_iter()
            .map(|option| option.label)
            .collect();
        assert_eq!(labels, ["Studio 12"]);
    }
}
