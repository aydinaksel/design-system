use std::cmp::Reverse;

use chrono::{Local, NaiveDate, TimeDelta};
use design_system::icons::{self, LucideGlyph};
use design_system::{
    button_class, button_icon_class, people_summary, ActivityBody, ActivityDraft, ActivityEntry,
    ActivityFeed, ActivityFormDialog, ActivityKind, ButtonSize, ButtonTone,
};
use leptos::prelude::*;

const ADD_SIZE: ButtonSize = ButtonSize::Compact;
const YOU: &str = "Aydin";
const DATE_FORMAT: &str = "%Y-%m-%d";
const DAY_LABEL_FORMAT: &str = "%-d %B";
const THREAD_HREF: &str = "#";

#[derive(Clone)]
struct DatedEntry {
    date: NaiveDate,
    entry: ActivityEntry,
}

#[derive(Clone, PartialEq, Eq)]
enum Composing {
    NewEntry,
    Editing(String),
}

fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn days_ago(days: i64) -> NaiveDate {
    let today = today();
    today
        .checked_sub_signed(TimeDelta::days(days))
        .unwrap_or(today)
}

fn day_label(date: NaiveDate) -> String {
    match today().signed_duration_since(date).num_days() {
        0 => "Today".to_owned(),
        1 => "Yesterday".to_owned(),
        _ => date.format(DAY_LABEL_FORMAT).to_string(),
    }
}

fn recorded(
    key: &str,
    kind: ActivityKind,
    subject: &str,
    person: Option<String>,
    date: NaiveDate,
    time: &str,
) -> DatedEntry {
    DatedEntry {
        date,
        entry: ActivityEntry {
            key: key.to_owned(),
            kind,
            subject: subject.to_owned(),
            body: None,
            person,
            day: day_label(date),
            time: Some(time.to_owned()),
            thread_href: None,
            changeable: false,
        },
    }
}

fn email(dated: DatedEntry, text: &str) -> DatedEntry {
    DatedEntry {
        entry: ActivityEntry {
            body: Some(ActivityBody::Text(text.to_owned())),
            thread_href: Some(THREAD_HREF.to_owned()),
            ..dated.entry
        },
        ..dated
    }
}

fn logged(key: &str, draft: ActivityDraft, person: Option<String>) -> Option<DatedEntry> {
    let date = NaiveDate::parse_from_str(&draft.date, DATE_FORMAT).ok()?;
    Some(DatedEntry {
        date,
        entry: ActivityEntry {
            key: key.to_owned(),
            kind: draft.kind,
            subject: draft.subject,
            body: (!draft.body.is_empty()).then_some(ActivityBody::Markdown(draft.body)),
            person,
            day: day_label(date),
            time: None,
            thread_href: None,
            changeable: true,
        },
    })
}

fn sample_draft(kind: ActivityKind, date: NaiveDate, subject: &str, body: &str) -> ActivityDraft {
    ActivityDraft {
        subject: subject.to_owned(),
        body: body.to_owned(),
        ..ActivityDraft::new(kind, date.format(DATE_FORMAT).to_string())
    }
}

