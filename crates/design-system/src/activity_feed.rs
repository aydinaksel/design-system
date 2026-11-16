use leptos::prelude::*;

use crate::button::{button_class, ButtonSize, ButtonTone};
use crate::dialog::{Dialog, DialogSize};
use crate::empty_state::EmptyState;
use crate::filter_chip::FilterChip;
use crate::icons::{self, LucideGlyph};
use crate::markdown::{markdown_plain_text, MarkdownText};
use crate::menu::RowActions;
use crate::typography::LABEL_CLASS;

const DAYS_CLASS: &str = "space-y-6";
const DAY_CLASS: &str = "space-y-2";
const DAY_CARD_CLASS: &str =
    "divide-y divide-divider overflow-hidden rounded-xl border border-edge bg-surface";
const OPENABLE_ENTRY_CLASS: &str = "transition-colors hover:bg-row-hover";
const SUMMARY_CLASS: &str = "flex min-w-0 flex-1 items-center gap-3 px-4 py-3 text-left";
const OPEN_BUTTON_CLASS: &str = "flex w-full cursor-pointer items-center gap-3 px-4 py-3 text-left focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus";
const ICON_TILE_CLASS: &str =
    "flex size-9 shrink-0 items-center justify-center rounded-lg bg-gray-100 text-gray-600";
const EVENT_TILE_CLASS: &str = "flex size-9 shrink-0 items-center justify-center text-gray-400";
const ICON_CLASS: &str = "size-4";
const SUBJECT_CLASS: &str = "truncate text-sm font-medium text-gray-900";
const EVENT_SUBJECT_CLASS: &str = "truncate text-sm text-gray-600";
const META_CLASS: &str = "shrink-0 text-xs tabular-nums text-gray-500";
const PREVIEW_CLASS: &str = "mt-0.5 truncate text-sm text-gray-500";
const BODY_TEXT_CLASS: &str = "whitespace-pre-line text-sm text-gray-700";
const CHIPS_CLASS: &str = "flex flex-wrap items-center gap-2";
const SEPARATOR: &str = " \u{b7} ";
const ALL_LABEL: &str = "All";
const DELETE_QUESTION: &str = "This removes the entry for everyone.";
const OPEN_THREAD_LABEL: &str = "Open Thread";
const CLOSE_LABEL: &str = "Close";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivityKind {
    EmailReceived,
    EmailSent,
    CallIncoming,
    CallOutgoing,
    Meeting,
    Note,
    Event,
}

