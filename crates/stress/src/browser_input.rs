use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType, InsertTextParams,
};
use chromiumoxide::layout::Point;
use serde::Deserialize;

use crate::page_session::PageSession;

const SHIFT_MODIFIER: i64 = 8;
const NO_MODIFIERS: i64 = 0;
const CENTRE_SCRIPT: &str = "(selector) => { const element = document.querySelector(selector); if (!element) return null; element.scrollIntoView({block: 'center', inline: 'center'}); const box = element.getBoundingClientRect(); return {x: box.left + box.width / 2, y: box.top + box.height / 2}; }";
const SELECT_ALL_SCRIPT: &str = "(selector) => { const element = document.querySelector(selector); if (!element) return false; element.focus(); if (typeof element.select === 'function') element.select(); return true; }";

#[derive(Clone, Copy, Debug)]
pub struct KeyPress {
    pub label: &'static str,
    key: &'static str,
    code: &'static str,
    key_code: i64,
    text: Option<&'static str>,
    shift: bool,
}

impl KeyPress {
    const fn plain(key: &'static str, key_code: i64) -> Self {
        Self {
            label: key,
            key,
            code: key,
            key_code,
            text: None,
            shift: false,
        }
    }
}

pub const BACKSPACE: KeyPress = KeyPress::plain("Backspace", 8);

pub const KEYS: [KeyPress; 11] = [
    KeyPress::plain("Tab", 9),
    KeyPress {
        label: "Shift+Tab",
        shift: true,
        ..KeyPress::plain("Tab", 9)
    },
    KeyPress::plain("Escape", 27),
    KeyPress {
        text: Some("\r"),
        ..KeyPress::plain("Enter", 13)
    },
    KeyPress {
        label: "Space",
        key: " ",
        code: "Space",
        key_code: 32,
        text: Some(" "),
        shift: false,
    },
    KeyPress::plain("ArrowDown", 40),
    KeyPress::plain("ArrowUp", 38),
    KeyPress::plain("ArrowLeft", 37),
    KeyPress::plain("ArrowRight", 39),
    KeyPress::plain("Home", 36),
    KeyPress::plain("End", 35),
];

pub async fn press(session: &PageSession, key: KeyPress) -> Result<(), String> {
    let modifiers = if key.shift {
        SHIFT_MODIFIER
    } else {
        NO_MODIFIERS
    };
    let mut builder = DispatchKeyEventParams::builder()
        .key(key.key)
        .code(key.code)
        .windows_virtual_key_code(key.key_code)
        .native_virtual_key_code(key.key_code)
        .modifiers(modifiers);
    let key_down = match key.text {
        Some(text) => {
            builder = builder.text(text);
            DispatchKeyEventType::KeyDown
        }
        None => DispatchKeyEventType::RawKeyDown,
    };
    for event_type in [key_down, DispatchKeyEventType::KeyUp] {
        let event = builder.clone().r#type(event_type).build()?;
        session
            .page()
            .execute(event)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[must_use]
pub fn key_named(name: &str) -> Option<KeyPress> {
    KEYS.into_iter()
        .chain([BACKSPACE])
        .find(|key| key.label == name)
}

pub async fn press_named(session: &PageSession, name: &str) -> Result<(), String> {
    let key = key_named(name).ok_or_else(|| format!("no key named {name}"))?;
    press(session, key).await?;
    session.settle().await;
    Ok(())
}

#[derive(Deserialize)]
struct Centre {
    x: f64,
    y: f64,
}

fn call(script: &str, selector: &str) -> Result<String, String> {
    let encoded = serde_json::to_string(selector).map_err(|error| error.to_string())?;
    Ok(format!("({script})({encoded})"))
}

pub async fn click(session: &PageSession, selector: &str) -> Result<(), String> {
    let centre: Option<Centre> = session.evaluate(&call(CENTRE_SCRIPT, selector)?).await?;
    let centre = centre.ok_or_else(|| format!("nothing matches {selector}"))?;
    click_at(session, centre.x, centre.y).await
}

pub async fn click_at(session: &PageSession, x: f64, y: f64) -> Result<(), String> {
    session
        .page()
        .click(Point { x, y })
        .await
        .map_err(|error| error.to_string())?;
    session.settle().await;
    Ok(())
}

pub async fn replace_text(session: &PageSession, selector: &str, text: &str) -> Result<(), String> {
    let found: bool = session
        .evaluate(&call(SELECT_ALL_SCRIPT, selector)?)
        .await?;
    if !found {
        return Err(format!("nothing matches {selector}"));
    }
    if text.is_empty() {
        press(session, BACKSPACE).await?;
    } else {
        session
            .page()
            .execute(InsertTextParams::new(text))
            .await
            .map_err(|error| error.to_string())?;
    }
    session.settle().await;
    Ok(())
}
