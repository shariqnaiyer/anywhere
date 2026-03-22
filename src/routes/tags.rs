use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::ErrorResponse;

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
