use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{ErrorResponse, ListInfo};

#[utoipa::path(
    get,
    path = "/lists",
    tag = "lists",
    responses(
        (status = 200, description = "The seven special Things 3 lists", body = [ListInfo]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_lists() -> impl IntoResponse {
    match commands::get_lists() {
        Ok(l) => (StatusCode::OK, Json(serde_json::json!(l))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/lists/{name}/show",
    tag = "lists",
    params(("name" = String, Path, description = "List name (e.g. Today, Inbox)")),
    responses(
        (status = 204, description = "List focused in Things UI"),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn show_list(Path(name): Path<String>) -> impl IntoResponse {
    match commands::show_list(&name) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}
