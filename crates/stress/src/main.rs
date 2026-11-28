mod action;
mod interaction;
mod server_load;

use std::process::ExitCode;

use std::sync::Arc;

use chromiumoxide::Browser;
use rand::SeedableRng;
use rand::rngs::ChaCha8Rng;

use crate::action::random_actions;
use crate::interaction::{Scene, Shrunk, replay, shrink};
use stress::browser_pool::{BrowserWindow, Job, LaunchOptions, run_jobs};
use stress::page_session::{PageSession, Viewport};

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
const PARALLEL_BROWSERS: usize = 4;
const UNREPRODUCED_STEPS_SHOWN: usize = 10;
const CHROMIUM_VARIABLE: &str = "STRESS_CHROMIUM";
const WATCH_FLAG: &str = "--watch";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

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

fn jobs() -> Vec<Job> {
    VIEWPORTS
        .into_iter()
        .flat_map(|viewport| ROUTES.into_iter().map(move |route| (route, viewport)))
        .enumerate()
        .map(|(index, (route, viewport))| Job {
            index,
            route,
            viewport,
        })
        .collect()
}

async fn load_freshly(
    browser: Arc<Browser>,
    job: Job,
    base_url: String,
) -> Result<Vec<String>, String> {
    let width = job.viewport.width_pixels;
    println!("  loading {} at {width}px", job.route);
    let url = format!("{base_url}{}", job.route);
    let session = PageSession::open(&browser, &url, job.viewport).await?;
    let failures = session
        .errors()
        .into_iter()
        .map(|error| format!("{} at {width}px: {error}", job.route))
        .collect();
    session.close().await;
    Ok(failures)
}

fn shrunk_report(route: &str, viewport: Viewport, shrunk: &Shrunk) -> String {
    let steps = &shrunk.failure.steps;
    let (how, shown) = if shrunk.reproduced {
        let how = format!(
            "shortest sequence found in {} replays, {} steps",
            shrunk.replays,
            steps.len()
        );
        (how, steps.as_slice())
    } else {
        let skipped = steps.len().saturating_sub(UNREPRODUCED_STEPS_SHOWN);
        let how = format!(
            "did not reproduce on replay, showing the last {} of {} steps",
            steps.len().saturating_sub(skipped),
            steps.len()
        );
        (how, steps.get(skipped..).unwrap_or_default())
    };
    format!(
        "{route} at {}px: {}\n    {how}:\n      {}",
        viewport.width_pixels,
        shrunk.failure.summary(),
        shown.join("\n      ")
    )
}

async fn interact_randomly(
    browser: Arc<Browser>,
    job: Job,
    base_url: String,
    seed: u64,
) -> Result<Option<String>, String> {
    let Job {
        route, viewport, ..
    } = job;
    let width = viewport.width_pixels;
    println!("  {RANDOM_ACTIONS_PER_PAGE} random actions on {route} at {width}px");
    let mut generator = ChaCha8Rng::seed_from_u64(page_seed(seed, route, viewport));
    let actions = random_actions(&mut generator, RANDOM_ACTIONS_PER_PAGE);
    let scene = Scene {
        browser: &browser,
        url: format!("{base_url}{route}"),
        origin: base_url.clone(),
        viewport,
    };
    let Some(failure) = replay(&scene, &actions).await? else {
        return Ok(None);
    };
    println!("    {route} at {width}px failed, shrinking");
    let shrunk = shrink(&scene, &actions, failure).await?;
    Ok(Some(shrunk_report(route, viewport, &shrunk)))
}

async fn stress(arguments: &Arguments) -> Result<Vec<String>, String> {
    println!("stress seed {}", arguments.seed);
    println!("server hammering");
    let mut failures: Vec<String> = server_load::hammer(&arguments.base_url, &ROUTES)
        .await
        .into_iter()
        .map(|failure| format!("server: {failure}"))
        .collect();
    let options = LaunchOptions {
        chromium_path: arguments.chromium_path.clone(),
        window: arguments.window,
    };
    println!("fresh page loads, {PARALLEL_BROWSERS} browsers at once");
    let base_url = arguments.base_url.clone();
    let hydration = run_jobs(&options, jobs(), PARALLEL_BROWSERS, move |browser, job| {
        load_freshly(browser, job, base_url.clone())
    })
    .await?;
    failures.extend(
        hydration
            .into_iter()
            .flatten()
            .map(|failure| format!("hydration: {failure}")),
    );
    println!("random interaction, {PARALLEL_BROWSERS} browsers at once");
    let base_url = arguments.base_url.clone();
    let seed = arguments.seed;
    let interaction = run_jobs(&options, jobs(), PARALLEL_BROWSERS, move |browser, job| {
        interact_randomly(browser, job, base_url.clone(), seed)
    })
    .await?;
    failures.extend(
        interaction
            .into_iter()
            .flatten()
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
