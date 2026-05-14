//! Subset of the things-api server's wire types, kept in sync by hand.
//! When the server adds fields we don't render, this client tolerates them
//! because `serde` ignores unknown fields by default.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub activation_date: Option<String>,
    #[serde(default)]
    pub list: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub contact: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub checklist_items: Vec<ChecklistItem>,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub canceled: bool,
    #[serde(default)]
    pub creation_date: Option<String>,
    #[serde(default)]
    pub modification_date: Option<String>,
    #[serde(default)]
    pub completion_date: Option<String>,
    #[serde(default)]
    pub cancellation_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ChecklistItem {
    pub title: String,
    #[serde(default)]
    pub completed: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Project {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub activation_date: Option<String>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub canceled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Area {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub keyboard_shortcut: Option<String>,
    #[serde(default)]
    pub parent_tag: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct CreateTask {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct UpdateTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct CreateProject {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ServerError {
    pub error: String,
}
