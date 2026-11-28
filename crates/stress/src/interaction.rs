use std::time::{Duration, Instant};

use chromiumoxide::Browser;

use crate::action::{Action, perform};
use stress::page_session::{PageSession, Viewport};

const ACTION_GUARD_MILLISECONDS: u64 = 3_000;
const SHRINK_BUDGET_SECONDS: u64 = 180;
const STACK_TRACE_MARKER: &str = "\n\nStack:";
const STACK_FRAME_MARKER: &str = "\n    at ";
const APP_CRATES: [&str; 2] = ["design_system::", "component_gallery::"];
const SYMBOL_HASH_MARKER: &str = "::h";
const SOURCE_LOCATION_MARKER: &str = " (";
const WASM_MODULE_PREFIX: &str = "component_gallery.wasm.";
const SMALLEST_CHUNK_COUNT: usize = 2;

pub struct Scene<'browser> {
    pub browser: &'browser Browser,
    pub url: String,
    pub origin: String,
    pub viewport: Viewport,
}

#[derive(Clone)]
pub struct Failure {
    pub steps: Vec<String>,
    pub error: String,
}

impl Failure {
    fn signature(&self) -> &str {
        let message = self
            .error
            .split(STACK_TRACE_MARKER)
            .next()
            .unwrap_or(&self.error);
        message.split(STACK_FRAME_MARKER).next().unwrap_or(message)
    }

    pub fn summary(&self) -> String {
        let frames: Vec<&str> = self.error.lines().filter_map(app_frame).collect();
        if frames.is_empty() {
            return self.signature().to_owned();
        }
        format!(
            "{}\n    in the app's own code, innermost first:\n      {}",
            self.signature(),
            frames.join("\n      ")
        )
    }
}

fn app_frame(line: &str) -> Option<&str> {
    let frame = line.trim().strip_prefix("at ")?;
    let frame = frame.split(SOURCE_LOCATION_MARKER).next().unwrap_or(frame);
    let frame = frame.strip_prefix(WASM_MODULE_PREFIX).unwrap_or(frame);
    let frame = frame
        .rsplit_once(SYMBOL_HASH_MARKER)
        .map_or(frame, |(symbol, _)| symbol);
    APP_CRATES
        .iter()
        .any(|app_crate| frame.starts_with(app_crate))
        .then_some(frame)
}

pub struct Shrunk {
    pub failure: Failure,
    pub replays: usize,
    pub reproduced: bool,
}

pub async fn replay(scene: &Scene<'_>, actions: &[Action]) -> Result<Option<Failure>, String> {
    let session = PageSession::open(scene.browser, &scene.url, scene.viewport).await?;
    let mut steps = Vec::new();
    let mut failure = session.first_error().map(|error| Failure {
        steps: Vec::new(),
        error,
    });
    for action in actions {
        if failure.is_some() {
            break;
        }
        let guard = Duration::from_millis(ACTION_GUARD_MILLISECONDS);
        let step = tokio::time::timeout(guard, perform(&session, action, &scene.origin))
            .await
            .unwrap_or_else(|_| "action could not be performed".to_owned());
        steps.push(step);
        session.settle().await;
        failure = session.first_error().map(|error| Failure {
            steps: steps.clone(),
            error,
        });
    }
    session.close().await;
    Ok(failure)
}

async fn reproduces(
    scene: &Scene<'_>,
    actions: &[Action],
    original: &Failure,
) -> Result<Option<Failure>, String> {
    let failure = replay(scene, actions).await?;
    Ok(failure.filter(|failure| failure.signature() == original.signature()))
}

fn without_chunk(actions: &[Action], start: usize, length: usize) -> Vec<Action> {
    let end = start.saturating_add(length);
    actions
        .iter()
        .enumerate()
        .filter(|(position, _)| *position < start || *position >= end)
        .map(|(_, action)| action.clone())
        .collect()
}

fn truncated(actions: &[Action], failure: &Failure) -> Vec<Action> {
    actions.iter().take(failure.steps.len()).cloned().collect()
}

