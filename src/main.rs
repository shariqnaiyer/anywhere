mod applescript;
mod models;
mod routes;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use clap::Parser;
use rand::Rng;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use models::HealthResponse;
use routes::{
    areas::{
        count_areas, create_area, delete_area, get_area, list_area_tasks, list_areas, show_area,
        update_area,
    },
    contacts::{
        count_contacts, create_contact, delete_contact, list_contact_tasks, list_contacts,
    },
    lists::{list_lists, show_list},
    projects::{
        count_projects, create_project, delete_project, edit_project, get_project,
        list_project_tasks, list_projects, show_project, update_project,
    },
    system::{
        app_info, close_window, list_windows, log_completed_now, parse_quicksilver, quit_app,
        show_quick_entry, update_window,
    },
    tags::{
        count_tags, create_tag, delete_tag, get_tag, list_tag_children, list_tag_tasks, list_tags,
        update_tag,
    },
    tasks::{
        cancel_task, complete_task, count_tasks, create_task, delete_task, edit_task, get_task,
        list_selected_tasks, list_tasks, show_task, update_task,
    },
    trash::empty_trash,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "things-api",
        description = "REST API for Things 3 (macOS) via AppleScript. Not affiliated with Cultured Code.",
    ),
    paths(
        health,
        routes::tasks::list_tasks,
        routes::tasks::list_selected_tasks,
        routes::tasks::count_tasks,
        routes::tasks::get_task,
        routes::tasks::create_task,
        routes::tasks::update_task,
        routes::tasks::complete_task,
        routes::tasks::cancel_task,
        routes::tasks::show_task,
        routes::tasks::edit_task,
        routes::tasks::delete_task,
        routes::projects::list_projects,
        routes::projects::count_projects,
        routes::projects::get_project,
        routes::projects::list_project_tasks,
        routes::projects::create_project,
        routes::projects::update_project,
        routes::projects::delete_project,
        routes::projects::show_project,
        routes::projects::edit_project,
        routes::areas::list_areas,
        routes::areas::count_areas,
        routes::areas::get_area,
        routes::areas::list_area_tasks,
        routes::areas::create_area,
        routes::areas::update_area,
        routes::areas::delete_area,
        routes::areas::show_area,
        routes::tags::list_tags,
        routes::tags::count_tags,
        routes::tags::get_tag,
        routes::tags::list_tag_tasks,
        routes::tags::list_tag_children,
        routes::tags::create_tag,
        routes::tags::update_tag,
        routes::tags::delete_tag,
        routes::contacts::list_contacts,
        routes::contacts::count_contacts,
        routes::contacts::list_contact_tasks,
        routes::contacts::create_contact,
        routes::contacts::delete_contact,
        routes::lists::list_lists,
        routes::lists::show_list,
        routes::system::app_info,
        routes::system::list_windows,
        routes::system::update_window,
        routes::system::close_window,
        routes::system::quit_app,
        routes::system::log_completed_now,
        routes::system::show_quick_entry,
        routes::system::parse_quicksilver,
        routes::trash::empty_trash,
    ),
    components(schemas(
        models::Task,
        models::ChecklistItem,
        models::Project,
        models::Tag,
        models::Area,
        models::Contact,
        models::ListInfo,
        models::CreateTask,
        models::UpdateTask,
        models::CreateProject,
        models::UpdateProject,
        models::CreateArea,
        models::UpdateArea,
        models::CreateTag,
        models::UpdateTag,
        models::CreateContact,
        models::QuickEntry,
        models::ParseInput,
        models::AppInfo,
        models::WindowInfo,
        models::UpdateWindow,
        models::QuitRequest,
        models::CountResponse,
        models::HealthResponse,
        models::ErrorResponse,
    )),
    tags(
        (name = "tasks", description = "Tasks in Things 3"),
        (name = "projects", description = "Projects in Things 3"),
        (name = "tags", description = "Tags in Things 3"),
        (name = "areas", description = "Areas in Things 3"),
        (name = "contacts", description = "Contacts in Things 3"),
        (name = "lists", description = "The seven special lists (Inbox, Today, …)"),
        (name = "system", description = "System / UI commands"),
        (name = "trash", description = "Trash management in Things 3"),
    ),
    modifiers(&BearerAuthAddon),
)]
struct ApiDoc;

