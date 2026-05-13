use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub activation_date: Option<String>,
    pub list: Option<String>,
    pub project: Option<String>,
    pub area: Option<String>,
    pub contact: Option<String>,
    pub tags: Vec<String>,
    pub checklist_items: Vec<ChecklistItem>,
    pub completed: bool,
    pub canceled: bool,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub completion_date: Option<String>,
    pub cancellation_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ChecklistItem {
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub activation_date: Option<String>,
    pub area: Option<String>,
    pub tags: Vec<String>,
    pub completed: bool,
    pub canceled: bool,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub completion_date: Option<String>,
    pub cancellation_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub keyboard_shortcut: Option<String>,
    pub parent_tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Area {
    pub id: String,
    pub title: String,
    pub collapsed: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Contact {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ListInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TasksQuery {
    /// Which Things 3 list to read from. One of `inbox`, `today`, `upcoming`, `anytime`, `someday`, `logbook`, `trash`. Defaults to `inbox`.
    pub list: Option<String>,
    /// Maximum number of tasks to return.
    pub limit: Option<usize>,
    /// Number of tasks to skip from the start of the list.
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTask {
    pub title: String,
    pub notes: Option<String>,
    /// Date string parseable by AppleScript, e.g. `"March 25, 2026"`.
    pub due_date: Option<String>,
    /// Scheduled date ("when"). Date string parseable by AppleScript. Wired via the AppleScript `schedule` command.
    pub activation_date: Option<String>,
    /// One of `inbox`, `today`, `upcoming`, `anytime`, `someday`. Wired via AppleScript `move`.
    pub list: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Exact project name. Takes priority over `list`.
    pub project: Option<String>,
    /// Exact area name (alternative to `project`).
    pub area: Option<String>,
    /// Exact contact name (must exist in Things 3 contacts).
    pub contact: Option<String>,
    pub checklist_items: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub notes: Option<String>,
    /// New due date. Empty string clears it.
    pub due_date: Option<String>,
    /// New scheduled date ("when"). Empty string clears it. Wired via AppleScript `schedule`.
    pub activation_date: Option<String>,
    /// Move to list: `inbox`, `today`, `upcoming`, `anytime`, `someday`.
    pub list: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Move to project (by name). Empty string detaches the project.
    pub project: Option<String>,
    /// Move to area (by name). Empty string detaches the area.
    pub area: Option<String>,
    /// Assign to contact (by name). Empty string clears it.
    pub contact: Option<String>,
    pub completed: Option<bool>,
    /// Set the canceled status of the task.
    pub canceled: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProject {
    pub title: String,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub activation_date: Option<String>,
    pub area: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProject {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub activation_date: Option<String>,
    pub area: Option<String>,
    pub tags: Option<Vec<String>>,
    pub completed: Option<bool>,
    pub canceled: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateArea {
    pub title: String,
    pub tags: Option<Vec<String>>,
    pub collapsed: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateArea {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub collapsed: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTag {
    pub name: String,
    pub keyboard_shortcut: Option<String>,
    /// Parent tag name (for hierarchical tags).
    pub parent_tag: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub keyboard_shortcut: Option<String>,
    /// Parent tag name. Empty string detaches from parent.
    pub parent_tag: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateContact {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QuickEntry {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub tags: Option<Vec<String>>,
    /// If true, Things 3 autofills from the foreground app (frontmost-window-aware capture).
    pub autofill: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ParseInput {
    /// Natural-language Quicksilver input, e.g. `"Buy milk @home #shopping !tomorrow"`.
    pub text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub frontmost: bool,
    /// Name of the list/area/project currently focused in the Things UI.
    pub current_list_name: Option<String>,
    /// URL of the list/area/project currently focused in the Things UI.
    pub current_list_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WindowInfo {
    pub id: i64,
    pub name: String,
    pub index: i64,
    pub bounds: [i64; 4],
    pub visible: bool,
    pub minimized: bool,
    pub zoomed: bool,
    pub closeable: bool,
    pub minimizable: bool,
    pub resizable: bool,
    pub zoomable: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWindow {
    /// Stacking order; 1 is frontmost.
    pub index: Option<i64>,
    /// `[x1, y1, x2, y2]` bounding rectangle in screen coordinates.
    pub bounds: Option<[i64; 4]>,
    pub visible: Option<bool>,
    pub minimized: Option<bool>,
    pub zoomed: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QuitRequest {
    /// Must be `true` to actually quit Things 3.
    pub confirm: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CountResponse {
    pub count: usize,
    pub scope: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}
