use std::collections::VecDeque;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use tokio::task::JoinHandle;

use crate::page_session::Viewport;

#[derive(Clone, Copy)]
pub enum BrowserWindow {
    Hidden,
    Shown,
}

#[derive(Clone)]
pub struct LaunchOptions {
    pub chromium_path: String,
    pub window: BrowserWindow,
}

#[derive(Clone, Copy)]
pub struct Job {
    pub index: usize,
    pub route: &'static str,
    pub viewport: Viewport,
}

type JobQueue = Arc<Mutex<VecDeque<Job>>>;
type Finished<Output> = Vec<(usize, Output)>;

fn profile_directory(worker: usize) -> PathBuf {
    std::env::temp_dir().join(format!("stress-browser-{}-{worker}", std::process::id()))
}

async fn launch(
    options: &LaunchOptions,
    profile: &Path,
) -> Result<(Browser, JoinHandle<()>), String> {
    let builder = BrowserConfig::builder()
        .chrome_executable(&options.chromium_path)
        .user_data_dir(profile)
        .no_sandbox()
        .viewport(None);
    let config = match options.window {
        BrowserWindow::Hidden => builder.new_headless_mode(),
        BrowserWindow::Shown => builder.with_head(),
    }
    .build()?;
    let (browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|error| error.to_string())?;
    let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });
    Ok((browser, handler_task))
}

async fn close(browser: Arc<Browser>, handler_task: JoinHandle<()>) {
    if let Ok(mut browser) = Arc::try_unwrap(browser)
        && let Err(error) = browser.close().await
    {
        eprintln!("could not close a browser: {error}");
    }
    handler_task.abort();
}

fn next_job(queue: &JobQueue) -> Option<Job> {
    queue.lock().ok().and_then(|mut jobs| jobs.pop_front())
}

async fn work_through<Output, Work, Running>(
    worker: usize,
    options: LaunchOptions,
    queue: JobQueue,
    work: Work,
) -> Result<Finished<Output>, String>
where
    Work: Fn(Arc<Browser>, Job) -> Running,
    Running: Future<Output = Result<Output, String>>,
{
    let profile = profile_directory(worker);
    let (browser, handler_task) = launch(&options, &profile).await?;
    let browser = Arc::new(browser);
    let mut finished = Vec::new();
    let mut outcome = Ok(());
    while let Some(job) = next_job(&queue) {
        match work(Arc::clone(&browser), job).await {
            Ok(output) => finished.push((job.index, output)),
            Err(error) => {
                outcome = Err(error);
                break;
            }
        }
    }
    close(browser, handler_task).await;
    if let Err(error) = std::fs::remove_dir_all(&profile) {
        eprintln!("could not remove {}: {error}", profile.display());
    }
    outcome.map(|()| finished)
}

pub async fn run_jobs<Output, Work, Running>(
    options: &LaunchOptions,
    jobs: Vec<Job>,
    browsers: usize,
    work: Work,
) -> Result<Vec<Output>, String>
where
    Output: Send + 'static,
    Work: Fn(Arc<Browser>, Job) -> Running + Clone + Send + 'static,
    Running: Future<Output = Result<Output, String>> + Send + 'static,
{
    let queue: JobQueue = Arc::new(Mutex::new(VecDeque::from(jobs)));
    let workers: Vec<JoinHandle<Result<Finished<Output>, String>>> = (0..browsers)
        .map(|worker| {
            tokio::spawn(work_through(
                worker,
                options.clone(),
                Arc::clone(&queue),
                work.clone(),
            ))
        })
        .collect();
    let mut finished = Vec::new();
    for worker in workers {
        finished.extend(worker.await.map_err(|error| error.to_string())??);
    }
    finished.sort_by_key(|(index, _)| *index);
    Ok(finished.into_iter().map(|(_, output)| output).collect())
}
