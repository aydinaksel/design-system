use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::network::{
    EventLoadingFailed, EventLoadingFinished, EventRequestWillBeSent, EventResponseReceived,
    RequestId, Response,
};
use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use chromiumoxide::cdp::js_protocol::runtime::{
    ConsoleApiCalledType, EvaluateParams, EventConsoleApiCalled, EventExceptionThrown, RemoteObject,
};
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use serde::de::DeserializeOwned;
use tokio::task::JoinHandle;

const PAGE_LOAD_TIMEOUT_MILLISECONDS: u64 = 15_000;
const HYDRATION_POLL_MILLISECONDS: u64 = 50;
const NETWORK_QUIET_MILLISECONDS: u64 = 100;
const SETTLE_POLL_MILLISECONDS: u64 = 20;
const SETTLE_TIMEOUT_MILLISECONDS: u64 = 3_000;
const FRAME_WAIT_TIMEOUT_MILLISECONDS: u64 = 1_000;
const FIRST_FAILING_STATUS: i64 = 400;
const DELIBERATE_FAILURE_HEADER: &str = "x-deliberate-failure";
const DEVICE_SCALE_FACTOR: f64 = 1.0;
const FULL_STACK_TRACES_SCRIPT: &str = "Error.stackTraceLimit = 200;";
const HYDRATED_SCRIPT: &str = "document.documentElement.hasAttribute('data-hydrated')";
const TWO_FRAMES_SCRIPT: &str = "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve(true))))";

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub width_pixels: u32,
    pub height_pixels: u32,
}

type ErrorLog = Arc<Mutex<Vec<String>>>;

struct NetworkActivity {
    in_flight: HashSet<RequestId>,
    last_change: Instant,
}

impl NetworkActivity {
    fn change(&mut self, update: impl FnOnce(&mut HashSet<RequestId>)) {
        update(&mut self.in_flight);
        self.last_change = Instant::now();
    }

    fn is_quiet(&self) -> bool {
        self.in_flight.is_empty()
            && self.last_change.elapsed() >= Duration::from_millis(NETWORK_QUIET_MILLISECONDS)
    }
}

type SharedActivity = Arc<Mutex<NetworkActivity>>;

pub struct PageSession {
    page: Page,
    errors: ErrorLog,
    activity: SharedActivity,
    listeners: Vec<JoinHandle<()>>,
}

impl PageSession {
    pub async fn open(browser: &Browser, url: &str, viewport: Viewport) -> Result<Self, String> {
        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|error| error.to_string())?;
        let errors: ErrorLog = Arc::default();
        let activity = Arc::new(Mutex::new(NetworkActivity {
            in_flight: HashSet::new(),
            last_change: Instant::now(),
        }));
        let listeners = vec![
            log_console_errors(&page, Arc::clone(&errors)).await?,
            log_uncaught_exceptions(&page, Arc::clone(&errors)).await?,
            log_failed_responses(&page, Arc::clone(&errors)).await?,
            track_started_requests(&page, Arc::clone(&activity)).await?,
            track_finished_requests(&page, Arc::clone(&activity)).await?,
            track_failed_requests(&page, Arc::clone(&activity)).await?,
        ];
        page.execute(AddScriptToEvaluateOnNewDocumentParams::new(
            FULL_STACK_TRACES_SCRIPT,
        ))
        .await
        .map_err(|error| error.to_string())?;
        page.execute(SetDeviceMetricsOverrideParams::new(
            i64::from(viewport.width_pixels),
            i64::from(viewport.height_pixels),
            DEVICE_SCALE_FACTOR,
            false,
        ))
        .await
        .map_err(|error| error.to_string())?;
        let session = Self {
            page,
            errors,
            activity,
            listeners,
        };
        let page_load_timeout = Duration::from_millis(PAGE_LOAD_TIMEOUT_MILLISECONDS);
        let loaded = tokio::time::timeout(page_load_timeout, session.load(url)).await;
        if !matches!(loaded, Ok(Ok(()))) {
            record(
                &session.errors,
                "the page did not finish loading".to_owned(),
            );
        }
        Ok(session)
    }

    async fn load(&self, url: &str) -> Result<(), String> {
        self.page
            .goto(url)
            .await
            .map_err(|error| error.to_string())?;
        while !self.evaluate::<bool>(HYDRATED_SCRIPT).await? {
            tokio::time::sleep(Duration::from_millis(HYDRATION_POLL_MILLISECONDS)).await;
        }
        self.settle().await;
        Ok(())
    }

    pub async fn settle(&self) {
        let deadline =
            Instant::now().checked_add(Duration::from_millis(SETTLE_TIMEOUT_MILLISECONDS));
        let past_deadline = || deadline.is_none_or(|deadline| Instant::now() >= deadline);
        loop {
            let frame_wait = Duration::from_millis(FRAME_WAIT_TIMEOUT_MILLISECONDS);
            let frames = tokio::time::timeout(frame_wait, self.evaluate::<bool>(TWO_FRAMES_SCRIPT));
            let rendered = matches!(frames.await, Ok(Ok(true)));
            if (rendered && self.network_is_quiet()) || past_deadline() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(SETTLE_POLL_MILLISECONDS)).await;
        }
    }

    fn network_is_quiet(&self) -> bool {
        self.activity
            .lock()
            .is_ok_and(|activity| activity.is_quiet())
    }

    #[must_use]
    pub fn page(&self) -> &Page {
        &self.page
    }

    #[must_use]
    pub fn errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .map(|errors| errors.clone())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn first_error(&self) -> Option<String> {
        self.errors().into_iter().next()
    }

    pub async fn evaluate<Value: DeserializeOwned>(
        &self,
        expression: &str,
    ) -> Result<Value, String> {
        let parameters = EvaluateParams {
            await_promise: Some(true),
            return_by_value: Some(true),
            ..EvaluateParams::new(expression)
        };
        self.page
            .evaluate_expression(parameters)
            .await
            .map_err(|error| error.to_string())?
            .into_value()
            .map_err(|error| error.to_string())
    }

    pub async fn close(self) {
        for listener in &self.listeners {
            listener.abort();
        }
        if let Err(error) = self.page.close().await {
            eprintln!("could not close a page: {error}");
        }
    }
}

