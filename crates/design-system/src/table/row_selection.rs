use std::collections::BTreeSet;
use std::sync::Arc;

use leptos::prelude::*;

use super::data_table::RowAttribute;
use crate::checkbox::CheckState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionMode {
    Single,
    Multiple,
}

/// Lets the user pick rows, each with a checkbox. The picked rows are named
/// by `key_for_row` in `keys`, which the page owns, so a pick survives
/// sorting, filtering and paging. A row without a link is also picked by
/// clicking anywhere on it.
pub struct RowSelection<Row> {
    keys: RwSignal<BTreeSet<String>>,
    mode: SelectionMode,
    key_for_row: RowAttribute<Row>,
    range_anchor_key: RwSignal<Option<String>>,
}

impl<Row> Clone for RowSelection<Row> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys,
            mode: self.mode,
            key_for_row: Arc::clone(&self.key_for_row),
            range_anchor_key: self.range_anchor_key,
        }
    }
}

impl<Row> RowSelection<Row> {
    pub fn single(
        keys: RwSignal<BTreeSet<String>>,
        key_for_row: impl Fn(&Row) -> String + Send + Sync + 'static,
    ) -> Self {
        Self::new(keys, SelectionMode::Single, key_for_row)
    }

    pub fn multiple(
        keys: RwSignal<BTreeSet<String>>,
        key_for_row: impl Fn(&Row) -> String + Send + Sync + 'static,
    ) -> Self {
        Self::new(keys, SelectionMode::Multiple, key_for_row)
    }

    fn new(
        keys: RwSignal<BTreeSet<String>>,
        mode: SelectionMode,
        key_for_row: impl Fn(&Row) -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            keys,
            mode,
            key_for_row: Arc::new(key_for_row),
            range_anchor_key: RwSignal::new(None),
        }
    }

    pub(super) fn key_of(&self, row: &Row) -> String {
        (self.key_for_row)(row)
    }

    pub(super) fn row_key(&self) -> RowAttribute<Row> {
        Arc::clone(&self.key_for_row)
    }

    pub(super) fn picks_many(&self) -> bool {
        self.mode == SelectionMode::Multiple
    }

    pub(super) fn contains(&self, key: &str) -> bool {
        self.keys.with(|keys| keys.contains(key))
    }

    pub(super) fn selected_count(&self) -> usize {
        self.keys.with(BTreeSet::len)
    }

    pub(super) fn clear(&self) {
        self.keys.set(BTreeSet::new());
        self.range_anchor_key.set(None);
    }

    pub(super) fn pick(&self, key: &str, extends_range: bool, keys_in_view: &[String]) {
        let anchor_key = self.range_anchor_key.get_untracked();
        let run_anchor_key = anchor_key.filter(|_| extends_range && self.picks_many());
        let mode = self.mode;
        self.keys
            .update(|keys| pick_row(keys, mode, run_anchor_key.as_deref(), key, keys_in_view));
        self.range_anchor_key.set(Some(key.to_owned()));
    }

    pub(super) fn state_of_rows_in_view(&self, keys_in_view: &[String]) -> CheckState {
        self.keys.with(|keys| state_of(keys, keys_in_view))
    }

    pub(super) fn toggle_rows_in_view(&self, keys_in_view: &[String]) {
        self.keys.update(|keys| toggle_every(keys, keys_in_view));
        self.range_anchor_key.set(None);
    }
}

fn pick_row(
    keys: &mut BTreeSet<String>,
    mode: SelectionMode,
    run_anchor_key: Option<&str>,
    key: &str,
    keys_in_view: &[String],
) {
    match run_anchor_key.filter(|anchor_key| keys.contains(*anchor_key)) {
        Some(anchor_key) => select_run(keys, keys_in_view, anchor_key, key),
        None => toggle(keys, mode, key),
    }
}

fn toggle(keys: &mut BTreeSet<String>, mode: SelectionMode, key: &str) {
    if keys.remove(key) {
        return;
    }
    if mode == SelectionMode::Single {
        keys.clear();
    }
    keys.insert(key.to_owned());
}