impl ActivityKind {
    pub const LOGGABLE: [Self; 6] = [
        Self::Note,
        Self::CallIncoming,
        Self::CallOutgoing,
        Self::EmailReceived,
        Self::EmailSent,
        Self::Meeting,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EmailReceived => "Email Received",
            Self::EmailSent => "Email Sent",
            Self::CallIncoming => "Incoming Call",
            Self::CallOutgoing => "Outgoing Call",
            Self::Meeting => "Meeting",
            Self::Note => "Note",
            Self::Event => "Change",
        }
    }

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::EmailReceived => "email-received",
            Self::EmailSent => "email-sent",
            Self::CallIncoming => "call-incoming",
            Self::CallOutgoing => "call-outgoing",
            Self::Meeting => "meeting",
            Self::Note => "note",
            Self::Event => "event",
        }
    }

    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::LOGGABLE
            .into_iter()
            .chain([Self::Event])
            .find(|kind| kind.key() == key)
    }

    const fn glyph(self) -> LucideGlyph {
        match self {
            Self::EmailReceived => LucideGlyph::Mail,
            Self::EmailSent => LucideGlyph::Send,
            Self::CallIncoming => LucideGlyph::PhoneIncoming,
            Self::CallOutgoing => LucideGlyph::PhoneOutgoing,
            Self::Meeting => LucideGlyph::Video,
            Self::Note => LucideGlyph::NotebookPen,
            Self::Event => LucideGlyph::RotateCcwClock,
        }
    }

    const fn person_relation(self) -> &'static str {
        match self {
            Self::EmailReceived | Self::CallIncoming => "From",
            Self::EmailSent | Self::CallOutgoing => "To",
            Self::Meeting => "With",
            Self::Note | Self::Event => "By",
        }
    }

    const fn filter(self) -> ActivityFilter {
        match self {
            Self::EmailReceived | Self::EmailSent => ActivityFilter::Emails,
            Self::CallIncoming | Self::CallOutgoing => ActivityFilter::Calls,
            Self::Meeting => ActivityFilter::Meetings,
            Self::Note => ActivityFilter::Notes,
            Self::Event => ActivityFilter::Events,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ActivityFilter {
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

    const fn label(self) -> &'static str {
        match self {
            Self::Emails => "Emails",
            Self::Calls => "Calls",
            Self::Meetings => "Meetings",
            Self::Notes => "Notes",
            Self::Events => "Changes",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActivityBody {
    Text(String),
    Markdown(String),
}

impl ActivityBody {
    fn preview(&self) -> String {
        match self {
            Self::Text(text) => text.split_whitespace().collect::<Vec<_>>().join(" "),
            Self::Markdown(text) => markdown_plain_text(text),
        }
    }

    fn full_view(self) -> AnyView {
        match self {
            Self::Text(text) => view! { <p class=BODY_TEXT_CLASS>{text}</p> }.into_any(),
            Self::Markdown(text) => view! { <MarkdownText text=text /> }.into_any(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityEntry {
    pub key: String,
    pub kind: ActivityKind,
    pub subject: String,
    pub body: Option<ActivityBody>,
    pub person: Option<String>,
    pub day: String,
    pub time: Option<String>,
    pub thread_href: Option<String>,
    pub changeable: bool,
}

impl ActivityEntry {
    fn person_line(&self) -> Option<String> {
        self.person
            .as_ref()
            .map(|person| format!("{} {person}", self.kind.person_relation()))
    }

    fn row_meta(&self) -> String {
        [self.person_line(), self.time.clone()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(SEPARATOR)
    }

    fn dialog_description(&self) -> String {
        let when = match &self.time {
            Some(time) => format!("{} at {time}", self.day),
            None => self.day.clone(),
        };
        [
            Some(self.kind.label().to_owned()),
            self.person_line(),
            Some(when),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(SEPARATOR)
    }

    const fn opens(&self) -> bool {
        self.body.is_some() || self.thread_href.is_some() || self.changeable
    }
}

#[must_use]
pub fn people_summary(names: &[String]) -> Option<String> {
    match names {
        [] => None,
        [only] => Some(only.clone()),
        [first, second] => Some(format!("{first} and {second}")),
        [first, others @ ..] => Some(format!("{first} and {} others", others.len())),
    }
}

#[derive(Clone, Copy)]
struct EntryChanges {
    on_edit: Option<Callback<String>>,
    on_delete: Option<Callback<String>>,
}

fn icon_tile(kind: ActivityKind) -> AnyView {
    let class = if kind == ActivityKind::Event {
        EVENT_TILE_CLASS
    } else {
        ICON_TILE_CLASS
    };
    view! { <div class=class>{icons::icon(kind.glyph(), ICON_CLASS)}</div> }.into_any()
}

fn summary_view(entry: &ActivityEntry) -> AnyView {
    let subject_class = if entry.kind == ActivityKind::Event {
        EVENT_SUBJECT_CLASS
    } else {
        SUBJECT_CLASS
    };
    let subject = entry.subject.clone();
    let full_subject = entry.subject.clone();
    let preview = entry
        .body
        .as_ref()
        .map(ActivityBody::preview)
        .filter(|preview| !preview.is_empty())
        .map(|preview| view! { <p class=PREVIEW_CLASS>{preview}</p> });
    view! {
        {icon_tile(entry.kind)}
        <div class="min-w-0 flex-1">
            <p class=subject_class title=full_subject>
                {subject}
            </p>
            {preview}
        </div>
        <span class=META_CLASS>{entry.row_meta()}</span>
    }
    .into_any()
}

fn entry_actions(
    entry: &ActivityEntry,
    changes: EntryChanges,
    on_close: Callback<()>,
) -> Option<AnyView> {
    if !entry.changeable {
        return None;
    }
    let (Some(on_edit), Some(on_delete)) = (changes.on_edit, changes.on_delete) else {
        return None;
    };
    let edit_key = entry.key.clone();
    let delete_key = entry.key.clone();
    let edit = Callback::new(move |()| {
        on_close.run(());
        on_edit.run(edit_key.clone());
    });
    let delete = Callback::new(move |()| {
        on_close.run(());
        on_delete.run(delete_key.clone());
    });
    Some(
        view! { <RowActions on_edit=edit on_delete=delete delete_question=DELETE_QUESTION /> }
            .into_any(),
    )
}

fn entry_view(entry: ActivityEntry, opened: RwSignal<Option<ActivityEntry>>) -> AnyView {
    let summary = summary_view(&entry);
    if !entry.opens() {
        return view! { <li class=SUMMARY_CLASS>{summary}</li> }.into_any();
    }
    view! {
        <li class=OPENABLE_ENTRY_CLASS>
            <button
                type="button"
                class=OPEN_BUTTON_CLASS
                on:click=move |_| opened.set(Some(entry.clone()))
            >
                {summary}
            </button>
        </li>
    }
    .into_any()
}

fn entry_dialog(entry: ActivityEntry, changes: EntryChanges, on_close: Callback<()>) -> AnyView {
    let description = Signal::stored(entry.dialog_description());
    let actions = entry_actions(&entry, changes, on_close);
    let body = entry.body.map(ActivityBody::full_view);
    let thread_link = entry.thread_href.map(|href| {
        view! {
            <a href=href class=button_class(ButtonTone::Secondary, ButtonSize::Standard)>
                {OPEN_THREAD_LABEL}
            </a>
        }
    });
    view! {
        <Dialog
            title=entry.subject
            description=description
            on_close=on_close
            size=DialogSize::Large
        >
            <div class="space-y-6">
                {body}
                <div class="flex items-center gap-2">
                    {actions} <div class="ml-auto"></div> {thread_link}
                    <button
                        type="button"
                        class=button_class(ButtonTone::Primary, ButtonSize::Standard)
                        on:click=move |_| on_close.run(())
                    >
                        {CLOSE_LABEL}
                    </button>
                </div>
            </div>
        </Dialog>
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

fn day_view(
    day: String,
    members: Vec<ActivityEntry>,
    opened: RwSignal<Option<ActivityEntry>>,
) -> AnyView {
    let rows = members
        .into_iter()
        .map(|entry| entry_view(entry, opened))
        .collect_view();
    view! {
        <section class=DAY_CLASS>
            <h3 class=LABEL_CLASS>{day}</h3>
            <ul class=DAY_CARD_CLASS>{rows}</ul>
        </section>
    }
    .into_any()
}

fn toggled(current: Option<ActivityFilter>, filter: ActivityFilter) -> Option<ActivityFilter> {
    (current != Some(filter)).then_some(filter)
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
                            chosen.update(|current| *current = toggled(*current, filter));
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
    let opened: RwSignal<Option<ActivityEntry>> = RwSignal::new(None);
    let changes = EntryChanges { on_edit, on_delete };
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
            .map(|(day, members)| day_view(day, members, opened))
            .collect_view();
        view! { <div class=DAYS_CLASS>{days}</div> }.into_any()
    };
    let close_dialog = Callback::new(move |()| opened.set(None));
    let dialog = move || {
        opened
            .get()
            .map(|entry| entry_dialog(entry, changes, close_dialog))
    };
    view! {
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-3">
                <div class=CHIPS_CLASS>{filter_chips(entries, chosen)}</div>
                <div class="flex items-center gap-2">{actions}</div>
            </div>
            {feed}
            {dialog}
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
            subject: "Note".to_owned(),
            body: None,
            person: None,
            day: day.to_owned(),
            time: None,
            thread_href: None,
            changeable: false,
        }
    }

    fn names(people: &[&str]) -> Vec<String> {
        people.iter().map(|&name| name.to_owned()).collect()
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

    #[test]
    fn summarises_many_recipients_by_the_first() {
        assert_eq!(people_summary(&[]), None);
        assert_eq!(
            people_summary(&names(&["Tom Reeve"])).as_deref(),
            Some("Tom Reeve")
        );
        assert_eq!(
            people_summary(&names(&["Tom Reeve", "Sarah Doyle"])).as_deref(),
            Some("Tom Reeve and Sarah Doyle")
        );
        assert_eq!(
            people_summary(&names(&[
                "Tom Reeve",
                "Sarah Doyle",
                "Priya Shah",
                "Ali Khan"
            ]))
            .as_deref(),
            Some("Tom Reeve and 3 others")
        );
    }

    #[test]
    fn describes_who_and_when_by_direction() {
        let received = ActivityEntry {
            kind: ActivityKind::EmailReceived,
            person: Some("Sarah Doyle".to_owned()),
            time: Some("14:02".to_owned()),
            ..entry("a", "Today")
        };
        assert_eq!(received.row_meta(), "From Sarah Doyle \u{b7} 14:02");
        assert_eq!(
            received.dialog_description(),
            "Email Received \u{b7} From Sarah Doyle \u{b7} Today at 14:02"
        );
        let logged_call = ActivityEntry {
            kind: ActivityKind::CallOutgoing,
            ..entry("b", "Yesterday")
        };
        assert_eq!(logged_call.row_meta(), "");
        assert_eq!(
            logged_call.dialog_description(),
            "Outgoing Call \u{b7} Yesterday"
        );
    }

    #[test]
    fn every_kind_round_trips_through_its_key() {
        for kind in ActivityKind::LOGGABLE
            .into_iter()
            .chain([ActivityKind::Event])
        {
            assert_eq!(ActivityKind::from_key(kind.key()), Some(kind));
        }
    }
}
