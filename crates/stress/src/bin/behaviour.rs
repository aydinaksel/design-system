use std::future::Future;
use std::pin::Pin;
use std::process::ExitCode;
use std::sync::Arc;

use chromiumoxide::Browser;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use stress::browser_input::{click, click_at, press_named, replace_text};
use stress::browser_pool::{BrowserWindow, Job, LaunchOptions, run_jobs};
use stress::page_session::{PageSession, Viewport};

const DESKTOP: Viewport = Viewport {
    width_pixels: 1280,
    height_pixels: 900,
};
const PHONE: Viewport = Viewport {
    width_pixels: 390,
    height_pixels: 844,
};
const PARALLEL_BROWSERS: usize = 4;
const CHROMIUM_VARIABLE: &str = "STRESS_CHROMIUM";
const POSITION_TOLERANCE_PIXELS: f64 = 2.0;
const MENU_GAP_PIXELS: f64 = 4.0;
const POPOVER_GAP_PIXELS: f64 = 6.0;
const TEXTAREA_MAXIMUM_HEIGHT_PIXELS: f64 = 320.0;
const MENU_TRIGGER: &str = "[data-check-trigger]";
const OUTCOME_POLL_MILLISECONDS: u64 = 50;
const OUTCOME_TIMEOUT_MILLISECONDS: u64 = 5_000;
const MENU_PANEL: &str = "[data-check-panel]";

type CheckOutcome = Result<(), String>;
type RunningCheck = Pin<Box<dyn Future<Output = CheckOutcome> + Send>>;

struct Check {
    name: &'static str,
    path: &'static str,
    viewport: Viewport,
    run: fn(Arc<PageSession>) -> RunningCheck,
}

#[derive(Deserialize)]
struct Geometry {
    open: bool,
    trigger_top: f64,
    trigger_bottom: f64,
    trigger_left: f64,
    trigger_right: f64,
    panel_top: f64,
    panel_bottom: f64,
    panel_left: f64,
    panel_right: f64,
}

const GEOMETRY_SCRIPT: &str = "(async ([trigger, panel]) => { const element = document.querySelector(panel); await Promise.all(element.getAnimations().map(animation => animation.finished.catch(() => null))); const t = document.querySelector(trigger).getBoundingClientRect(); const p = element.getBoundingClientRect(); return {open: element.matches(':popover-open'), trigger_top: t.top, trigger_bottom: t.bottom, trigger_left: t.left, trigger_right: t.right, panel_top: p.top, panel_bottom: p.bottom, panel_left: p.left, panel_right: p.right}; })";
const ACTIVE_ELEMENT_SCRIPT: &str = "(() => { const active = document.activeElement; return active ? (active.getAttribute('aria-label') || active.textContent || active.tagName).trim().slice(0, 40) : ''; })()";
const MARK_MENU_SCRIPT: &str = "(() => { document.querySelector('button[aria-haspopup=menu]').setAttribute('data-check-trigger', ''); document.querySelector('[role=menu][popover]').setAttribute('data-check-panel', ''); return true; })()";

fn ensure(condition: bool, failure: impl FnOnce() -> String) -> CheckOutcome {
    if condition { Ok(()) } else { Err(failure()) }
}

fn ensure_close(actual: f64, expected: f64, what: &str) -> CheckOutcome {
    ensure(
        (actual - expected).abs() <= POSITION_TOLERANCE_PIXELS,
        || format!("{what} was {actual:.0}px, expected {expected:.0}px"),
    )
}

async fn value<Value: DeserializeOwned>(
    session: &PageSession,
    script: &str,
) -> Result<Value, String> {
    session.evaluate(script).await
}

async fn wait_until(session: &PageSession, condition: &str, what: &str) -> CheckOutcome {
    let deadline = std::time::Instant::now().checked_add(std::time::Duration::from_millis(
        OUTCOME_TIMEOUT_MILLISECONDS,
    ));
    loop {
        if value::<bool>(session, condition).await? {
            return Ok(());
        }
        if deadline.is_none_or(|deadline| std::time::Instant::now() >= deadline) {
            return Err(format!("timed out waiting for {what}"));
        }
        tokio::time::sleep(std::time::Duration::from_millis(OUTCOME_POLL_MILLISECONDS)).await;
    }
}

async fn focused(session: &PageSession) -> Result<String, String> {
    value(session, ACTIVE_ELEMENT_SCRIPT).await
}

async fn geometry(session: &PageSession, trigger: &str, panel: &str) -> Result<Geometry, String> {
    let arguments = serde_json::to_string(&[trigger, panel]).map_err(|error| error.to_string())?;
    value(session, &format!("{GEOMETRY_SCRIPT}({arguments})")).await
}

