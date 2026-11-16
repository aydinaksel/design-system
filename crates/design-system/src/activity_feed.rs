use leptos::prelude::*;

use crate::disclosure::Disclosure;
use crate::empty_state::EmptyState;
use crate::filter_chip::FilterChip;
use crate::icons;
use crate::markdown::MarkdownText;
use crate::menu::RowActions;
use crate::typography::LABEL_CLASS;

const FEED_CLASS: &str = "overflow-hidden rounded-xl border border-edge bg-surface";
const DAY_CLASS: &str = "bg-gray-50 px-4 py-2";
const ENTRY_CLASS: &str = "flex gap-3 px-4 py-3";
const ICON_TILE_CLASS: &str =
    "flex size-9 shrink-0 items-center justify-center rounded-lg bg-gray-100 text-gray-500";
const EVENT_TILE_CLASS: &str = "flex size-9 shrink-0 items-center justify-center text-gray-300";
const ICON_CLASS: &str = "h-4 w-4";
const EVENT_ICON_CLASS: &str = "h-2.5 w-2.5";
const HEADING_ROW_CLASS: &str = "flex min-h-9 items-center justify-between gap-3";
const TITLE_CLASS: &str = "min-w-0 truncate text-sm font-medium text-gray-900";
const EVENT_TITLE_CLASS: &str = "min-w-0 truncate text-sm text-gray-600";
const LINK_CLASS: &str = "hover:text-blue-700 hover:underline";
const META_CLASS: &str = "shrink-0 text-xs tabular-nums text-gray-500";
const SNIPPET_CLASS: &str = "line-clamp-1 text-sm text-gray-600";
const FULL_TEXT_CLASS: &str = "whitespace-pre-line text-sm text-gray-700";
const CHIPS_CLASS: &str = "flex flex-wrap items-center gap-2";
const ALL_LABEL: &str = "All";
const DELETE_QUESTION: &str = "This removes the note for everyone.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivityKind {
    EmailReceived,
    EmailSent,
    Call,
    Meeting,
    Note,
    Event,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivityFilter {
    Emails,
    Calls,
    Meetings,
    Notes,
    Events,
}

impl ActivityFilter {
    const ALL: [Self; 5] = [
        Self::Emails,
        Self::Calls,
        Self::Meetings,
        Self::Notes,
        Self::Events,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Emails => "Emails",
            Self::Calls => "Calls",
            Self::Meetings => "Meetings",
            Self::Notes => "Notes",
            Self::Events => "Changes",
        }
    }
}

impl ActivityKind {
    fn glyph(self) -> icons::LucideGlyph {
        match self {
            Self::EmailReceived | Self::EmailSent => icons::LucideGlyph::Mail,
            Self::Call => icons::LucideGlyph::Phone,
            Self::Meeting => icons::LucideGlyph::Video,
            Self::Note => icons::LucideGlyph::NotebookPen,
            Self::Event => icons::LucideGlyph::CircleDot,
        }
    }

    fn filter(self) -> ActivityFilter {
        match self {
            Self::EmailReceived | Self::EmailSent => ActivityFilter::Emails,
            Self::Call => ActivityFilter::Calls,
            Self::Meeting => ActivityFilter::Meetings,
            Self::Note => ActivityFilter::Notes,
            Self::Event => ActivityFilter::Events,
        }
    }

