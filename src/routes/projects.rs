use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{ErrorResponse, Project};

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