async fn mark_the_first_menu(session: &PageSession) -> CheckOutcome {
    let marked: bool = value(session, MARK_MENU_SCRIPT).await?;
    ensure(marked, || "the page has no menu".to_owned())
}

fn menu_opens_below_its_trigger(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        mark_the_first_menu(&session).await?;
        click(&session, MENU_TRIGGER).await?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure(placed.open, || "the menu did not open".to_owned())?;
        ensure_close(
            placed.panel_top,
            placed.trigger_bottom + MENU_GAP_PIXELS,
            "the menu's top",
        )?;
        ensure_close(
            placed.panel_right,
            placed.trigger_right,
            "the menu's right edge",
        )
    })
}

fn menu_flips_above_near_the_bottom(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        mark_the_first_menu(&session).await?;
        let _: bool = value(&session, "(() => { document.querySelector('[data-check-trigger]').parentElement.style.marginTop = '650px'; return true; })()").await?;
        click(&session, MENU_TRIGGER).await?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure(placed.open, || "the menu did not open".to_owned())?;
        ensure_close(
            placed.panel_bottom,
            placed.trigger_top - MENU_GAP_PIXELS,
            "the menu's bottom",
        )
    })
}

fn outside_click_closes_a_menu_without_navigating(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        mark_the_first_menu(&session).await?;
        click(&session, MENU_TRIGGER).await?;
        let before: String = value(&session, "location.pathname").await?;
        click(&session, "nav a[href='/tables']").await?;
        let after: String = value(&session, "location.pathname").await?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure(!placed.open, || "the menu stayed open".to_owned())?;
        ensure(before == after, || {
            format!("the click navigated from {before} to {after}")
        })?;
        let focus = focused(&session).await?;
        ensure(focus == "Actions", || {
            format!("focus went to {focus:?}, not the menu button")
        })
    })
}

fn keyboard_opens_and_escape_closes_a_menu(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        mark_the_first_menu(&session).await?;
        let _: bool = value(
            &session,
            "(() => { document.querySelector('[data-check-trigger]').focus(); return true; })()",
        )
        .await?;
        press_named(&session, "ArrowDown").await?;
        let first = focused(&session).await?;
        ensure(first.starts_with("Edit"), || {
            format!("ArrowDown focused {first:?}, not the first item")
        })?;
        press_named(&session, "Escape").await?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure(!placed.open, || "Escape left the menu open".to_owned())?;
        let focus = focused(&session).await?;
        ensure(focus == "Actions", || {
            format!("Escape left focus on {focus:?}")
        })
    })
}

fn disabled_menu_items_are_reachable_with_their_reason(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        mark_the_first_menu(&session).await?;
        let _: bool = value(
            &session,
            "(() => { document.querySelector('[data-check-trigger]').focus(); return true; })()",
        )
        .await?;
        press_named(&session, "ArrowDown").await?;
        for _ in 0..4 {
            press_named(&session, "ArrowDown").await?;
        }
        let described: String = value(&session, "(() => { const active = document.activeElement; const reason = document.getElementById(active.getAttribute('aria-describedby') || ''); return active.getAttribute('aria-disabled') === 'true' && reason ? reason.textContent.trim() : ''; })()").await?;
        ensure(described.contains("Narrow the list"), || {
            format!("the fifth item was not a disabled item announcing its reason ({described:?})")
        })?;
        press_named(&session, "Enter").await?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure(placed.open, || {
            "Enter on a disabled item closed the menu".to_owned()
        })
    })
}

fn popover_opens_below_and_escape_returns_focus(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        let trigger: String = value(&session, "(() => { const trigger = [...document.querySelectorAll('main button[aria-haspopup=dialog]')].find(button => button.textContent.trim() === 'Open Popover'); if (!trigger) return 'none'; trigger.setAttribute('data-check-trigger', ''); return trigger.textContent.trim(); })()").await?;
        ensure(trigger == "Open Popover", || {
            "no Open Popover button".to_owned()
        })?;
        click(&session, MENU_TRIGGER).await?;
        let panel: String = value(&session, "(() => { const panel = document.querySelector('[role=dialog][popover]:popover-open'); if (!panel) return 'none open'; panel.setAttribute('data-check-panel', ''); return 'open'; })()").await?;
        ensure(panel == "open", || {
            "clicking Open Popover opened nothing".to_owned()
        })?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure_close(
            placed.panel_top,
            placed.trigger_bottom + POPOVER_GAP_PIXELS,
            "the popover's top",
        )?;
        ensure_close(
            placed.panel_left,
            placed.trigger_left,
            "the popover's left edge",
        )?;
        press_named(&session, "Escape").await?;
        let focus = focused(&session).await?;
        ensure(focus == "Open Popover", || {
            format!("Escape left focus on {focus:?}")
        })
    })
}

