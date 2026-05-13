use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{CountResponse, CreateProject, ErrorResponse, Project, Task, UpdateProject};

fn status_for_error(e: &str) -> StatusCode {
    if e.contains("Can't get") || e.contains("doesn't understand") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[utoipa::path(
    get,
    path = "/projects",
    tag = "projects",
    responses(
        (status = 200, description = "List of projects", body = [Project]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_projects() -> impl IntoResponse {
    match commands::get_projects() {
        Ok(projects) => (StatusCode::OK, Json(serde_json::json!(projects))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "projects",
    params(("id" = String, Path, description = "Things 3 project ID")),
    responses(
        (status = 200, description = "The requested project", body = Project),
        (status = 404, description = "Project not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn get_project(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_project_by_id(&id) {
        Ok(p) => (StatusCode::OK, Json(serde_json::json!(p))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/projects",
    tag = "projects",
    request_body = CreateProject,
    responses(
        (status = 201, description = "Project created", body = Project),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn create_project(Json(payload): Json<CreateProject>) -> impl IntoResponse {
    match commands::create_project(&payload) {
        Ok(p) => (StatusCode::CREATED, Json(serde_json::json!(p))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/projects/{id}",
    tag = "projects",
    params(("id" = String, Path, description = "Things 3 project ID")),
    request_body = UpdateProject,
    responses(
        (status = 200, description = "Updated project", body = Project),
        (status = 404, description = "Project not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn update_project(
    Path(id): Path<String>,
    Json(payload): Json<UpdateProject>,
) -> impl IntoResponse {
    match commands::update_project(&id, &payload) {
        Ok(p) => (StatusCode::OK, Json(serde_json::json!(p))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "projects",
    params(("id" = String, Path, description = "Things 3 project ID")),
    responses(
        (status = 204, description = "Project deleted (sent to Trash)"),
        (status = 404, description = "Project not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn delete_project(Path(id): Path<String>) -> impl IntoResponse {
    match commands::delete_project(&id) {
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
    path = "/projects/count",
    tag = "projects",
    responses(
        (status = 200, description = "Number of projects", body = CountResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn count_projects() -> impl IntoResponse {
    match commands::count_projects() {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!(CountResponse {
                count,
                scope: "projects".to_string(),
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
    get,
    path = "/projects/{id}/tasks",
    tag = "projects",
    params(("id" = String, Path, description = "Things 3 project ID")),
    responses(
        (status = 200, description = "Tasks within the project", body = [Task]),
        (status = 404, description = "Project not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_project_tasks(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_project_tasks(&id) {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::json!(tasks))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/projects/{id}/edit",
    tag = "projects",
    params(("id" = String, Path, description = "Things 3 project ID")),
    responses(
        (status = 204, description = "Project opened for editing in Things UI"),
        (status = 404, description = "Project not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn edit_project(Path(id): Path<String>) -> impl IntoResponse {
    match commands::edit_project(&id) {
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
    path = "/projects/{id}/show",
    tag = "projects",
    params(("id" = String, Path, description = "Things 3 project ID")),
    responses(
        (status = 204, description = "Project focused in Things UI"),
        (status = 404, description = "Project not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn show_project(Path(id): Path<String>) -> impl IntoResponse {
    match commands::show_project(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}
