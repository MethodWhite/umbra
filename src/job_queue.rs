use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::time::Duration;

pub enum Job {
    TrainModel { model_name: String, data_path: String },
    BrowserCollect { urls: Vec<String> },
    Backup { label: String },
    Cleanup,
}

pub struct JobQueue {
    sender: mpsc::UnboundedSender<Job>,
    handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Job>();

        let handle = tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                match job {
                    Job::TrainModel { model_name, data_path } => {
                        tracing::info!("[JobQueue] Training model '{}' from '{}'", model_name, data_path);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        tracing::info!("[JobQueue] Model '{}' training complete", model_name);
                    }
                    Job::BrowserCollect { urls } => {
                        tracing::info!("[JobQueue] Collecting {} URLs", urls.len());
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        tracing::info!("[JobQueue] URL collection complete");
                    }
                    Job::Backup { label } => {
                        tracing::info!("[JobQueue] Running backup: {}", label);
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        tracing::info!("[JobQueue] Backup '{}' complete", label);
                    }
                    Job::Cleanup => {
                        tracing::info!("[JobQueue] Running cleanup");
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        tracing::info!("[JobQueue] Cleanup complete");
                    }
                }
            }
        });

        Self {
            sender: tx,
            handle: Arc::new(Mutex::new(Some(handle))),
        }
    }

    pub fn enqueue(&self, job: Job) {
        if let Err(e) = self.sender.send(job) {
            tracing::warn!("[JobQueue] Failed to enqueue job: {}", e);
        }
    }

    pub async fn shutdown(&self) {
        let mut handle = self.handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}
