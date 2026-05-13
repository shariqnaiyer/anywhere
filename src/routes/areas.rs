use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{Area, CountResponse, CreateArea, ErrorResponse, Task, UpdateArea};

fn status_for_error(e: &str) -> StatusCode {
    if e.contains("Can't get") || e.contains("doesn't understand") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[utoipa::path(
    get,
    path = "/areas",
    tag = "areas",
    responses(
        (status = 200, description = "List of areas", body = [Area]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_areas() -> impl IntoResponse {
    match commands::get_areas() {
        Ok(areas) => (StatusCode::OK, Json(serde_json::json!(areas))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/areas/{id}",
    tag = "areas",
    params(("id" = String, Path, description = "Things 3 area ID")),
    responses(
        (status = 200, description = "The requested area", body = Area),
        (status = 404, description = "Area not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn get_area(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_area_by_id(&id) {
        Ok(a) => (StatusCode::OK, Json(serde_json::json!(a))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/areas",
    tag = "areas",
    request_body = CreateArea,
    responses(
        (status = 201, description = "Area created", body = Area),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn create_area(Json(payload): Json<CreateArea>) -> impl IntoResponse {
    match commands::create_area(&payload) {
        Ok(a) => (StatusCode::CREATED, Json(serde_json::json!(a))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/areas/{id}",
    tag = "areas",
    params(("id" = String, Path, description = "Things 3 area ID")),
    request_body = UpdateArea,
    responses(
        (status = 200, description = "Updated area", body = Area),
        (status = 404, description = "Area not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn update_area(
    Path(id): Path<String>,
    Json(payload): Json<UpdateArea>,
) -> impl IntoResponse {
    match commands::update_area(&id, &payload) {
        Ok(a) => (StatusCode::OK, Json(serde_json::json!(a))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/areas/{id}",
    tag = "areas",
    params(("id" = String, Path, description = "Things 3 area ID")),
    responses(
        (status = 204, description = "Area deleted"),
        (status = 404, description = "Area not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn delete_area(Path(id): Path<String>) -> impl IntoResponse {
    match commands::delete_area(&id) {
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
    path = "/areas/count",
    tag = "areas",
    responses(
        (status = 200, description = "Number of areas", body = CountResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn count_areas() -> impl IntoResponse {
    match commands::count_areas() {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!(CountResponse {
                count,
                scope: "areas".to_string(),
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
    path = "/areas/{id}/tasks",
    tag = "areas",
    params(("id" = String, Path, description = "Things 3 area ID")),
    responses(
        (status = 200, description = "Tasks within the area", body = [Task]),
        (status = 404, description = "Area not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_area_tasks(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_area_tasks(&id) {
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
    path = "/areas/{id}/show",
    tag = "areas",
    params(("id" = String, Path, description = "Things 3 area ID")),
    responses(
        (status = 204, description = "Area focused in Things UI"),
        (status = 404, description = "Area not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn show_area(Path(id): Path<String>) -> impl IntoResponse {
    match commands::show_area(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}
