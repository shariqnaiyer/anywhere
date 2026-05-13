use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{cloudflare::CfError, db, AppState};

pub async fn health() -> &'static str {
    "ok"
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub username: String,
    pub url: String,
    pub tunnel_token: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

const RESERVED: &[&str] = &[
    "admin", "api", "app", "auth", "billing", "blog", "cdn", "dashboard",
    "docs", "help", "info", "login", "logout", "mail", "ns", "ns1", "ns2",
    "root", "signup", "static", "status", "support", "test", "www",
];

fn validate_username(u: &str) -> Result<(), String> {
    if u.len() < 3 {
        return Err("username must be at least 3 characters".into());
    }
    if u.len() > 32 {
        return Err("username must be at most 32 characters".into());
    }
    let ok = u
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !ok {
        return Err("username may only contain a-z, 0-9, hyphen".into());
    }
    if u.starts_with('-') || u.ends_with('-') {
        return Err("username may not begin or end with a hyphen".into());
    }
    if RESERVED.contains(&u) {
        return Err("username is reserved".into());
    }
    Ok(())
}

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!(ErrorResponse { error: msg.into() })),
    )
}

fn server_error(msg: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!(ErrorResponse { error: msg.into() })),
    )
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignupRequest>,
) -> impl IntoResponse {
    let username = payload.username.trim().to_lowercase();
    if let Err(msg) = validate_username(&username) {
        return bad_request(msg).into_response();
    }

    match db::username_taken(&state.db, &username).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!(ErrorResponse {
                    error: "username already taken".into()
                })),
            )
                .into_response();
        }
        Ok(false) => {}
        Err(e) => return server_error(format!("db: {e}")).into_response(),
    }

    let hostname = format!("{username}.{}", state.root_domain);
    let tunnel_name = format!("things-api-{username}");

    // Create tunnel
    let tunnel = match state.cf.create_tunnel(&tunnel_name).await {
        Ok(t) => t,
        Err(CfError::Api(msg)) => {
            return server_error(format!("cloudflare: {msg}")).into_response();
        }
        Err(e) => return server_error(format!("cloudflare: {e}")).into_response(),
    };

    // Configure tunnel ingress: hostname -> http://localhost:3333 (the local things-api)
    if let Err(e) = state
        .cf
        .configure_tunnel(&tunnel.id, &hostname, "http://localhost:3333")
        .await
    {
        let _ = state.cf.delete_tunnel(&tunnel.id).await;
        return server_error(format!("cloudflare configure: {e}")).into_response();
    }

    // Create DNS CNAME
    let dns = match state.cf.create_dns_cname(&hostname, &tunnel.id).await {
        Ok(r) => r,
        Err(e) => {
            let _ = state.cf.delete_tunnel(&tunnel.id).await;
            return server_error(format!("cloudflare dns: {e}")).into_response();
        }
    };

    // Persist
    if let Err(e) = db::insert_account(
        &state.db,
        &username,
        payload.email.as_deref(),
        &tunnel.id,
        &tunnel.token,
        &dns.id,
    )
    .await
    {
        let _ = state.cf.delete_dns(&dns.id).await;
        let _ = state.cf.delete_tunnel(&tunnel.id).await;
        return server_error(format!("db insert: {e}")).into_response();
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!(SignupResponse {
            username,
            url: format!("https://{hostname}"),
            tunnel_token: tunnel.token,
        })),
    )
        .into_response()
}