fn listbox_matches_its_trigger_and_choosing_closes_it(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        let _: bool = value(&session, "(() => { const trigger = document.querySelector('button[role=combobox]'); trigger.setAttribute('data-check-trigger', ''); trigger.parentElement.querySelector('[role=listbox][popover]').setAttribute('data-check-panel', ''); return true; })()").await?;
        click(&session, MENU_TRIGGER).await?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure_close(
            placed.panel_right - placed.panel_left,
            placed.trigger_right - placed.trigger_left,
            "the list's width",
        )?;
        click(&session, "[data-check-panel] [role=option]:nth-child(2)").await?;
        let chosen: String = value(
            &session,
            "document.querySelector('[data-check-trigger]').textContent.trim()",
        )
        .await?;
        ensure(chosen == "Polished", || {
            format!("choosing the second option showed {chosen:?}")
        })?;
        let placed = geometry(&session, MENU_TRIGGER, MENU_PANEL).await?;
        ensure(!placed.open, || {
            "the list stayed open after choosing".to_owned()
        })
    })
}

fn cell_editor_discards_on_click_away_and_saves_on_enter(
    session: Arc<PageSession>,
) -> RunningCheck {
    Box::pin(async move {
        let trigger = "table button[aria-haspopup=dialog]";
        let original: String = value(
            &session,
            "document.querySelector('table button[aria-haspopup=dialog]').textContent.trim()",
        )
        .await?;
        click(&session, trigger).await?;
        replace_text(
            &session,
            "[role=dialog][popover]:popover-open input",
            "DISCARDED1",
        )
        .await?;
        click_at(&session, 5.0, 450.0).await?;
        let after_click_away: String = value(
            &session,
            "document.querySelector('table button[aria-haspopup=dialog]').textContent.trim()",
        )
        .await?;
        ensure(after_click_away == original, || {
            format!("clicking away saved {after_click_away:?}")
        })?;
        click(&session, trigger).await?;
        let edited = format!(
            "{}{}",
            original
                .get(..original.len().saturating_sub(1))
                .unwrap_or_default(),
            if original.ends_with('4') { '5' } else { '4' }
        );
        replace_text(
            &session,
            "[role=dialog][popover]:popover-open input",
            &edited,
        )
        .await?;
        press_named(&session, "Enter").await?;
        wait_until(
            &session,
            "!document.querySelector('[role=dialog][popover]:popover-open')",
            "the editor to close after saving",
        )
        .await?;
        let saved: String = value(
            &session,
            "document.querySelector('table button[aria-haspopup=dialog]').textContent.trim()",
        )
        .await?;
        ensure(saved == edited, || {
            format!("Enter saved {saved:?}, expected {edited:?}")
        })
    })
}

fn dialog_is_modal_and_escape_returns_focus(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        click(&session, "main button").await?;
        let modal: bool = value(&session, "!!document.querySelector('dialog:modal')").await?;
        ensure(modal, || "the dialog did not open as a modal".to_owned())?;
        let inside: bool = value(
            &session,
            "document.querySelector('dialog:modal').contains(document.activeElement)",
        )
        .await?;
        ensure(inside, || "focus did not move into the dialog".to_owned())?;
        let locked: String = value(
            &session,
            "getComputedStyle(document.documentElement).overflow",
        )
        .await?;
        ensure(locked == "hidden", || {
            "the page behind still scrolls".to_owned()
        })?;
        press_named(&session, "Escape").await?;
        let closed: bool = value(&session, "!document.querySelector('dialog[open]')").await?;
        ensure(closed, || "Escape left the dialog open".to_owned())?;
        let focus = focused(&session).await?;
        ensure(focus == "Rename Item", || {
            format!("focus returned to {focus:?}")
        })
    })
}

