use design_system::icons::{self, LucideGlyph};
use design_system::{
    button_class, button_icon_class, ActivityDetail, ActivityEntry, ActivityFeed, ActivityKind,
    ButtonSize, ButtonTone, DialogSize, FormDialog, MarkdownEditor,
};
use leptos::prelude::*;

const ADD_SIZE: ButtonSize = ButtonSize::Compact;
const YOU: &str = "Aydin";
const NOTE_TITLE: &str = "Note";

#[derive(Clone, PartialEq, Eq)]
enum Composing {
    NewNote,
    EditNote(String),
}

fn email(
    key: &str,
    kind: ActivityKind,
    subject: &str,
    full_text: &str,
    people: &str,
    day: &str,
    time: &str,
) -> ActivityEntry {
    let preview = full_text.lines().next().unwrap_or_default().to_owned();
    ActivityEntry {
        key: key.to_owned(),
        kind,
        title: subject.to_owned(),
        detail: Some(ActivityDetail::Snippet {
            preview,
            full_text: full_text.to_owned(),
        }),
        people: people.to_owned(),
        day: day.to_owned(),
        time: time.to_owned(),
        href: Some("#".to_owned()),
        changeable: false,
    }
}

fn note(key: &str, text: &str, day: &str, time: &str) -> ActivityEntry {
    ActivityEntry {
        key: key.to_owned(),
        kind: ActivityKind::Note,
        title: NOTE_TITLE.to_owned(),
        detail: Some(ActivityDetail::Markdown(text.to_owned())),
        people: YOU.to_owned(),
        day: day.to_owned(),
        time: time.to_owned(),
        href: None,
        changeable: true,
    }
}

fn sample_entries() -> Vec<ActivityEntry> {
    vec![
        email(
            "email-1",
            ActivityKind::EmailReceived,
            "Re: revised ETA for MSKU4471029",
            "Vessel delayed at Algeciras, new ETA 28 November.\n\nThe carrier has rolled the booking by one sailing. Customs documents are unchanged.",
            "Sarah Doyle \u{2192} Aydin",
            "Today",
            "14:02",
        ),
        note(
            "note-1",
            "Supplier confirmed **13 crates** instead of 12. Updated line B.",
            "Today",
            "11:40",
        ),
        email(
            "email-2",
            ActivityKind::EmailSent,
            "Quote Q-2318 for 14 slabs",
            "Please find the quote attached.\n\nPrices hold until the end of the month.",
            "Aydin \u{2192} Tom Reeve",
            "Today",
            "09:15",
        ),
        ActivityEntry {
            key: "call-1".to_owned(),
            kind: ActivityKind::Call,
            title: "Call with Tom Reeve \u{b7} 12 min".to_owned(),
            detail: Some(ActivityDetail::Text(
                "Agreed to split the order across two containers.".to_owned(),
            )),
            people: "Melis".to_owned(),
            day: "Yesterday".to_owned(),
            time: "16:15".to_owned(),
            href: None,
            changeable: false,
        },
        ActivityEntry {
            key: "meeting-1".to_owned(),
            kind: ActivityKind::Meeting,
            title: "Showroom visit with Priya Shah".to_owned(),
            detail: Some(ActivityDetail::Text(
                "Selected two quartz options for the Richmond project.".to_owned(),
            )),
            people: "Aydin".to_owned(),
            day: "Yesterday".to_owned(),
            time: "11:00".to_owned(),
            href: None,
            changeable: false,
        },
        ActivityEntry {
            key: "event-1".to_owned(),
            kind: ActivityKind::Event,
            title: "Status changed to Sent".to_owned(),
            detail: None,
            people: "Aydin".to_owned(),
            day: "12 November".to_owned(),
            time: "09:03".to_owned(),
            href: None,
            changeable: false,
        },
    ]
}

fn note_text(entries: RwSignal<Vec<ActivityEntry>>, key: &str) -> String {
    entries.with_untracked(|all| {
        all.iter()
            .find(|entry| entry.key == key)
            .and_then(|entry| match &entry.detail {
                Some(ActivityDetail::Markdown(text)) => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default()
    })
}

fn save_note(entries: RwSignal<Vec<ActivityEntry>>, composing: Option<Composing>, text: String) {
    match composing {
        Some(Composing::EditNote(key)) => entries.update(|all| {
            if let Some(entry) = all.iter_mut().find(|entry| entry.key == key) {
                entry.detail = Some(ActivityDetail::Markdown(text));
            }
        }),
        Some(Composing::NewNote) => entries.update(|all| {
            let key = format!("note-{}", all.len());
            all.insert(0, note(&key, &text, "Today", "Now"));
        }),
        None => {}
    }
}

fn note_dialog(
    entries: RwSignal<Vec<ActivityEntry>>,
    composing: RwSignal<Option<Composing>>,
    text: RwSignal<String>,
) -> impl IntoView {
    move || {
        let target = composing.get()?;
        let title = match target {
            Composing::NewNote => "Add Note",
            Composing::EditNote(_) => "Edit Note",
        };
        Some(view! {
            <FormDialog
                title=title
                size=DialogSize::Large
                on_close=Callback::new(move |()| composing.set(None))
                on_submit=Callback::new(move |()| {
                    save_note(entries, composing.get_untracked(), text.get_untracked());
                    composing.set(None);
                })
                saving=Signal::stored(false)
                error=Signal::stored(None::<String>)
                can_submit=Signal::derive(move || text.with(|text| !text.trim().is_empty()))
                submit_label="Save Note"
            >
                <MarkdownEditor value=text placeholder="What should the team know?" />
            </FormDialog>
        })
    }
}

pub(crate) fn activity_feed() -> AnyView {
    let entries = RwSignal::new(sample_entries());
    let composing: RwSignal<Option<Composing>> = RwSignal::new(None);
    let text = RwSignal::new(String::new());
    let add_note = view! {
        <button
            type="button"
            class=button_class(ButtonTone::Secondary, ADD_SIZE)
            on:click=move |_| {
                text.set(String::new());
                composing.set(Some(Composing::NewNote));
            }
        >
            {icons::icon(LucideGlyph::Plus, button_icon_class(ADD_SIZE))}
            "Add Note"
        </button>
    }
    .into_any();
    let on_edit = Callback::new(move |key: String| {
        text.set(note_text(entries, &key));
        composing.set(Some(Composing::EditNote(key)));
    });
    let on_delete = Callback::new(move |key: String| {
        entries.update(|all| all.retain(|entry| entry.key != key));
    });
    view! {
        <div class="max-w-3xl">
            <ActivityFeed entries=entries actions=add_note on_edit=on_edit on_delete=on_delete />
            {note_dialog(entries, composing, text)}
        </div>
    }
    .into_any()
}
