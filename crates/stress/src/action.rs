use chromiumoxide::cdp::browser_protocol::input::InsertTextParams;
use chromiumoxide::layout::Point;
use rand::RngExt;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

use stress::browser_input::{BACKSPACE, KEYS, KeyPress, press};
use stress::page_session::PageSession;

const ACTION_TIMEOUT_MILLISECONDS: u64 = 300;
const DROP_TARGET_COUNT: u32 = 2000;
const LONG_TEXT_CHARACTERS: usize = 300;
const TYPED_TEXT_SHOWN_CHARACTERS: usize = 20;

const INTERACTIVE_SELECTOR: &str = "button:not([disabled]), a[href], input:not([type=file]):not([disabled]), textarea, [role=combobox], [tabindex='0']";

const CANDIDATES_SCRIPT: &str = "(selector, origin) => [...document.querySelectorAll(selector)].map((element, index) => ({element, index})).filter(({element}) => {
    const box = element.getBoundingClientRect();
    const visible = box.width > 0 && box.height > 0 && getComputedStyle(element).visibility !== 'hidden';
    const leavesGallery = element.tagName === 'A' && (
        element.hasAttribute('download') || element.target === '_blank' || new URL(element.href, location.href).origin !== origin
    );
    return visible && !leavesGallery;
}).map(({index}) => index)";

const ACTIONABLE_TARGET_SCRIPT: &str =
    "(selector, index, timeoutMilliseconds) => new Promise(resolve => {
    const element = document.querySelectorAll(selector)[index];
    if (!element) return resolve(null);
    element.scrollIntoView({block: 'center', inline: 'center'});
    const started = performance.now();
    let previous = null;
    const check = () => {
        const box = element.getBoundingClientRect();
        const x = box.left + box.width / 2;
        const y = box.top + box.height / 2;
        const stable = previous !== null && previous.x === x && previous.y === y;
        const hit = document.elementFromPoint(x, y);
        const receivesEvents = hit !== null && element.contains(hit);
        if (stable && receivesEvents && box.width > 0 && box.height > 0) {
            return resolve({
                x,
                y,
                description: element.outerHTML.slice(0, 80),
                acceptsText: element.isContentEditable || element.tagName === 'TEXTAREA' || (element.tagName === 'INPUT' && ['text', 'search', 'email', 'tel', 'url', 'password', 'number'].includes(element.type)),
            });
        }
        if (performance.now() - started > timeoutMilliseconds) return resolve(null);
        previous = {x, y};
        requestAnimationFrame(check);
    };
    requestAnimationFrame(check);
})";

const SELECT_FOR_TYPING_SCRIPT: &str = "(selector, index) => {
    const element = document.querySelectorAll(selector)[index];
    element.focus();
    if (typeof element.select === 'function') element.select();
    return true;
}";

const DROP_SCRIPT: &str = "(index) => {
    const target = document.querySelectorAll('*')[index] || document.body;
    const transfer = new DataTransfer();
    transfer.items.add(new File([new Uint8Array([137, 80, 78, 71])], 'stress.png', {type: 'image/png'}));
    for (const type of ['dragenter', 'dragover', 'drop']) {
        target.dispatchEvent(new DragEvent(type, {dataTransfer: transfer, bubbles: true, cancelable: true}));
    }
    return true;
}";

#[derive(Clone, Debug)]
pub enum Action {
    Click { target: u32 },
    Type { target: u32, text: String },
    Press(KeyPress),
    DropFile { element_index: u32 },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionableTarget {
    x: f64,
    y: f64,
    description: String,
    accepts_text: bool,
}

fn typed_texts() -> [String; 7] {
    [
        "gran".to_owned(),
        String::new(),
        "a".repeat(LONG_TEXT_CHARACTERS),
        "<script>".to_owned(),
        "😀 ünïcödé".to_owned(),
        "0".to_owned(),
        "-1".to_owned(),
    ]
}

fn random_action(generator: &mut impl RngExt) -> Action {
    match generator.random_range(0..7_u8) {
        0..=2 => Action::Click {
            target: generator.random(),
        },
        3 => Action::Type {
            target: generator.random(),
            text: typed_texts().choose(generator).cloned().unwrap_or_default(),
        },
        4 | 5 => Action::Press(*KEYS.choose(generator).unwrap_or(&BACKSPACE)),
        _ => Action::DropFile {
            element_index: generator.random_range(0..DROP_TARGET_COUNT),
        },
    }
}

pub fn random_actions(generator: &mut impl RngExt, count: usize) -> Vec<Action> {
    (0..count).map(|_| random_action(generator)).collect()
}

fn call_script(script: &str, arguments: &[&dyn ScriptArgument]) -> Result<String, String> {
    let encoded: Vec<String> = arguments
        .iter()
        .map(|argument| argument.encode())
        .collect::<Result<_, _>>()?;
    Ok(format!("({script})({})", encoded.join(", ")))
}

trait ScriptArgument {
    fn encode(&self) -> Result<String, String>;
}

impl<Value: Serialize> ScriptArgument for Value {
    fn encode(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|error| error.to_string())
    }
}

