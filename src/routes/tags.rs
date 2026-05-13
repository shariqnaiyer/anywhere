use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{CountResponse, CreateTag, ErrorResponse, Tag, Task, UpdateTag};

fn status_for_error(e: &str) -> StatusCode {
    if e.contains("Can't get") || e.contains("doesn't understand") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[utoipa::path(
    get,
    path = "/tags",
    tag = "tags",
    responses(
        (status = 200, description = "List of tags", body = [Tag]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_tags() -> impl IntoResponse {
    match commands::get_tags() {
        Ok(tags) => (StatusCode::OK, Json(serde_json::json!(tags))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tags/{id}",
    tag = "tags",
    params(("id" = String, Path, description = "Things 3 tag ID")),
    responses(
        (status = 200, description = "The requested tag", body = Tag),
        (status = 404, description = "Tag not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn get_tag(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_tag_by_id(&id) {
        Ok(t) => (StatusCode::OK, Json(serde_json::json!(t))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/tags",
    tag = "tags",
    request_body = CreateTag,
    responses(
        (status = 201, description = "Tag created", body = Tag),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn create_tag(Json(payload): Json<CreateTag>) -> impl IntoResponse {
    match commands::create_tag(&payload) {
        Ok(t) => (StatusCode::CREATED, Json(serde_json::json!(t))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/tags/{id}",
    tag = "tags",
    params(("id" = String, Path, description = "Things 3 tag ID")),
    request_body = UpdateTag,
    responses(
        (status = 200, description = "Updated tag", body = Tag),
        (status = 404, description = "Tag not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn update_tag(
    Path(id): Path<String>,
    Json(payload): Json<UpdateTag>,
) -> impl IntoResponse {
    match commands::update_tag(&id, &payload) {
        Ok(t) => (StatusCode::OK, Json(serde_json::json!(t))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tags/count",
    tag = "tags",
    responses(
        (status = 200, description = "Number of tags", body = CountResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn count_tags() -> impl IntoResponse {
    match commands::count_tags() {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!(CountResponse {
                count,
                scope: "tags".to_string(),
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
    path = "/tags/{id}/tasks",
    tag = "tags",
    params(("id" = String, Path, description = "Things 3 tag ID")),
    responses(
        (status = 200, description = "Tasks bearing the tag", body = [Task]),
        (status = 404, description = "Tag not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_tag_tasks(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_tag_tasks(&id) {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::json!(tasks))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/tags/{id}/children",
    tag = "tags",
    params(("id" = String, Path, description = "Things 3 tag ID")),
    responses(
        (status = 200, description = "Child tags (hierarchical)", body = [Tag]),
        (status = 404, description = "Tag not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_tag_children(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_tag_children(&id) {
        Ok(tags) => (StatusCode::OK, Json(serde_json::json!(tags))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/tags/{id}",
    tag = "tags",
    params(("id" = String, Path, description = "Things 3 tag ID")),
    responses(
        (status = 204, description = "Tag deleted"),
        (status = 404, description = "Tag not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn delete_tag(Path(id): Path<String>) -> impl IntoResponse {
    match commands::delete_tag(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}