    fn direction(self) -> Option<icons::LucideGlyph> {
        match self {
            Self::EmailReceived => Some(icons::LucideGlyph::ArrowDownLeft),
            Self::EmailSent => Some(icons::LucideGlyph::ArrowUpRight),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActivityDetail {
    Snippet { preview: String, full_text: String },
    Text(String),
    Markdown(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityEntry {
    pub key: String,
    pub kind: ActivityKind,
    pub title: String,
    pub detail: Option<ActivityDetail>,
    pub people: String,
    pub day: String,
    pub time: String,
    pub href: Option<String>,
    pub changeable: bool,
}

fn icon_tile(kind: ActivityKind) -> AnyView {
    if kind == ActivityKind::Event {
        return view! { <div class=EVENT_TILE_CLASS>{icons::icon(kind.glyph(), EVENT_ICON_CLASS)}</div> }
        .into_any();
    }
    let direction = kind
        .direction()
        .map(|glyph| icons::icon(glyph, "h-3 w-3 -ml-0.5"));
    view! { <div class=ICON_TILE_CLASS>{icons::icon(kind.glyph(), ICON_CLASS)} {direction}</div> }
        .into_any()
}

fn detail_view(detail: ActivityDetail) -> AnyView {
    match detail {
        ActivityDetail::Snippet { preview, full_text } => {
            let remainder = full_text
                .strip_prefix(preview.as_str())
                .map_or(full_text.as_str(), str::trim_start)
                .to_owned();
            if remainder.is_empty() {
                return view! { <p class=FULL_TEXT_CLASS>{preview}</p> }.into_any();
            }
            let summary = ViewFn::from(move || {
                view! { <p class=SNIPPET_CLASS>{preview.clone()}</p> }.into_any()
            });
            view! {
                <Disclosure summary=summary>
                    <p class=FULL_TEXT_CLASS>{remainder.clone()}</p>
                </Disclosure>
            }
            .into_any()
        }
        ActivityDetail::Text(text) => view! { <p class=FULL_TEXT_CLASS>{text}</p> }.into_any(),
        ActivityDetail::Markdown(text) => view! { <MarkdownText text=text /> }.into_any(),
    }
}

fn title_view(entry: &ActivityEntry) -> AnyView {
    let class = if entry.kind == ActivityKind::Event {
        EVENT_TITLE_CLASS
    } else {
        TITLE_CLASS
    };
    let title = entry.title.clone();
    let full_title = entry.title.clone();
    match entry.href.clone() {
        Some(href) => view! {
            <a href=href class=format!("{class} {LINK_CLASS}") title=full_title>
                {title}
            </a>
        }
        .into_any(),
        None => view! {
            <p class=class title=full_title>
                {title}
            </p>
        }
        .into_any(),
    }
}

fn entry_actions(
    entry: &ActivityEntry,
    on_edit: Option<Callback<String>>,
    on_delete: Option<Callback<String>>,
) -> Option<AnyView> {
    if !entry.changeable {
        return None;
    }
    let (Some(on_edit), Some(on_delete)) = (on_edit, on_delete) else {
        return None;
    };
    let edit_key = entry.key.clone();
    let delete_key = entry.key.clone();
    Some(
        view! {
            <RowActions
                on_edit=Callback::new(move |()| on_edit.run(edit_key.clone()))
                edit_label="Edit Note"
                on_delete=Callback::new(move |()| on_delete.run(delete_key.clone()))
                delete_label="Delete Note"
                delete_question=DELETE_QUESTION
            />
        }
        .into_any(),
    )
}

fn entry_view(
    entry: ActivityEntry,
    on_edit: Option<Callback<String>>,
    on_delete: Option<Callback<String>>,
) -> AnyView {
    let actions = entry_actions(&entry, on_edit, on_delete);
    let meta = if entry.people.is_empty() {
        entry.time.clone()
    } else {
        format!("{} \u{b7} {}", entry.people, entry.time)
    };
    let title = title_view(&entry);
    let detail = entry.detail.map(detail_view);
    view! {
        <li class=ENTRY_CLASS>
            {icon_tile(entry.kind)} <div class="min-w-0 flex-1">
                <div class=HEADING_ROW_CLASS>
                    {title} <div class="flex shrink-0 items-center gap-2">
                        <span class=META_CLASS>{meta}</span>
                        {actions}
                    </div>
                </div>
                {detail}
            </div>
        </li>
    }
    .into_any()
}

fn grouped_by_day(entries: Vec<ActivityEntry>) -> Vec<(String, Vec<ActivityEntry>)> {
    let mut groups: Vec<(String, Vec<ActivityEntry>)> = Vec::new();
    for entry in entries {
        match groups.last_mut() {
            Some((day, members)) if *day == entry.day => members.push(entry),
            _ => groups.push((entry.day.clone(), vec![entry])),
        }
    }
    groups
}

fn filter_chips(
    entries: Signal<Vec<ActivityEntry>>,
    chosen: RwSignal<Option<ActivityFilter>>,
) -> impl IntoView {
    move || {
        let counts: Vec<(ActivityFilter, usize)> = entries.with(|all| {
            ActivityFilter::ALL
                .into_iter()
                .map(|filter| {
                    let count = all
                        .iter()
                        .filter(|entry| entry.kind.filter() == filter)
                        .count();
                    (filter, count)
                })
                .filter(|(_, count)| *count > 0)
                .collect()
        });
        let total = entries.with(Vec::len);
        let all_chip = view! {
            <FilterChip
                label=ALL_LABEL
                selected=Signal::derive(move || chosen.get().is_none())
                on_toggle=Callback::new(move |()| chosen.set(None))
                count=total
            />
        };
        let kind_chips = counts
            .into_iter()
            .map(|(filter, count)| {
                view! {
                    <FilterChip
                        label=filter.label()
                        selected=Signal::derive(move || chosen.get() == Some(filter))
                        on_toggle=Callback::new(move |()| {
                            chosen
                                .update(|current| {
                                    *current = if *current == Some(filter) {
                                        None
                                    } else {
                                        Some(filter)
                                    };
                                });
                        })
                        count=count
                    />
                }
            })
            .collect_view();
        view! {
            {all_chip}
            {kind_chips}
        }
    }
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ActivityFeed(
    #[prop(into)] entries: Signal<Vec<ActivityEntry>>,
    #[prop(optional)] actions: Option<AnyView>,
    #[prop(optional, into)] on_edit: Option<Callback<String>>,
    #[prop(optional, into)] on_delete: Option<Callback<String>>,
    #[prop(into, default = "Nothing has happened here yet.".to_owned())] empty_message: String,
) -> impl IntoView {
    let chosen: RwSignal<Option<ActivityFilter>> = RwSignal::new(None);
    let shown = Signal::derive(move || {
        let filter = chosen.get();
        entries.with(|all| {
            all.iter()
                .filter(|entry| filter.is_none_or(|filter| entry.kind.filter() == filter))
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let feed = move || {
        let groups = grouped_by_day(shown.get());
        if groups.is_empty() {
            return view! { <EmptyState title=empty_message.clone() /> }.into_any();
        }
        let days = groups
            .into_iter()
            .map(|(day, members)| {
                let rows = members
                    .into_iter()
                    .map(|entry| entry_view(entry, on_edit, on_delete))
                    .collect_view();
                view! {
                    <li>
                        <h3 class=format!("{DAY_CLASS} {LABEL_CLASS}")>{day}</h3>
                        <ul class="divide-y divide-divider">{rows}</ul>
                    </li>
                }
            })
            .collect_view();
        view! { <ul class=format!("{FEED_CLASS} divide-y divide-edge")>{days}</ul> }.into_any()
    };
    view! {
        <div class="space-y-3">
            <div class="flex flex-wrap items-center justify-between gap-3">
                <div class=CHIPS_CLASS>{filter_chips(entries, chosen)}</div>
                <div class="flex items-center gap-2">{actions}</div>
            </div>
            {feed}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, day: &str) -> ActivityEntry {
        ActivityEntry {
            key: key.to_owned(),
            kind: ActivityKind::Note,
            title: "Note".to_owned(),
            detail: None,
            people: String::new(),
            day: day.to_owned(),
            time: String::new(),
            href: None,
            changeable: false,
        }
    }

    #[test]
    fn groups_consecutive_entries_by_day() {
        let groups = grouped_by_day(vec![
            entry("a", "Today"),
            entry("b", "Today"),
            entry("c", "Yesterday"),
        ]);
        let shape: Vec<(String, usize)> = groups
            .into_iter()
            .map(|(day, members)| (day, members.len()))
            .collect();
        assert_eq!(
            shape,
            vec![("Today".to_owned(), 2), ("Yesterday".to_owned(), 1)]
        );
    }
}