fn phone_drawer_is_built_only_while_open(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        let before: u32 =
            value(&session, "document.querySelectorAll('dialog nav a').length").await?;
        ensure(before == 0, || {
            "the drawer's links exist before it opens".to_owned()
        })?;
        click(&session, "button[aria-label='Open Navigation']").await?;
        let open: u32 = value(
            &session,
            "document.querySelectorAll('dialog:modal nav a').length",
        )
        .await?;
        ensure(open > 0, || {
            "the drawer did not open as a modal with links".to_owned()
        })?;
        press_named(&session, "Escape").await?;
        let after: u32 =
            value(&session, "document.querySelectorAll('dialog nav a').length").await?;
        ensure(after == 0, || {
            "the drawer's links remain after closing".to_owned()
        })?;
        let focus = focused(&session).await?;
        ensure(focus == "Open Navigation", || {
            format!("focus returned to {focus:?}")
        })
    })
}

fn field_error_waits_until_the_field_is_left(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        click(&session, "button[aria-label='Edit Classification']").await?;
        let field: String = value(&session, "(() => { const row = [...document.querySelectorAll('dl > div')].find(candidate => candidate.querySelector('dt')?.textContent.includes('Thickness')); row.querySelector('input').setAttribute('data-check-field', ''); return 'marked'; })()").await?;
        ensure(field == "marked", || "no Thickness field".to_owned())?;
        replace_text(&session, "[data-check-field]", "abc").await?;
        let while_typing: String = value(
            &session,
            "document.querySelector('[data-check-field]').getAttribute('aria-invalid')",
        )
        .await?;
        ensure(while_typing == "false", || {
            "the error showed while still typing".to_owned()
        })?;
        press_named(&session, "Tab").await?;
        let after_leaving: String = value(
            &session,
            "document.querySelector('[data-check-field]').getAttribute('aria-invalid')",
        )
        .await?;
        ensure(after_leaving == "true", || {
            "leaving an invalid field showed no error".to_owned()
        })?;
        replace_text(&session, "[data-check-field]", "2.5").await?;
        let fixed: String = value(
            &session,
            "document.querySelector('[data-check-field]').getAttribute('aria-invalid')",
        )
        .await?;
        ensure(fixed == "false", || {
            "fixing the value kept the error".to_owned()
        })
    })
}

fn view_tabs_are_links_to_their_views(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        let links: Vec<(String, Option<String>)> = value(&session, "[...document.querySelectorAll('nav[aria-label=Views] a')].map(link => [link.getAttribute('href'), link.getAttribute('aria-current')])").await?;
        ensure(links.len() > 1, || "no view tabs".to_owned())?;
        ensure(
            links.iter().all(|(href, _)| href.starts_with("/tables")),
            || format!("a view tab has no link: {links:?}"),
        )?;
        ensure(
            links
                .iter()
                .filter(|(_, current)| current.as_deref() == Some("page"))
                .count()
                == 1,
            || "not exactly one tab is marked current".to_owned(),
        )?;
        click(&session, "nav[aria-label=Views] a:nth-child(2)").await?;
        let path: String = value(&session, "location.pathname + location.search").await?;
        ensure(path.contains("view="), || {
            format!("clicking a view tab went to {path}")
        })
    })
}

fn textarea_grows_up_to_its_maximum(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        let empty: f64 = value(
            &session,
            "document.querySelector('main textarea').getBoundingClientRect().height",
        )
        .await?;
        let long_text = (1..=40)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        replace_text(&session, "main textarea", &long_text).await?;
        let full: f64 = value(
            &session,
            "document.querySelector('main textarea').getBoundingClientRect().height",
        )
        .await?;
        ensure(full > empty, || "the textarea did not grow".to_owned())?;
        ensure_close(
            full,
            TEXTAREA_MAXIMUM_HEIGHT_PIXELS,
            "the full textarea's height",
        )
    })
}

fn disclosure_opens_and_closes(session: Arc<PageSession>) -> RunningCheck {
    Box::pin(async move {
        let start: Vec<bool> = value(
            &session,
            "[...document.querySelectorAll('main details')].map(details => details.open)",
        )
        .await?;
        ensure(start == [true, false], || {
            format!("disclosures started as {start:?}")
        })?;
        click(&session, "main details:nth-of-type(2) summary").await?;
        let after: Vec<bool> = value(
            &session,
            "[...document.querySelectorAll('main details')].map(details => details.open)",
        )
        .await?;
        ensure(after == [true, true], || {
            format!("clicking the second summary gave {after:?}")
        })
    })
}

