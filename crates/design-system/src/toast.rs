use std::cell::Cell;
use std::time::Duration;

use leptos::prelude::*;

use crate::button::{icon_button_class, ButtonSize, ButtonTone};
use crate::icons::{self, LucideGlyph};

const VIEWPORT_CLASS: &str = "pointer-events-none fixed inset-x-4 bottom-4 z-[60] mx-auto flex max-w-sm flex-col gap-3 sm:left-auto sm:right-4 sm:mx-0 sm:w-full";
const TOAST_CLASS: &str = "pointer-events-auto flex w-full items-center gap-3 rounded-surface border border-gray-200 bg-surface p-4 text-gray-900 shadow-lg transition-[translate,opacity] duration-500 ease-[cubic-bezier(0.22,1,0.36,1)] starting:translate-y-[150%] starting:opacity-0 motion-reduce:transition-none";
const LEAVING_CLASS: &str = "translate-y-[150%] opacity-0";
const SUCCESS_VISIBLE_DURATION: Duration = Duration::from_secs(5);
const LEAVING_DURATION: Duration = Duration::from_millis(300);
const MOST_TOASTS_SHOWN: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToastKind {
    Success,
    Failure,
}

impl ToastKind {
    const fn glyph(self) -> LucideGlyph {
        match self {
            Self::Success => LucideGlyph::CircleCheck,
            Self::Failure => LucideGlyph::OctagonX,
        }
    }

    const fn icon_class(self) -> &'static str {
        match self {
            Self::Success => "size-4 shrink-0",
            Self::Failure => "size-4 shrink-0 text-red-600",
        }
    }

    const fn role(self) -> &'static str {
        match self {
            Self::Success => "status",
            Self::Failure => "alert",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ToastMessage {
    id: u64,
    kind: ToastKind,
    title: String,
    description: Option<String>,
    leaving: bool,
}

type ShownToasts = RwSignal<Vec<ToastMessage>>;

thread_local! {
    static SHOWN_TOASTS: Cell<Option<ShownToasts>> = const { Cell::new(None) };
    static NEXT_TOAST_ID: Cell<u64> = const { Cell::new(0) };
}

fn dismiss(toasts: ShownToasts, id: u64) {
    toasts.try_update(|shown| {
        shown
            .iter_mut()
            .filter(|toast| toast.id == id)
            .for_each(|toast| toast.leaving = true);
    });
    set_timeout(
        move || {
            toasts.try_update(|shown| shown.retain(|toast| toast.id != id));
        },
        LEAVING_DURATION,
    );
}

fn is_leaving(toasts: ShownToasts, id: u64) -> bool {
    toasts
        .try_with(|shown| shown.iter().any(|toast| toast.id == id && toast.leaving))
        .unwrap_or(true)
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toasts: ShownToasts = RwSignal::new(Vec::new());
    #[cfg(feature = "hydrate")]
    SHOWN_TOASTS.with(|shown| shown.set(Some(toasts)));
    view! {
        {children()}
        <section aria-label="Notifications" class=VIEWPORT_CLASS>
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                children=move |toast| toast_view(toast, toasts)
            />
        </section>
    }
    .into_any()
}

fn toast_view(toast: ToastMessage, toasts: ShownToasts) -> AnyView {
    let id = toast.id;
    let description = toast
        .description
        .map(|text| view! { <p class="text-sm text-gray-500">{text}</p> });
    let class = move || {
        if is_leaving(toasts, id) {
            format!("{TOAST_CLASS} {LEAVING_CLASS}")
        } else {
            TOAST_CLASS.to_owned()
        }
    };
    view! {
        <div role=toast.kind.role() class=class>
            {icons::icon(toast.kind.glyph(), toast.kind.icon_class())}
            <div class="flex min-w-0 flex-1 flex-col gap-1 break-words">
                <p class="text-sm font-medium">{toast.title}</p>
                {description}
            </div>
            <button
                type="button"
                aria-label="Dismiss"
                class=icon_button_class(ButtonTone::Ghost, ButtonSize::Dense)
                on:click=move |_| dismiss(toasts, id)
            >
                {icons::icon(LucideGlyph::X, "size-4")}
            </button>
        </div>
    }
    .into_any()
}

pub fn notify_success(title: impl Into<String>) {
    notify(ToastKind::Success, title.into(), None);
}

pub fn notify_failure(title: impl Into<String>, description: Option<String>) {
    notify(ToastKind::Failure, title.into(), description);
}

fn notify(kind: ToastKind, title: String, description: Option<String>) {
    let Some(toasts) = SHOWN_TOASTS.with(Cell::get) else {
        return;
    };
    let id = NEXT_TOAST_ID.with(|next| next.replace(next.get().wrapping_add(1)));
    let message = ToastMessage {
        id,
        kind,
        title,
        description,
        leaving: false,
    };
    toasts.try_update(|shown| push_within_limit(shown, message));
    if kind == ToastKind::Success {
        set_timeout(move || dismiss(toasts, id), SUCCESS_VISIBLE_DURATION);
    }
}

fn push_within_limit(shown: &mut Vec<ToastMessage>, message: ToastMessage) {
    shown.push(message);
    let excess = shown.len().saturating_sub(MOST_TOASTS_SHOWN);
    shown.drain(..excess);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(id: u64) -> ToastMessage {
        ToastMessage {
            id,
            kind: ToastKind::Success,
            title: format!("Toast {id}"),
            description: None,
            leaving: false,
        }
    }

    #[test]
    fn the_oldest_toast_makes_way_once_the_limit_is_reached() {
        let mut shown = Vec::new();
        for id in 0..5 {
            push_within_limit(&mut shown, message(id));
        }
        let ids: Vec<u64> = shown.iter().map(|toast| toast.id).collect();
        assert_eq!(ids, vec![2, 3, 4]);
    }
}