struct BearerAuthAddon;

impl Modify for BearerAuthAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

#[derive(Parser)]
#[command(name = "things-api", about = "REST API server for Things 3")]
struct Args {
    /// Expose the server via a Cloudflare HTTPS tunnel (no account needed)
    #[arg(long)]
    tunnel: bool,

    /// Port to listen on (default: 3333, or PORT env var)
    #[arg(short, long)]
    port: Option<u16>,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, description = "Server status", body = HealthResponse)),
)]
async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }),
    )
}

fn generate_token() -> String {
    let bytes: [u8; 24] = rand::rng().random();
    let encoded = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    format!("thingsapi_{}", encoded)
}

fn resolve_token() -> String {
    std::env::var("THINGS_AUTH_TOKEN").unwrap_or_else(|_| generate_token())
}

async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let token = request
        .extensions()
        .get::<Arc<String>>()
        .expect("auth token not in extensions");

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header == format!("Bearer {}", token) => Ok(next.run(request).await),
        Some(_) => Err(StatusCode::UNAUTHORIZED),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

fn router(auth_token: Option<String>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        // Tasks
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/selected", get(list_selected_tasks))
        .route("/tasks/count", get(count_tasks))
        .route("/tasks/parse", post(parse_quicksilver))
        .route(
            "/tasks/{id}",
            get(get_task).patch(update_task).delete(delete_task),
        )
        .route("/tasks/{id}/complete", patch(complete_task))
        .route("/tasks/{id}/cancel", patch(cancel_task))
        .route("/tasks/{id}/show", post(show_task))
        .route("/tasks/{id}/edit", post(edit_task))
        // Projects
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/count", get(count_projects))
        .route(
            "/projects/{id}",
            get(get_project).patch(update_project).delete(delete_project),
        )
        .route("/projects/{id}/tasks", get(list_project_tasks))
        .route("/projects/{id}/show", post(show_project))
        .route("/projects/{id}/edit", post(edit_project))
        // Areas
        .route("/areas", get(list_areas).post(create_area))
        .route("/areas/count", get(count_areas))
        .route(
            "/areas/{id}",
            get(get_area).patch(update_area).delete(delete_area),
        )
        .route("/areas/{id}/tasks", get(list_area_tasks))
        .route("/areas/{id}/show", post(show_area))
        // Tags
        .route("/tags", get(list_tags).post(create_tag))
        .route("/tags/count", get(count_tags))
        .route(
            "/tags/{id}",
            get(get_tag).patch(update_tag).delete(delete_tag),
        )
        .route("/tags/{id}/tasks", get(list_tag_tasks))
        .route("/tags/{id}/children", get(list_tag_children))
        // Contacts
        .route("/contacts", get(list_contacts).post(create_contact))
        .route("/contacts/count", get(count_contacts))
        .route("/contacts/{id}", delete(delete_contact))
        .route("/contacts/{id}/tasks", get(list_contact_tasks))
        // Lists
        .route("/lists", get(list_lists))
        .route("/lists/{name}/show", post(show_list))
        // System / UI
        .route("/info", get(app_info))
        .route("/windows", get(list_windows))
        .route(
            "/windows/{id}",
            patch(update_window).delete(close_window),
        )
        .route("/system/log-completed", post(log_completed_now))
        .route("/system/quick-entry", post(show_quick_entry))
        .route("/system/quit", post(quit_app))
        // Trash
        .route("/trash", delete(empty_trash));

    let api_routes = if let Some(token) = auth_token {
        let token = Arc::new(token);
        api_routes
            .layer(middleware::from_fn(auth_middleware))
            .layer(axum::Extension(token))
    } else {
        api_routes
    };

    Router::new()
        .route("/health", get(health))
        .merge(api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors)
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

    let args = Args::parse();

    let port: u16 = args
        .port
        .or_else(|| std::env::var("PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(3333);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    if args.tunnel {
        let token = resolve_token();
        let tunnel = start_tunnel(port).await;

        println!();
        println!("  HTTPS tunnel: {}", tunnel.url);
        println!("  Auth token:   {}", token);
        println!();
        println!("  Use this header on other devices:");
        println!("  Authorization: Bearer {}", token);
        println!();

        tokio::select! {
            result = axum::serve(listener, router(Some(token))) => {
                result.expect("Server failed");
            }
            _ = tunnel.wait() => {
                eprintln!("cloudflared process exited unexpectedly");
            }
        }
    } else {
        axum::serve(listener, router(None))
            .await
            .expect("Server failed");
    }
}

struct Tunnel {
    url: String,
    child: tokio::process::Child,
}

impl Tunnel {
    async fn wait(mut self) {
        let _ = self.child.wait().await;
    }
}

fn cloudflared_path() -> std::path::PathBuf {
    dirs_for_download().join("cloudflared")
}

fn dirs_for_download() -> std::path::PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("things-api");
    std::fs::create_dir_all(&dir).ok();
    dir
}

