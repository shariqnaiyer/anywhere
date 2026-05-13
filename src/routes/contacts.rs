use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::applescript::commands;
use crate::models::{Contact, CountResponse, CreateContact, ErrorResponse, Task};

fn status_for_error(e: &str) -> StatusCode {
    if e.contains("Can't get") || e.contains("doesn't understand") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[utoipa::path(
    get,
    path = "/contacts",
    tag = "contacts",
    responses(
        (status = 200, description = "List of contacts known to Things 3", body = [Contact]),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_contacts() -> impl IntoResponse {
    match commands::get_contacts() {
        Ok(c) => (StatusCode::OK, Json(serde_json::json!(c))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/contacts",
    tag = "contacts",
    request_body = CreateContact,
    responses(
        (status = 201, description = "Contact created", body = Contact),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn create_contact(Json(payload): Json<CreateContact>) -> impl IntoResponse {
    match commands::create_contact(&payload) {
        Ok(c) => (StatusCode::CREATED, Json(serde_json::json!(c))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/contacts/count",
    tag = "contacts",
    responses(
        (status = 200, description = "Number of contacts", body = CountResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn count_contacts() -> impl IntoResponse {
    match commands::count_contacts() {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!(CountResponse {
                count,
                scope: "contacts".to_string(),
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
    path = "/contacts/{id}/tasks",
    tag = "contacts",
    params(("id" = String, Path, description = "Things 3 contact ID")),
    responses(
        (status = 200, description = "Tasks assigned to the contact", body = [Task]),
        (status = 404, description = "Contact not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn list_contact_tasks(Path(id): Path<String>) -> impl IntoResponse {
    match commands::get_contact_tasks(&id) {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::json!(tasks))).into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/contacts/{id}",
    tag = "contacts",
    params(("id" = String, Path, description = "Things 3 contact ID")),
    responses(
        (status = 204, description = "Contact deleted"),
        (status = 404, description = "Contact not found", body = ErrorResponse),
        (status = 500, description = "AppleScript error", body = ErrorResponse),
    ),
)]
pub async fn delete_contact(Path(id): Path<String>) -> impl IntoResponse {
    match commands::delete_contact(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            status_for_error(&e),
            Json(serde_json::json!(ErrorResponse { error: e })),
        )
            .into_response(),
    }
}