fn record(errors: &ErrorLog, error: String) {
    if let Ok(mut errors) = errors.lock() {
        errors.push(error);
    }
}

fn is_deliberate_failure(response: &Response) -> bool {
    response.headers.inner().as_object().is_some_and(|headers| {
        headers
            .keys()
            .any(|name| name.eq_ignore_ascii_case(DELIBERATE_FAILURE_HEADER))
    })
}

fn update_activity(activity: &SharedActivity, update: impl FnOnce(&mut HashSet<RequestId>)) {
    if let Ok(mut activity) = activity.lock() {
        activity.change(update);
    }
}

fn remote_text(object: &RemoteObject) -> String {
    object
        .value
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .or_else(|| object.description.clone())
        .unwrap_or_default()
}

async fn log_console_errors(page: &Page, errors: ErrorLog) -> Result<JoinHandle<()>, String> {
    let mut events = page
        .event_listener::<EventConsoleApiCalled>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(tokio::spawn(async move {
        while let Some(event) = events.next().await {
            if event.r#type != ConsoleApiCalledType::Error {
                continue;
            }
            let text: Vec<String> = event.args.iter().map(remote_text).collect();
            record(&errors, format!("console error: {}", text.join(" ")));
        }
    }))
}

async fn log_uncaught_exceptions(page: &Page, errors: ErrorLog) -> Result<JoinHandle<()>, String> {
    let mut events = page
        .event_listener::<EventExceptionThrown>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(tokio::spawn(async move {
        while let Some(event) = events.next().await {
            let details = &event.exception_details;
            let text = details
                .exception
                .as_ref()
                .map_or_else(|| details.text.clone(), remote_text);
            record(&errors, format!("page error: {text}"));
        }
    }))
}

async fn log_failed_responses(page: &Page, errors: ErrorLog) -> Result<JoinHandle<()>, String> {
    let mut events = page
        .event_listener::<EventResponseReceived>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(tokio::spawn(async move {
        while let Some(event) = events.next().await {
            let response = &event.response;
            if response.status >= FIRST_FAILING_STATUS && !is_deliberate_failure(response) {
                let status = response.status;
                let url = &response.url;
                record(&errors, format!("failed request: {status} for {url}"));
            }
        }
    }))
}

async fn track_started_requests(
    page: &Page,
    activity: SharedActivity,
) -> Result<JoinHandle<()>, String> {
    let mut events = page
        .event_listener::<EventRequestWillBeSent>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(tokio::spawn(async move {
        while let Some(event) = events.next().await {
            let request_id = event.request_id.clone();
            update_activity(&activity, |in_flight| {
                in_flight.insert(request_id);
            });
        }
    }))
}

async fn track_finished_requests(
    page: &Page,
    activity: SharedActivity,
) -> Result<JoinHandle<()>, String> {
    let mut events = page
        .event_listener::<EventLoadingFinished>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(tokio::spawn(async move {
        while let Some(event) = events.next().await {
            update_activity(&activity, |in_flight| {
                in_flight.remove(&event.request_id);
            });
        }
    }))
}

async fn track_failed_requests(
    page: &Page,
    activity: SharedActivity,
) -> Result<JoinHandle<()>, String> {
    let mut events = page
        .event_listener::<EventLoadingFailed>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(tokio::spawn(async move {
        while let Some(event) = events.next().await {
            update_activity(&activity, |in_flight| {
                in_flight.remove(&event.request_id);
            });
        }
    }))
}
