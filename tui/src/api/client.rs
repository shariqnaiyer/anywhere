//! Thin async wrapper over the things-api server. Injects `Authorization: Bearer <token>`
//! and decodes JSON. Errors flatten to a single `String` because we render them as
//! status-line toasts — the UI doesn't need to distinguish kinds.

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

use super::models::{
    Area, CreateProject, CreateTask, Project, ServerError, Tag, Task, UpdateTask,
};

#[derive(Clone)]
pub struct ApiClient {
    http: Client,
    base_url: String,
    token: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client");
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: token.into(),
        }
    }

    /// `GET /health` — no auth. Used during init to probe candidate endpoints.
    pub async fn ping(base_url: &str, timeout: Duration) -> Result<(), String> {
        let url = format!("{}/health", base_url.trim_end_matches('/'));
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| e.to_string())?;
        let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let resp = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        decode(resp).await
    }

    async fn post_empty(&self, path: &str) -> Result<(), String> {
        let resp = self
            .http
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        decode_empty(resp).await
    }

    async fn patch_empty(&self, path: &str) -> Result<(), String> {
        let resp = self
            .http
            .patch(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        decode_empty(resp).await
    }

    async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, String> {
        let resp = self
            .http
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        decode(resp).await
    }

    async fn patch_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, String> {
        let resp = self
            .http
            .patch(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        decode(resp).await
    }

    async fn delete_empty(&self, path: &str) -> Result<(), String> {
        let resp = self
            .http
            .delete(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        decode_empty(resp).await
    }

    pub async fn list_tasks(&self, list: Option<&str>) -> Result<Vec<Task>, String> {
        let path = match list {
            Some(l) => format!("/tasks?list={}&limit=500", urlencode(l)),
            None => "/tasks?limit=500".to_string(),
        };
        self.get(&path).await
    }

    pub async fn list_areas(&self) -> Result<Vec<Area>, String> {
        self.get("/areas").await
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, String> {
        self.get("/projects").await
    }

    pub async fn list_tags(&self) -> Result<Vec<Tag>, String> {
        self.get("/tags").await
    }

    pub async fn list_area_tasks(&self, id: &str) -> Result<Vec<Task>, String> {
        self.get(&format!("/areas/{}/tasks", urlencode(id))).await
    }

    pub async fn list_project_tasks(&self, id: &str) -> Result<Vec<Task>, String> {
        self.get(&format!("/projects/{}/tasks", urlencode(id))).await
    }

    pub async fn list_tag_tasks(&self, id: &str) -> Result<Vec<Task>, String> {
        self.get(&format!("/tags/{}/tasks", urlencode(id))).await
    }

    pub async fn create_task(&self, body: &CreateTask) -> Result<Task, String> {
        self.post_json("/tasks", body).await
    }

    pub async fn update_task(&self, id: &str, body: &UpdateTask) -> Result<Task, String> {
        self.patch_json(&format!("/tasks/{}", urlencode(id)), body).await
    }

    pub async fn complete_task(&self, id: &str) -> Result<(), String> {
        self.patch_empty(&format!("/tasks/{}/complete", urlencode(id))).await
    }

    pub async fn cancel_task(&self, id: &str) -> Result<(), String> {
        self.patch_empty(&format!("/tasks/{}/cancel", urlencode(id))).await
    }

    pub async fn delete_task(&self, id: &str) -> Result<(), String> {
        self.delete_empty(&format!("/tasks/{}", urlencode(id))).await
    }

    pub async fn create_project(&self, body: &CreateProject) -> Result<Project, String> {
        self.post_json("/projects", body).await
    }

    pub async fn empty_trash(&self) -> Result<(), String> {
        self.delete_empty("/trash").await
    }

    pub async fn show_quick_entry(&self) -> Result<(), String> {
        self.post_empty("/system/quick-entry").await
    }
}

async fn decode<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T, String> {
    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if status.is_success() {
        serde_json::from_str(&text).map_err(|e| format!("decode: {e} (body: {text})"))
    } else {
        Err(error_message(status, &text))
    }
}

async fn decode_empty(resp: reqwest::Response) -> Result<(), String> {
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(error_message(status, &text))
    }
}

fn error_message(status: StatusCode, body: &str) -> String {
    if let Ok(e) = serde_json::from_str::<ServerError>(body) {
        format!("{}: {}", status.as_u16(), e.error)
    } else if !body.is_empty() {
        format!("{}: {}", status.as_u16(), body)
    } else {
        format!("HTTP {}", status.as_u16())
    }
}

fn urlencode(s: &str) -> String {
    // Conservative: percent-encode anything that isn't a typical URL-safe byte. Things 3
    // IDs are short alphanumerics so this is rarely triggered.
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