fn select_run(keys: &mut BTreeSet<String>, keys_in_view: &[String], anchor_key: &str, key: &str) {
    let position_of = |wanted: &str| keys_in_view.iter().position(|shown| shown == wanted);
    let (Some(anchor_position), Some(position)) = (position_of(anchor_key), position_of(key))
    else {
        keys.insert(key.to_owned());
        return;
    };
    let first = anchor_position.min(position);
    let last = anchor_position.max(position);
    let run = keys_in_view
        .iter()
        .skip(first)
        .take(last.saturating_sub(first).saturating_add(1));
    keys.extend(run.cloned());
}

fn state_of(keys: &BTreeSet<String>, keys_in_view: &[String]) -> CheckState {
    let selected_in_view = keys_in_view
        .iter()
        .filter(|key| keys.contains(*key))
        .count();
    if selected_in_view == 0 {
        CheckState::Unchecked
    } else if selected_in_view == keys_in_view.len() {
        CheckState::Checked
    } else {
        CheckState::Mixed
    }
}

fn toggle_every(keys: &mut BTreeSet<String>, keys_in_view: &[String]) {
    if state_of(keys, keys_in_view) == CheckState::Checked {
        for key in keys_in_view {
            keys.remove(key);
        }
    } else {
        keys.extend(keys_in_view.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys_of(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    fn in_view(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn picking_one_row_drops_the_row_picked_before() {
        let mut keys = keys_of(&["a"]);
        toggle(&mut keys, SelectionMode::Single, "b");
        assert_eq!(keys, keys_of(&["b"]));
        toggle(&mut keys, SelectionMode::Single, "b");
        assert!(keys.is_empty());
    }

    #[test]
    fn picking_many_rows_keeps_the_rows_picked_before() {
        let mut keys = keys_of(&["a"]);
        toggle(&mut keys, SelectionMode::Multiple, "b");
        assert_eq!(keys, keys_of(&["a", "b"]));
        toggle(&mut keys, SelectionMode::Multiple, "a");
        assert_eq!(keys, keys_of(&["b"]));
    }

    #[test]
    fn a_run_covers_every_row_between_the_anchor_and_the_click_in_either_direction() {
        let shown = in_view(&["a", "b", "c", "d", "e"]);
        let mut downwards = keys_of(&["b"]);
        select_run(&mut downwards, &shown, "b", "d");
        assert_eq!(downwards, keys_of(&["b", "c", "d"]));
        let mut upwards = keys_of(&["e"]);
        select_run(&mut upwards, &shown, "e", "c");
        assert_eq!(upwards, keys_of(&["c", "d", "e"]));
    }

    #[test]
    fn a_run_from_an_anchor_no_longer_in_view_picks_only_the_clicked_row() {
        let mut keys = keys_of(&["gone"]);
        select_run(&mut keys, &in_view(&["a", "b"]), "gone", "b");
        assert_eq!(keys, keys_of(&["b", "gone"]));
    }

    #[test]
    fn a_run_does_not_start_from_a_row_that_is_no_longer_picked() {
        let shown = in_view(&["a", "b", "c", "d"]);
        let mut cleared_by_the_page = keys_of(&[]);
        pick_row(
            &mut cleared_by_the_page,
            SelectionMode::Multiple,
            Some("a"),
            "c",
            &shown,
        );
        assert_eq!(cleared_by_the_page, keys_of(&["c"]));
        let mut still_picked = keys_of(&["a"]);
        pick_row(
            &mut still_picked,
            SelectionMode::Multiple,
            Some("a"),
            "c",
            &shown,
        );
        assert_eq!(still_picked, keys_of(&["a", "b", "c"]));
    }

    #[test]
    fn the_header_reads_mixed_until_every_row_in_view_is_picked() {
        let shown = in_view(&["a", "b"]);
        assert_eq!(state_of(&keys_of(&[]), &shown), CheckState::Unchecked);
        assert_eq!(state_of(&keys_of(&["a"]), &shown), CheckState::Mixed);
        assert_eq!(state_of(&keys_of(&["a", "b"]), &shown), CheckState::Checked);
        assert_eq!(state_of(&keys_of(&["a"]), &[]), CheckState::Unchecked);
    }

    #[test]
    fn the_header_picks_every_row_in_view_and_leaves_rows_out_of_view_alone() {
        let shown = in_view(&["a", "b"]);
        let mut keys = keys_of(&["a", "elsewhere"]);
        toggle_every(&mut keys, &shown);
        assert_eq!(keys, keys_of(&["a", "b", "elsewhere"]));
        toggle_every(&mut keys, &shown);
        assert_eq!(keys, keys_of(&["elsewhere"]));
    }
}
