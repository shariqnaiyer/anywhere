mod applescript;
mod models;
mod routes;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use models::HealthResponse;
use routes::{
    projects::list_projects,
    tags::{list_areas, list_tags},
    tasks::{complete_task, create_task, delete_task, get_task, list_tasks, update_task},
};

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }),
    )
}

fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).patch(update_task).delete(delete_task),
        )
        .route("/tasks/{id}/complete", patch(complete_task))
        .route("/projects", get(list_projects))
        .route("/tags", get(list_tags))
        .route("/areas", get(list_areas))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "things_api=debug,tower_http=debug".parse().unwrap()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3333);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, router())
        .await
        .expect("Server failed");
}
