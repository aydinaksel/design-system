use std::time::Duration;

use futures::future::join_all;
use reqwest::{Client, StatusCode};

const SEQUENTIAL_REQUESTS_PER_ROUTE: usize = 20;
const CONCURRENT_REQUESTS_PER_ROUTE: usize = 40;
const REQUEST_TIMEOUT_SECONDS: u64 = 10;

async fn status_of(client: &Client, url: &str) -> Result<StatusCode, String> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    response.bytes().await.map_err(|error| error.to_string())?;
    Ok(status)
}

fn failed_request(outcome: &Result<StatusCode, String>) -> Option<String> {
    match outcome {
        Ok(StatusCode::OK) => None,
        Ok(status) => Some(status.to_string()),
        Err(error) => Some(error.clone()),
    }
}

async fn hammer_route(client: &Client, base_url: &str, route: &str) -> Option<String> {
    let url = format!("{base_url}{route}");
    let mut outcomes = Vec::new();
    for _ in 0..SEQUENTIAL_REQUESTS_PER_ROUTE {
        outcomes.push(status_of(client, &url).await);
    }
    let concurrent = (0..CONCURRENT_REQUESTS_PER_ROUTE).map(|_| status_of(client, &url));
    outcomes.extend(join_all(concurrent).await);
    let failures: Vec<String> = outcomes.iter().filter_map(failed_request).collect();
    let example = failures.first()?;
    Some(format!(
        "{route}: {} of {} requests failed, e.g. {example}",
        failures.len(),
        outcomes.len(),
    ))
}

pub async fn hammer(base_url: &str, routes: &[&str]) -> Vec<String> {
    let client = Client::new();
    let mut failures = Vec::new();
    for route in routes {
        println!("  hammering {route}");
        failures.extend(hammer_route(&client, base_url, route).await);
    }
    failures
}