/// Delta debugging, removing complements only: Zeller and Hildebrandt,
/// "Simplifying and Isolating Failure-Inducing Input" (2002),
/// <https://www.st.cs.uni-saarland.de/papers/tse2002/>
pub async fn shrink(
    scene: &Scene<'_>,
    actions: &[Action],
    original: Failure,
) -> Result<Shrunk, String> {
    let deadline = Instant::now().checked_add(Duration::from_secs(SHRINK_BUDGET_SECONDS));
    let within_budget = || deadline.is_some_and(|deadline| Instant::now() < deadline);
    let mut current = truncated(actions, &original);
    let mut replays: usize = 1;
    let Some(mut failure) = reproduces(scene, &current, &original).await? else {
        return Ok(Shrunk {
            failure: original,
            replays,
            reproduced: false,
        });
    };
    let mut chunk_count = SMALLEST_CHUNK_COUNT;
    while current.len() >= SMALLEST_CHUNK_COUNT && within_budget() {
        let search = smaller_failure(scene, &current, chunk_count, &original).await?;
        replays = replays.saturating_add(search.replays);
        if let Some((actions, found)) = search.found {
            current = actions;
            failure = found;
            chunk_count = chunk_count.saturating_sub(1).max(SMALLEST_CHUNK_COUNT);
            continue;
        }
        if chunk_count >= current.len() {
            break;
        }
        chunk_count = chunk_count.saturating_mul(2).min(current.len());
    }
    Ok(Shrunk {
        failure,
        replays,
        reproduced: true,
    })
}

struct ChunkSearch {
    found: Option<(Vec<Action>, Failure)>,
    replays: usize,
}

async fn smaller_failure(
    scene: &Scene<'_>,
    current: &[Action],
    chunk_count: usize,
    original: &Failure,
) -> Result<ChunkSearch, String> {
    let chunk_length = current.len().div_ceil(chunk_count).max(1);
    let mut replays: usize = 0;
    for start in (0..current.len()).step_by(chunk_length) {
        let candidate = without_chunk(current, start, chunk_length);
        replays = replays.saturating_add(1);
        if let Some(found) = reproduces(scene, &candidate, original).await? {
            let found = Some((truncated(&candidate, &found), found));
            return Ok(ChunkSearch { found, replays });
        }
    }
    Ok(ChunkSearch {
        found: None,
        replays,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drops(count: u32) -> Vec<Action> {
        (0..count)
            .map(|element_index| Action::DropFile { element_index })
            .collect()
    }

    fn element_indices(actions: &[Action]) -> Vec<u32> {
        actions
            .iter()
            .filter_map(|action| match action {
                Action::DropFile { element_index } => Some(*element_index),
                _ => None,
            })
            .collect()
    }

    fn failure_after(steps: usize, error: &str) -> Failure {
        Failure {
            steps: vec![String::new(); steps],
            error: error.to_owned(),
        }
    }

    #[test]
    fn removes_one_chunk() {
        assert_eq!(
            element_indices(&without_chunk(&drops(6), 2, 2)),
            [0, 1, 4, 5]
        );
    }

    #[test]
    fn removes_a_chunk_that_runs_past_the_end() {
        assert_eq!(
            element_indices(&without_chunk(&drops(5), 4, 3)),
            [0, 1, 2, 3]
        );
    }

    #[test]
    fn truncates_to_the_failing_step() {
        let failure = failure_after(3, "boom");
        assert_eq!(element_indices(&truncated(&drops(10), &failure)), [0, 1, 2]);
    }

    #[test]
    fn signature_ignores_the_stack_trace() {
        let first = failure_after(1, "panicked at a.rs:1\nmessage\n\nStack:\n\nError at 0x1");
        let second = failure_after(9, "panicked at a.rs:1\nmessage\n\nStack:\n\nError at 0x2");
        assert_eq!(first.signature(), second.signature());
    }

    #[test]
    fn summary_keeps_only_the_app_frames() {
        let failure = failure_after(
            1,
            "panicked at a.rs:1\n\nStack:\n\nError\n    at __wbg_new_1 (http://x/pkg/app.js:9:25)\n    at component_gallery.wasm.reactive_graph::traits::Get::get::h0123456789abcdef (http://x/pkg/app.wasm:1:2)\n    at component_gallery.wasm.design_system::listbox::__component_listbox::{{closure}}::h6cc0c57a578727d2 (http://x/pkg/app.wasm:3:4)",
        );
        assert_eq!(
            failure.summary(),
            "panicked at a.rs:1\n    in the app's own code, innermost first:\n      design_system::listbox::__component_listbox::{{closure}}"
        );
    }

    #[test]
    fn signature_of_an_exception_stops_at_its_first_frame() {
        let failure = failure_after(
            1,
            "page error: RuntimeError: unreachable\n    at component_gallery.wasm.__rust_abort (http://x/pkg/app.wasm:1:2)",
        );
        assert_eq!(failure.signature(), "page error: RuntimeError: unreachable");
    }

    #[test]
    fn signature_tells_different_panics_apart() {
        let first = failure_after(1, "panicked at a.rs:1\n\nStack:");
        let second = failure_after(1, "panicked at b.rs:7\n\nStack:");
        assert_ne!(first.signature(), second.signature());
    }
}
