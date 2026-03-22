use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::ErrorResponse;

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
