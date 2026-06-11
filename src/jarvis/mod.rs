// Zone 6 — Research/Stubs (server-gated)
pub mod bridge;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct JarvisManager {
    jarvis_dir: PathBuf,
    process: Arc<Mutex<Option<tokio::process::Child>>>,
    api_base: String,
    frontend_port: u16,
}

impl JarvisManager {
    pub fn new(jarvis_dir: PathBuf, frontend_port: u16) -> Self {
        Self {
            jarvis_dir,
            process: Arc::new(Mutex::new(None)),
            api_base: format!("http://127.0.0.1:{}", frontend_port),
            frontend_port,
        }
    }

    pub fn api_url(&self) -> &str {
        &self.api_base
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut proc_lock = self.process.lock().await;
        if proc_lock.is_some() {
            return Ok(());
        }

        let child = tokio::process::Command::new("python3")
            .arg("server.py")
            .arg("--port")
            .arg(self.frontend_port.to_string())
            .current_dir(&self.jarvis_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start Jarvis: {}", e))?;

        *proc_lock = Some(child);
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut proc_lock = self.process.lock().await;
        if let Some(mut child) = proc_lock.take() {
            child
                .kill()
                .await
                .map_err(|e| format!("Failed to stop Jarvis: {}", e))?;
            child
                .wait()
                .await
                .map_err(|e| format!("Failed to wait for Jarvis: {}", e))?;
        }
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let mut proc_lock = self.process.lock().await;
        if let Some(ref mut child) = *proc_lock {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *proc_lock = None;
                    false
                }
            }
        } else {
            false
        }
    }

    pub async fn restart(&self) -> Result<(), String> {
        self.stop().await?;
        self.start().await
    }
}
