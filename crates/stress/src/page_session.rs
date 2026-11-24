use std::sync::{Arc, Mutex};
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::js_protocol::runtime::{
    ConsoleApiCalledType, EvaluateParams, EventConsoleApiCalled, EventExceptionThrown, RemoteObject,
};
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use serde::de::DeserializeOwned;
use tokio::task::JoinHandle;

const PAGE_LOAD_TIMEOUT_MILLISECONDS: u64 = 15_000;
const HYDRATION_POLL_MILLISECONDS: u64 = 50;
const HYDRATION_SETTLE_MILLISECONDS: u64 = 300;
const DEVICE_SCALE_FACTOR: f64 = 1.0;
const WASM_LOADED_SCRIPT: &str =
    "performance.getEntriesByType('resource').some(entry => entry.name.endsWith('.wasm'))";

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub width_pixels: u32,
    pub height_pixels: u32,
}

type ErrorLog = Arc<Mutex<Vec<String>>>;

pub struct PageSession {
    page: Page,
    errors: ErrorLog,
    listeners: Vec<JoinHandle<()>>,
}

impl PageSession {
    pub async fn open(browser: &Browser, url: &str, viewport: Viewport) -> Result<Self, String> {
        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|error| error.to_string())?;
        let errors: ErrorLog = Arc::default();
        let listeners = vec![
            log_console_errors(&page, Arc::clone(&errors)).await?,
            log_uncaught_exceptions(&page, Arc::clone(&errors)).await?,
        ];
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
        while !self.evaluate::<bool>(WASM_LOADED_SCRIPT).await? {
            tokio::time::sleep(Duration::from_millis(HYDRATION_POLL_MILLISECONDS)).await;
        }
        tokio::time::sleep(Duration::from_millis(HYDRATION_SETTLE_MILLISECONDS)).await;
        Ok(())
    }

    pub fn page(&self) -> &Page {
        &self.page
    }

    pub fn errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .map(|errors| errors.clone())
            .unwrap_or_default()
    }

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