async fn ensure_cloudflared() -> std::path::PathBuf {
    // Check if cloudflared is on PATH first
    if let Ok(output) = tokio::process::Command::new("which")
        .arg("cloudflared")
        .output()
        .await
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            info!("Found cloudflared at {}", path);
            return std::path::PathBuf::from(path);
        }
    }

    let path = cloudflared_path();
    if path.exists() {
        info!("Using cached cloudflared at {}", path.display());
        return path;
    }

    println!("Downloading cloudflared...");

    let dir = dirs_for_download();

    #[cfg(target_os = "macos")]
    {
        let tgz_path = dir.join("cloudflared.tgz");
        let url = download_url();
        run_curl(&tgz_path, url).await;

        let output = tokio::process::Command::new("tar")
            .args(["-xzf"])
            .arg(&tgz_path)
            .arg("-C")
            .arg(&dir)
            .output()
            .await
            .expect("Failed to extract cloudflared");

        if !output.status.success() {
            panic!(
                "Failed to extract cloudflared: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        std::fs::remove_file(&tgz_path).ok();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let url = download_url();
        run_curl(&path, url).await;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("Failed to make cloudflared executable");
    }

    info!("Downloaded cloudflared to {}", path.display());
    path
}

async fn run_curl(dest: &std::path::Path, url: &str) {
    let output = tokio::process::Command::new("curl")
        .args(["-L", "-o"])
        .arg(dest)
        .arg(url)
        .output()
        .await
        .expect("Failed to run curl — is curl installed?");

    if !output.status.success() {
        panic!(
            "Failed to download cloudflared: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn download_url() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-arm64.tgz";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64";
}

async fn start_tunnel(port: u16) -> Tunnel {
    let cloudflared = ensure_cloudflared().await;

    let mut child = tokio::process::Command::new(&cloudflared)
        .args(["tunnel", "--url", &format!("http://localhost:{}", port)])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start cloudflared");

    // cloudflared prints the URL to stderr
    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let url = parse_tunnel_url(stderr).await;

    // Re-pipe stderr to our stderr for logging
    Tunnel { url, child }
}

async fn parse_tunnel_url(stderr: tokio::process::ChildStderr) -> String {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);

    while let Ok(Some(line)) = lines.next_line().await {
        eprintln!("[cloudflared] {}", line);
        if let Some(url_start) = line.find("https://") {
            let url = &line[url_start..];
            let url = url.split_whitespace().next().unwrap_or(url);
            if url.contains(".trycloudflare.com") {
                let found = url.to_string();
                // Keep draining stderr in the background so cloudflared doesn't get a broken pipe
                tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        eprintln!("[cloudflared] {}", line);
                    }
                });
                return found;
            }
        }
        if start.elapsed() > timeout {
            panic!("Timed out waiting for cloudflared tunnel URL");
        }
    }

    panic!("cloudflared exited without providing a tunnel URL");
}
