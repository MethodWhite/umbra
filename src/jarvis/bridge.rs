use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDetail {
    pub id: String,
    pub prompt: String,
    pub status: String,
    pub working_dir: String,
    pub result: String,
    pub error: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub elapsed_seconds: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsStatusResponse {
    pub claude_code_installed: bool,
    pub calendar_accessible: bool,
    pub mail_accessible: bool,
    pub notes_accessible: bool,
    pub memory_count: i64,
    pub task_count: i64,
    pub server_port: u16,
    pub uptime_seconds: i64,
    pub env_keys_set: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageResponse {
    pub session: HashMap<String, serde_json::Value>,
    pub today: HashMap<String, serde_json::Value>,
    pub all_time: HashMap<String, serde_json::Value>,
}

pub struct JarvisApi {
    client: Client,
    base_url: String,
}

impl JarvisApi {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn health(&self) -> Result<HealthResponse, String> {
        self.client
            .get(format!("{}/api/health", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Health check failed: {}", e))?
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("Health parse failed: {}", e))
    }

    pub async fn create_task(&self, prompt: &str, working_dir: &str) -> Result<TaskResponse, String> {
        let body = serde_json::json!({
            "prompt": prompt,
            "working_dir": working_dir,
        });
        self.client
            .post(format!("{}/api/tasks", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Create task failed: {}", e))?
            .json::<TaskResponse>()
            .await
            .map_err(|e| format!("Task response parse failed: {}", e))
    }

    pub async fn list_tasks(&self) -> Result<Vec<TaskDetail>, String> {
        let resp = self
            .client
            .get(format!("{}/api/tasks", self.base_url))
            .send()
            .await
            .map_err(|e| format!("List tasks failed: {}", e))?
            .json::<TaskListResponse>()
            .await
            .map_err(|e| format!("Tasks parse failed: {}", e))?;
        Ok(resp.tasks)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<TaskDetail, String> {
        let resp = self
            .client
            .get(format!("{}/api/tasks/{}", self.base_url, task_id))
            .send()
            .await
            .map_err(|e| format!("Get task failed: {}", e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Task parse failed: {}", e))?;
        serde_json::from_value(resp["task"].clone())
            .map_err(|e| format!("Task deserialize failed: {}", e))
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<(), String> {
        self.client
            .delete(format!("{}/api/tasks/{}", self.base_url, task_id))
            .send()
            .await
            .map_err(|e| format!("Cancel task failed: {}", e))?;
        Ok(())
    }

    pub async fn list_projects(&self) -> Result<Vec<ProjectInfo>, String> {
        let resp = self
            .client
            .get(format!("{}/api/projects", self.base_url))
            .send()
            .await
            .map_err(|e| format!("List projects failed: {}", e))?
            .json::<ProjectListResponse>()
            .await
            .map_err(|e| format!("Projects parse failed: {}", e))?;
        Ok(resp.projects)
    }

    pub async fn settings_status(&self) -> Result<SettingsStatusResponse, String> {
        self.client
            .get(format!("{}/api/settings/status", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Settings status failed: {}", e))?
            .json::<SettingsStatusResponse>()
            .await
            .map_err(|e| format!("Settings parse failed: {}", e))
    }

    pub async fn usage(&self) -> Result<UsageResponse, String> {
        self.client
            .get(format!("{}/api/usage", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Usage fetch failed: {}", e))?
            .json::<UsageResponse>()
            .await
            .map_err(|e| format!("Usage parse failed: {}", e))
    }
}
