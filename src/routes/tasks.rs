use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::applescript::commands;
use crate::models::{CreateTask, ErrorResponse, Task, TasksQuery, UpdateTask};

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
        Err(e) => {
            let status = if e.contains("Can't get") || e.contains("doesn't understand") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!(ErrorResponse { error: e }))).into_response()
        }
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
        Err(e) => {
            let status = if e.contains("Can't get") || e.contains("doesn't understand") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!(ErrorResponse { error: e }))).into_response()
        }
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
        Err(e) => {
            let status = if e.contains("Can't get") || e.contains("doesn't understand") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!(ErrorResponse { error: e }))).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/tasks/{id}",
    tag = "tasks",
    params(("id" = String, Path, description = "Things 3 task ID")),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn delete_task(Path(id): Path<String>) -> impl IntoResponse {
    match commands::delete_task(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let status = if e.contains("Can't get") || e.contains("doesn't understand") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!(ErrorResponse { error: e }))).into_response()
        }
    }
}