pub async fn perform(session: &PageSession, action: &Action, origin: &str) -> String {
    let outcome = match action {
        Action::Press(key) => press(session, *key)
            .await
            .map(|()| format!("press {}", key.label)),
        Action::DropFile { element_index } => drop_file(session, *element_index).await,
        Action::Click { target } => act_on_candidate(session, *target, None, origin).await,
        Action::Type { target, text } => {
            act_on_candidate(session, *target, Some(text), origin).await
        }
    };
    outcome.unwrap_or_else(|_| "action could not be performed".to_owned())
}

async fn drop_file(session: &PageSession, element_index: u32) -> Result<String, String> {
    let script = call_script(DROP_SCRIPT, &[&element_index])?;
    session.evaluate::<bool>(&script).await?;
    Ok(format!("drop a file on element {element_index}"))
}

async fn act_on_candidate(
    session: &PageSession,
    target: u32,
    text: Option<&String>,
    origin: &str,
) -> Result<String, String> {
    let script = call_script(CANDIDATES_SCRIPT, &[&INTERACTIVE_SELECTOR, &origin])?;
    let candidates: Vec<u32> = session.evaluate(&script).await?;
    let Some(index) = pick(&candidates, target) else {
        return Ok("nothing to act on".to_owned());
    };
    let script = call_script(
        ACTIONABLE_TARGET_SCRIPT,
        &[&INTERACTIVE_SELECTOR, &index, &ACTION_TIMEOUT_MILLISECONDS],
    )?;
    let actionable: Option<ActionableTarget> = session.evaluate(&script).await?;
    let element = actionable.ok_or("the element never became clickable")?;
    match text.filter(|_| element.accepts_text) {
        Some(text) => fill(session, index, text, &element.description).await,
        None => click(session, &element).await,
    }
}

fn pick(candidates: &[u32], target: u32) -> Option<u32> {
    let count = u32::try_from(candidates.len()).ok()?;
    let position = usize::try_from(target.checked_rem(count)?).ok()?;
    candidates.get(position).copied()
}

async fn click(session: &PageSession, element: &ActionableTarget) -> Result<String, String> {
    session
        .page()
        .click(Point {
            x: element.x,
            y: element.y,
        })
        .await
        .map_err(|error| error.to_string())?;
    Ok(format!("click {}", element.description))
}

async fn fill(
    session: &PageSession,
    index: u32,
    text: &str,
    description: &str,
) -> Result<String, String> {
    let script = call_script(SELECT_FOR_TYPING_SCRIPT, &[&INTERACTIVE_SELECTOR, &index])?;
    session.evaluate::<bool>(&script).await?;
    if text.is_empty() {
        press(session, BACKSPACE).await?;
    } else {
        session
            .page()
            .execute(InsertTextParams::new(text))
            .await
            .map_err(|error| error.to_string())?;
    }
    let shown: String = text.chars().take(TYPED_TEXT_SHOWN_CHARACTERS).collect();
    Ok(format!("type {shown:?} into {description}"))
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::ChaCha8Rng;

    use super::*;

    #[test]
    fn picks_a_candidate_by_wrapping_the_target() {
        assert_eq!(pick(&[10, 20, 30], 4), Some(20));
    }

    #[test]
    fn picks_nothing_without_candidates() {
        assert_eq!(pick(&[], 4), None);
    }

    #[test]
    fn a_seed_always_gives_the_same_actions() {
        let first = random_actions(&mut ChaCha8Rng::seed_from_u64(7), 50);
        let second = random_actions(&mut ChaCha8Rng::seed_from_u64(7), 50);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
    }
}
