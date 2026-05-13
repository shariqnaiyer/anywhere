mod cloudflare;
mod db;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cloudflare::CloudflareClient;

#[derive(Parser)]
#[command(name = "control-plane", about = "things-api signup & tunnel provisioning")]
struct Args {
    /// Port to listen on
    #[arg(long, env = "PORT", default_value_t = 8080)]
    port: u16,

    /// SQLite database file path
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite://control-plane.db?mode=rwc")]
    database_url: String,

    /// Root domain to issue subdomains under (e.g. anywhere-api.io)
    #[arg(long, env = "ROOT_DOMAIN")]
    root_domain: String,

    /// Cloudflare API token with Account.Tunnel:Edit + Zone.DNS:Edit
    #[arg(long, env = "CF_API_TOKEN")]
    cf_api_token: String,

    /// Cloudflare account ID
    #[arg(long, env = "CF_ACCOUNT_ID")]
    cf_account_id: String,

    /// Cloudflare zone ID for ROOT_DOMAIN
    #[arg(long, env = "CF_ZONE_ID")]
    cf_zone_id: String,
}

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub cf: CloudflareClient,
    pub root_domain: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "control_plane=debug,tower_http=info".parse().unwrap()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let pool = db::init(&args.database_url).await?;

    let cf = CloudflareClient::new(
        args.cf_api_token.clone(),
        args.cf_account_id.clone(),
        args.cf_zone_id.clone(),
    );

    let state = Arc::new(AppState {
        db: pool,
        cf,
        root_domain: args.root_domain.clone(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/signup", axum::routing::post(routes::signup))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