const CHECKS: [Check; 14] = [
    Check {
        name: "a menu opens below its trigger",
        path: "/menus",
        viewport: DESKTOP,
        run: menu_opens_below_its_trigger,
    },
    Check {
        name: "a menu flips above near the bottom",
        path: "/menus",
        viewport: DESKTOP,
        run: menu_flips_above_near_the_bottom,
    },
    Check {
        name: "an outside click closes a menu without navigating",
        path: "/menus",
        viewport: DESKTOP,
        run: outside_click_closes_a_menu_without_navigating,
    },
    Check {
        name: "the keyboard opens a menu and Escape closes it",
        path: "/menus",
        viewport: DESKTOP,
        run: keyboard_opens_and_escape_closes_a_menu,
    },
    Check {
        name: "disabled menu items are reachable with their reason",
        path: "/menus",
        viewport: DESKTOP,
        run: disabled_menu_items_are_reachable_with_their_reason,
    },
    Check {
        name: "a popover opens below and Escape returns focus",
        path: "/popovers",
        viewport: DESKTOP,
        run: popover_opens_below_and_escape_returns_focus,
    },
    Check {
        name: "a listbox matches its trigger and choosing closes it",
        path: "/form-controls",
        viewport: DESKTOP,
        run: listbox_matches_its_trigger_and_choosing_closes_it,
    },
    Check {
        name: "the cell editor discards on click away and saves on Enter",
        path: "/cell-editing",
        viewport: DESKTOP,
        run: cell_editor_discards_on_click_away_and_saves_on_enter,
    },
    Check {
        name: "a dialog is modal and Escape returns focus",
        path: "/dialogs",
        viewport: DESKTOP,
        run: dialog_is_modal_and_escape_returns_focus,
    },
    Check {
        name: "the phone drawer is built only while open",
        path: "/sidebar/dashboard",
        viewport: PHONE,
        run: phone_drawer_is_built_only_while_open,
    },
    Check {
        name: "a field's error waits until the field is left",
        path: "/field-sections",
        viewport: DESKTOP,
        run: field_error_waits_until_the_field_is_left,
    },
    Check {
        name: "view tabs are links to their views",
        path: "/tables",
        viewport: DESKTOP,
        run: view_tabs_are_links_to_their_views,
    },
    Check {
        name: "a textarea grows up to its maximum",
        path: "/form-sections",
        viewport: DESKTOP,
        run: textarea_grows_up_to_its_maximum,
    },
    Check {
        name: "a disclosure opens and closes",
        path: "/disclosures",
        viewport: DESKTOP,
        run: disclosure_opens_and_closes,
    },
];

async fn run_check(browser: Arc<Browser>, job: Job, base_url: String) -> Result<String, String> {
    let Some(check) = CHECKS.get(job.index) else {
        return Err(format!("no check number {}", job.index));
    };
    let url = format!("{base_url}{}", check.path);
    let session = Arc::new(PageSession::open(&browser, &url, check.viewport).await?);
    let outcome = match session.first_error() {
        Some(error) => Err(format!("the page failed to load: {error}")),
        None => (check.run)(Arc::clone(&session)).await,
    };
    let console = session.first_error();
    if let Ok(session) = Arc::try_unwrap(session) {
        session.close().await;
    }
    let outcome = outcome.and_then(|()| {
        console.map_or(Ok(()), |error| {
            Err(format!("the page logged an error: {error}"))
        })
    });
    Ok(match outcome {
        Ok(()) => format!("pass  {}", check.name),
        Err(reason) => format!("FAIL  {}: {reason}", check.name),
    })
}

async fn behaviour(base_url: String, chromium_path: String) -> Result<Vec<String>, String> {
    let options = LaunchOptions {
        chromium_path,
        window: BrowserWindow::Hidden,
    };
    let jobs = CHECKS
        .iter()
        .enumerate()
        .map(|(index, check)| Job {
            index,
            route: check.path,
            viewport: check.viewport,
        })
        .collect();
    run_jobs(&options, jobs, PARALLEL_BROWSERS, move |browser, job| {
        run_check(browser, job, base_url.clone())
    })
    .await
}

#[tokio::main]
async fn main() -> ExitCode {
    let Some(base_url) = std::env::args().nth(1) else {
        eprintln!("usage: behaviour <base url>");
        return ExitCode::FAILURE;
    };
    let Ok(chromium_path) = std::env::var(CHROMIUM_VARIABLE) else {
        eprintln!("{CHROMIUM_VARIABLE} is not set: run this inside the dev shell on Linux.");
        return ExitCode::FAILURE;
    };
    match behaviour(base_url.trim_end_matches('/').to_owned(), chromium_path).await {
        Ok(lines) => {
            for line in &lines {
                println!("{line}");
            }
            let failures = lines.iter().filter(|line| line.starts_with("FAIL")).count();
            println!(
                "{} of {} checks passed",
                lines.len().saturating_sub(failures),
                lines.len()
            );
            if failures == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
