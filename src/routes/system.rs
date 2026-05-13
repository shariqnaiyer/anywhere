use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{
    AppInfo, ErrorResponse, ParseInput, QuickEntry, QuitRequest, Task, UpdateWindow, WindowInfo,
};

#[utoipa::path(
    post,
    path = "/system/log-completed",
    tag = "system",
    responses(
        (status = 204, description = "Completed items logged to Logbook"),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn log_completed_now() -> impl IntoResponse {
    match commands::log_completed_now() {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/system/quick-entry",
    tag = "system",
    request_body = QuickEntry,
    responses(
        (status = 204, description = "Quick entry panel shown"),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn show_quick_entry(Json(payload): Json<QuickEntry>) -> impl IntoResponse {
    match commands::show_quick_entry_panel(&payload) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/info",
    tag = "system",
    responses(
        (status = 200, description = "Things 3 application info", body = AppInfo),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn app_info() -> impl IntoResponse {
    match commands::get_app_info() {
        Ok(info) => (StatusCode::OK, Json(serde_json::json!(info))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/windows",
    tag = "system",
    responses(
        (status = 200, description = "Open Things 3 windows", body = [WindowInfo]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_windows() -> impl IntoResponse {
    match commands::get_windows() {
        Ok(w) => (StatusCode::OK, Json(serde_json::json!(w))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/windows/{id}",
    tag = "system",
    params(("id" = i64, Path, description = "Window id (from GET /windows)")),
    request_body = UpdateWindow,
    responses(
        (status = 204, description = "Window updated"),
        (status = 404, description = "Window not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn update_window(
    Path(id): Path<i64>,
    Json(payload): Json<UpdateWindow>,
) -> impl IntoResponse {
    match commands::update_window(id, &payload) {
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

#[utoipa::path(
    delete,
    path = "/windows/{id}",
    tag = "system",
    params(("id" = i64, Path, description = "Window id (from GET /windows)")),
    responses(
        (status = 204, description = "Window closed"),
        (status = 404, description = "Window not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn close_window(Path(id): Path<i64>) -> impl IntoResponse {
    match commands::close_window(id) {
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

#[utoipa::path(
    post,
    path = "/system/quit",
    tag = "system",
    request_body = QuitRequest,
    responses(
        (status = 204, description = "Things 3 quit"),
        (status = 400, description = "Confirmation flag not set", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn quit_app(Json(payload): Json<QuitRequest>) -> impl IntoResponse {
    if !payload.confirm {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!(ErrorResponse {
                error: "confirm must be true to quit Things 3".to_string()
            })),
        )
            .into_response();
    }
    match commands::quit_app() {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/tasks/parse",
    tag = "tasks",
    request_body = ParseInput,
    responses(
        (status = 201, description = "Task created from Quicksilver input", body = Task),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn parse_quicksilver(Json(payload): Json<ParseInput>) -> impl IntoResponse {
    match commands::parse_quicksilver(&payload) {
        Ok(t) => (StatusCode::CREATED, Json(serde_json::json!(t))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}