fn sample_entries() -> Vec<DatedEntry> {
    let quote_recipients = ["Tom Reeve", "Sarah Doyle", "Priya Shah"].map(str::to_owned);
    let logged_entries = [
        logged(
            "logged-note",
            sample_draft(
                ActivityKind::Note,
                today(),
                "Crate count on PO-2318",
                "Supplier confirmed **13 crates** instead of 12. Updated line B.",
            ),
            Some(YOU.to_owned()),
        ),
        logged(
            "logged-call",
            sample_draft(
                ActivityKind::CallOutgoing,
                days_ago(1),
                "Splitting the order",
                "Agreed with Tom to split the order across two containers.",
            ),
            Some("Tom Reeve".to_owned()),
        ),
        logged(
            "logged-meeting",
            sample_draft(
                ActivityKind::Meeting,
                days_ago(1),
                "Showroom visit",
                "Selected two quartz options for the Richmond project.",
            ),
            Some("Priya Shah".to_owned()),
        ),
    ];
    let [note, call, meeting] = logged_entries;
    [
        Some(email(
            recorded(
                "email-received",
                ActivityKind::EmailReceived,
                "Re: revised ETA for MSKU4471029",
                Some("Sarah Doyle".to_owned()),
                today(),
                "14:02",
            ),
            "Vessel delayed at Algeciras, new ETA 28 November.\n\nThe carrier has rolled the booking by one sailing. Customs documents are unchanged.",
        )),
        note,
        Some(email(
            recorded(
                "email-sent",
                ActivityKind::EmailSent,
                "Quote Q-2318 for 14 slabs",
                people_summary(&quote_recipients),
                today(),
                "09:15",
            ),
            "Please find the quote attached.\n\nPrices hold until the end of the month.",
        )),
        call,
        Some(recorded(
            "call-incoming",
            ActivityKind::CallIncoming,
            "Lead times for Carrara",
            Some("Priya Shah".to_owned()),
            days_ago(1),
            "11:00",
        )),
        meeting,
        Some(recorded(
            "event",
            ActivityKind::Event,
            "Status changed to Sent",
            Some(YOU.to_owned()),
            days_ago(4),
            "09:03",
        )),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn draft_for(entries: RwSignal<Vec<DatedEntry>>, key: &str) -> Option<ActivityDraft> {
    entries.with_untracked(|all| {
        let dated = all.iter().find(|dated| dated.entry.key == key)?;
        let body = match &dated.entry.body {
            Some(ActivityBody::Markdown(text) | ActivityBody::Text(text)) => text.clone(),
            None => String::new(),
        };
        Some(ActivityDraft {
            subject: dated.entry.subject.clone(),
            body,
            ..ActivityDraft::new(dated.entry.kind, dated.date.format(DATE_FORMAT).to_string())
        })
    })
}

fn save_entry(entries: RwSignal<Vec<DatedEntry>>, composing: &Composing, draft: ActivityDraft) {
    entries.update(|all| {
        let (key, person) = match composing {
            Composing::Editing(key) => {
                let person = all
                    .iter()
                    .find(|dated| dated.entry.key == *key)
                    .and_then(|dated| dated.entry.person.clone());
                (key.clone(), person)
            }
            Composing::NewEntry => (format!("logged-{}", all.len()), Some(YOU.to_owned())),
        };
        let Some(saved) = logged(&key, draft, person) else {
            return;
        };
        all.retain(|dated| dated.entry.key != key);
        all.insert(0, saved);
        all.sort_by_key(|dated| Reverse(dated.date));
    });
}

fn form_dialog(
    entries: RwSignal<Vec<DatedEntry>>,
    composing: RwSignal<Option<Composing>>,
) -> impl IntoView {
    move || {
        let target = composing.get()?;
        let (title, draft) = match &target {
            Composing::NewEntry => (
                "Log Activity",
                ActivityDraft::new(ActivityKind::Note, today().format(DATE_FORMAT).to_string()),
            ),
            Composing::Editing(key) => ("Edit Activity", draft_for(entries, key)?),
        };
        let on_save = Callback::new(move |draft: ActivityDraft| {
            save_entry(entries, &target, draft);
            composing.set(None);
        });
        Some(view! {
            <ActivityFormDialog
                title=title
                draft=draft
                on_close=Callback::new(move |()| composing.set(None))
                on_save=on_save
            />
        })
    }
}

pub(crate) fn activity_feed() -> AnyView {
    let entries = RwSignal::new(sample_entries());
    let composing: RwSignal<Option<Composing>> = RwSignal::new(None);
    let shown = Signal::derive(move || {
        entries.with(|all| {
            all.iter()
                .map(|dated| dated.entry.clone())
                .collect::<Vec<_>>()
        })
    });
    let log_button = view! {
        <button
            type="button"
            class=button_class(ButtonTone::Secondary, ADD_SIZE)
            on:click=move |_| composing.set(Some(Composing::NewEntry))
        >
            {icons::icon(LucideGlyph::Plus, button_icon_class(ADD_SIZE))}
            "Log Activity"
        </button>
    }
    .into_any();
    let on_edit = Callback::new(move |key: String| composing.set(Some(Composing::Editing(key))));
    let on_delete = Callback::new(move |key: String| {
        entries.update(|all| all.retain(|dated| dated.entry.key != key));
    });
    view! {
        <div class="max-w-3xl">
            <ActivityFeed entries=shown actions=log_button on_edit=on_edit on_delete=on_delete />
            {form_dialog(entries, composing)}
        </div>
    }
    .into_any()
}
