mod action;
mod interaction;
mod page_session;
mod server_load;

use std::process::ExitCode;

use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use rand::SeedableRng;
use rand::rngs::ChaCha8Rng;

use crate::action::random_actions;
use crate::interaction::{Scene, Shrunk, replay, shrink};
use crate::page_session::{PageSession, Viewport};

const ROUTES: [&str; 9] = [
    "/",
    "/tables",
    "/tables?mode=cards",
    "/tables?search=gran",
    "/client-side-table",
    "/files",
    "/tables?mode=calendar",
    "/charts",
    "/sidebar/dashboard",
];
const VIEWPORTS: [Viewport; 2] = [
    Viewport {
        width_pixels: 1280,
        height_pixels: 900,
    },
    Viewport {
        width_pixels: 390,
        height_pixels: 844,
    },
];
const RANDOM_ACTIONS_PER_PAGE: usize = 150;
const CHROMIUM_VARIABLE: &str = "STRESS_CHROMIUM";
const WATCH_FLAG: &str = "--watch";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

#[derive(Clone, Copy)]
enum BrowserWindow {
    Hidden,
    Shown,
}

struct Arguments {
    base_url: String,
    seed: u64,
    chromium_path: String,
    window: BrowserWindow,
}

fn arguments() -> Result<Arguments, String> {
    let mut given = std::env::args().skip(1);
    let usage = "usage: stress <base url> <seed> [--watch]";
    let base_url = given.next().ok_or(usage)?;
    let seed = given
        .next()
        .ok_or(usage)?
        .parse()
        .map_err(|error| format!("the seed must be a whole number: {error}"))?;
    let window = match given.next().as_deref() {
        None => BrowserWindow::Hidden,
        Some(WATCH_FLAG) => BrowserWindow::Shown,
        Some(_) => return Err(usage.to_owned()),
    };
    let chromium_path = std::env::var(CHROMIUM_VARIABLE).map_err(|error| {
        format!("{CHROMIUM_VARIABLE} ({error}): run this inside the dev shell on Linux.")
    })?;
    Ok(Arguments {
        base_url: base_url.trim_end_matches('/').to_owned(),
        seed,
        chromium_path,
        window,
    })
}

/// FNV-1a: <http://www.isthe.com/chongo/tech/comp/fnv/>
fn page_seed(seed: u64, route: &str, viewport: Viewport) -> u64 {
    let width = viewport.width_pixels.to_le_bytes();
    let bytes = seed
        .to_le_bytes()
        .into_iter()
        .chain(route.bytes())
        .chain(width);
    bytes.fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

async fn hydration_sweep(browser: &Browser, base_url: &str) -> Result<Vec<String>, String> {
    let mut failures = Vec::new();
    for viewport in VIEWPORTS {
        for route in ROUTES {
            println!("  loading {route} at {}px", viewport.width_pixels);
            let url = format!("{base_url}{route}");
            let session = PageSession::open(browser, &url, viewport).await?;
            let width = viewport.width_pixels;
            failures.extend(
                session
                    .errors()
                    .into_iter()
                    .map(|error| format!("{route} at {width}px: {error}")),
            );
            session.close().await;
        }
    }
    Ok(failures)
}

fn shrunk_report(route: &str, viewport: Viewport, shrunk: &Shrunk) -> String {
    let steps = shrunk.failure.steps.join("\n      ");
    let how = if shrunk.reproduced {
        format!(
            "shortest sequence found in {} replays, {} steps",
            shrunk.replays,
            shrunk.failure.steps.len()
        )
    } else {
        "did not reproduce on replay, showing the original steps".to_owned()
    };
    format!(
        "{route} at {}px: {}\n    {how}:\n      {steps}",
        viewport.width_pixels, shrunk.failure.error
    )
}

async fn random_interaction(
    browser: &Browser,
    arguments: &Arguments,
) -> Result<Vec<String>, String> {
    let base_url = arguments.base_url.as_str();
    let mut failures = Vec::new();
    for viewport in VIEWPORTS {
        for route in ROUTES {
            let width = viewport.width_pixels;
            println!("  {RANDOM_ACTIONS_PER_PAGE} random actions on {route} at {width}px");
            let mut generator =
                ChaCha8Rng::seed_from_u64(page_seed(arguments.seed, route, viewport));
            let actions = random_actions(&mut generator, RANDOM_ACTIONS_PER_PAGE);
            let scene = Scene {
                browser,
                url: format!("{base_url}{route}"),
                origin: base_url.to_owned(),
                viewport,
            };
            let Some(failure) = replay(&scene, &actions).await? else {
                continue;
            };
            println!("    failed, shrinking");
            let shrunk = shrink(&scene, &actions, failure).await?;
            failures.push(shrunk_report(route, viewport, &shrunk));
        }
    }
    Ok(failures)
}

async fn stress(arguments: &Arguments) -> Result<Vec<String>, String> {
    println!("stress seed {}", arguments.seed);
    println!("server hammering");
    let mut failures: Vec<String> = server_load::hammer(&arguments.base_url, &ROUTES)
        .await
        .into_iter()
        .map(|failure| format!("server: {failure}"))
        .collect();
    let builder = BrowserConfig::builder()
        .chrome_executable(&arguments.chromium_path)
        .no_sandbox()
        .viewport(None);
    let config = match arguments.window {
        BrowserWindow::Hidden => builder.new_headless_mode(),
        BrowserWindow::Shown => builder.with_head(),
    }
    .build()?;
    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|error| error.to_string())?;
    let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });
    println!("fresh page loads");
    let hydration = hydration_sweep(&browser, &arguments.base_url).await;
    println!("random interaction");
    let interaction = random_interaction(&browser, arguments).await;
    if let Err(error) = browser.close().await {
        eprintln!("could not close the browser: {error}");
    }
    handler_task.abort();
    failures.extend(
        hydration?
            .into_iter()
            .map(|failure| format!("hydration: {failure}")),
    );
    failures.extend(
        interaction?
            .into_iter()
            .map(|failure| format!("interaction: {failure}")),
    );
    Ok(failures)
}

#[tokio::main]
async fn main() -> ExitCode {
    let outcome = match arguments() {
        Ok(arguments) => stress(&arguments)
            .await
            .map(|failures| (arguments.seed, failures)),
        Err(error) => Err(error),
    };
    match outcome {
        Ok((_, failures)) if failures.is_empty() => {
            println!("stress passed");
            ExitCode::SUCCESS
        }
        Ok((seed, failures)) => {
            for failure in &failures {
                println!("FAIL {failure}");
            }
            println!("{} stress failures with seed {seed}", failures.len());
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHONE: Viewport = Viewport {
        width_pixels: 390,
        height_pixels: 844,
    };
    const DESKTOP: Viewport = Viewport {
        width_pixels: 1280,
        height_pixels: 900,
    };

    #[test]
    fn page_seed_is_stable() {
        assert_eq!(
            page_seed(1, "/tables", PHONE),
            page_seed(1, "/tables", PHONE)
        );
    }

    #[test]
    fn page_seed_differs_by_route_and_width() {
        let tables = page_seed(1, "/tables", PHONE);
        assert_ne!(tables, page_seed(1, "/files", PHONE));
        assert_ne!(tables, page_seed(1, "/tables", DESKTOP));
    }
}
