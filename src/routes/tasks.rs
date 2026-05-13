use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::applescript::commands;
use crate::models::{CountResponse, CreateTask, ErrorResponse, Task, TasksQuery, UpdateTask};

fn status_for_error(e: &str) -> StatusCode {
    if e.contains("Can't get") || e.contains("doesn't understand") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[utoipa::path(
    get,
    path = "/tasks",
    tag = "tasks",
    params(TasksQuery),
    responses(
        (status = 200, description = "List of tasks", body = [Task]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_tasks(Query(query): Query<TasksQuery>) -> impl IntoResponse {
    match commands::get_tasks(query.list.as_deref(), query.limit, query.offset) {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::json!(tasks))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tasks/selected",
    tag = "tasks",
    responses(
        (status = 200, description = "Tasks currently selected in the Things UI", body = [Task]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_selected_tasks() -> impl IntoResponse {
    match commands::get_selected_tasks() {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::json!(tasks))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tasks/{id}",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    responses(
        (status = 200, description = "The requested task", body = Task),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn get_task(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_task_by_id(&id) {
        Ok(task) => (StatusCode::OK, Json(serde_json::json!(task))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/tasks",
    tag = "tasks",
    request_body = CreateTask,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn create_task(Json(payload): Json<CreateTask>) -> impl IntoResponse {
    match commands::create_task(&payload) {
        Ok(task) => (StatusCode::CREATED, Json(serde_json::json!(task))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/tasks/{id}",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    request_body = UpdateTask,
    responses(
        (status = 200, description = "Updated task", body = Task),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn update_task(
    Path(id): Path<String>,
    Json(payload): Json<UpdateTask>,
) -> impl IntoResponse {
    match commands::update_task(&id, &payload) {
        Ok(task) => (StatusCode::OK, Json(serde_json::json!(task))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/tasks/{id}/complete",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    responses(
        (status = 200, description = "Completed task", body = Task),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn complete_task(Path(id): Path<String>) -> impl IntoResponse {
    match commands::complete_task(&id) {
        Ok(task) => (StatusCode::OK, Json(serde_json::json!(task))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/tasks/{id}/cancel",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    responses(
        (status = 200, description = "Canceled task", body = Task),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn cancel_task(Path(id): Path<String>) -> impl IntoResponse {
    match commands::cancel_task(&id) {
        Ok(task) => (StatusCode::OK, Json(serde_json::json!(task))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/tasks/{id}/show",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    responses(
        (status = 204, description = "Task focused in Things UI"),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn show_task(Path(id): Path<String>) -> impl IntoResponse {
    match commands::show_task(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/tasks/{id}/edit",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    responses(
        (status = 204, description = "Task opened for editing in Things UI"),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn edit_task(Path(id): Path<String>) -> impl IntoResponse {
    match commands::edit_task(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tasks/count",
    tag = "tasks",
    params(TasksQuery),
    responses(
        (status = 200, description = "Count of tasks in the given list", body = CountResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn count_tasks(Query(query): Query<TasksQuery>) -> impl IntoResponse {
    match commands::count_tasks(query.list.as_deref()) {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!(CountResponse {
                count,
                scope: query.list.clone().unwrap_or_else(|| "inbox".to_string()),
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/tasks/{id}",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    responses(
        (status = 204, description = "Task deleted (sent to Trash)"),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn delete_task(Path(id): Path<String>) -> impl IntoResponse {
    match commands::delete_task(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}
