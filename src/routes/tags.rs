use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{Area, ErrorResponse, Tag};

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
