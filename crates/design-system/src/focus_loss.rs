use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use leptos::prelude::on_cleanup;

pub(crate) fn while_mounted<Event>(handler: impl Fn(Event) + 'static) -> impl Fn(Event) + 'static {
    let torn_down = Arc::new(AtomicBool::new(false));
    let marks_teardown = Arc::clone(&torn_down);
    on_cleanup(move || marks_teardown.store(true, Ordering::Relaxed));
    move |event| {
        if !torn_down.load(Ordering::Relaxed) {
            handler(event);
        }
    }
}
