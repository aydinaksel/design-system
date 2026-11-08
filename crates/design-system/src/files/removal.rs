use leptos::prelude::*;

use super::upload::FileRemover;
use super::upload_queue::UploadQueue;
use super::StoredFile;
use crate::button::ButtonTone;
use crate::confirm_dialog::ConfirmDialog;
use crate::toast::notify_failure;

#[derive(Clone, Copy)]
pub(super) struct Removals {
    remover: StoredValue<Option<FileRemover>>,
    removing: RwSignal<Vec<String>>,
    removed: RwSignal<Vec<String>>,
    confirming: RwSignal<Option<StoredFile>>,
    confirm_open: RwSignal<bool>,
}

impl Removals {
    pub(super) fn new(remover: Option<FileRemover>) -> Self {
        Self {
            remover: StoredValue::new(remover),
            removing: RwSignal::new(Vec::new()),
            removed: RwSignal::new(Vec::new()),
            confirming: RwSignal::new(None),
            confirm_open: RwSignal::new(false),
        }
    }

    pub(super) fn allowed(self) -> bool {
        self.remover.with_value(Option::is_some)
    }

    pub(super) fn is_removing(self, key: &str) -> bool {
        self.removing
            .with(|keys| keys.iter().any(|removing| removing == key))
    }

    pub(super) fn removed_keys(self) -> Vec<String> {
        self.removed.get()
    }

    pub(super) fn forget_removed(self) {
        self.removed.set(Vec::new());
    }

    pub(super) fn ask(self, file: StoredFile) {
        self.confirming.set(Some(file));
        self.confirm_open.set(true);
    }

    fn confirmed(self, queue: UploadQueue) {
        let Some(file) = self.confirming.get_untracked() else {
            return;
        };
        let Some(removal) = self
            .remover
            .with_value(|remover| remover.as_ref().map(|remove| remove(file.clone())))
        else {
            return;
        };
        let key = file.key;
        let name = file.name;
        self.removing.update(|keys| keys.push(key.clone()));
        leptos::task::spawn_local(async move {
            let outcome = removal.await;
            self.removing
                .try_update(|keys| keys.retain(|removing| *removing != key));
            match outcome {
                Ok(()) => {
                    self.removed.try_update(|keys| keys.push(key.clone()));
                    queue.forget_uploaded(&key);
                }
                Err(reason) => notify_failure(format!("Couldn't Remove {name}"), Some(reason)),
            }
        });
    }

    pub(super) fn dialog(self, queue: UploadQueue) -> AnyView {
        let name = move || {
            self.confirming
                .get()
                .map(|file| file.name)
                .unwrap_or_default()
        };
        view! {
            <ConfirmDialog
                open=self.confirm_open
                title=Signal::derive(move || format!("Remove {}?", name()))
                body="It will be removed for everyone who can see this record."
                confirm_label="Remove"
                tone=ButtonTone::Danger
                on_confirm=Callback::new(move |()| self.confirmed(queue))
            />
        }
        .into_any()
    }
}
