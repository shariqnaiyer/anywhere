mod applescript;
mod config;
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
use clap::{Parser, Subcommand};
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

/// Default control plane endpoint. Override with `THINGS_API_CONTROL_PLANE` env var.
const DEFAULT_CONTROL_PLANE: &str = "https://things-api-control-plane.fly.dev";
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
#[command(
    name = "things-api",
    about = "REST API server for Things 3",
    version,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Port to listen on for the local server (default: 3333, or PORT env var)
    #[arg(short, long, global = true)]
    port: Option<u16>,
}

#[derive(Subcommand)]
enum Command {
    /// (default) Start the API server and tunnel.
    Run {
        /// Force quick-tunnel mode even if a signed-up account exists (ephemeral URL).
        #[arg(long)]
        quick_tunnel: bool,
        /// Don't start any tunnel; serve on localhost only.
        #[arg(long)]
        local_only: bool,
    },
    /// Claim a username and provision a permanent URL via the control plane.
    Signup {
        /// Desired subdomain (3–32 chars, lowercase letters/digits/hyphen).
        username: String,
        /// Optional email for account recovery.
        #[arg(long)]
        email: Option<String>,
        /// Override the control plane URL.
        #[arg(long, env = "THINGS_API_CONTROL_PLANE")]
        control_plane: Option<String>,
    },
    /// Print current configuration: URL, bearer token, where files live.
    Status,
    /// Generate a new bearer token (invalidates the old one).
    RotateToken,
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

/// Resolve the bearer token. Order: THINGS_AUTH_TOKEN env, persisted file, or generate+persist.
fn resolve_token() -> String {
    if let Ok(t) = std::env::var("THINGS_AUTH_TOKEN") {
        if !t.is_empty() {
            return t;
        }
    }
    config::ensure_auth_token().unwrap_or_else(|e| {
        eprintln!("⚠️  failed to persist auth token ({}); falling back to ephemeral", e);
        config::generate_token()
    })
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

fn router(auth_token: String) -> Router {
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

    let token = Arc::new(auth_token);
    let api_routes = api_routes
        .layer(middleware::from_fn(auth_middleware))
        .layer(axum::Extension(token));

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
                .unwrap_or_else(|_| "things_api=info,tower_http=info".parse().unwrap()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let port: u16 = cli
        .port
        .or_else(|| std::env::var("PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(3333);

    match cli.command.unwrap_or(Command::Run {
        quick_tunnel: false,
        local_only: false,
    }) {
        Command::Run {
            quick_tunnel,
            local_only,
        } => run_server(port, quick_tunnel, local_only).await,
        Command::Signup {
            username,
            email,
            control_plane,
        } => {
            if let Err(e) = cmd_signup(&username, email.as_deref(), control_plane.as_deref()).await {
                eprintln!("signup failed: {e}");
                std::process::exit(1);
            }
        }
        Command::Status => cmd_status(port),
        Command::RotateToken => cmd_rotate_token(),
    }
}

async fn run_server(port: u16, force_quick: bool, local_only: bool) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    let token = resolve_token();
    let account = config::read_account();

    if local_only {
        print_banner(&format!("http://127.0.0.1:{port}"), &token, "local-only");
        info!("Listening on http://{}", addr);
        axum::serve(listener, router(token))
            .await
            .expect("Server failed");
        return;
    }

    // Decide tunnel mode: named (if signed up and not forced quick) or quick.
    let (tunnel, mode_label) = if !force_quick && account.is_some() {
        let acct = account.as_ref().unwrap();
        (
            start_named_tunnel(&acct.tunnel_token).await,
            format!("named ({})", acct.username),
        )
    } else {
        if force_quick && account.is_some() {
            eprintln!("(ignoring stored account; --quick-tunnel forces ephemeral mode)");
        } else if account.is_none() {
            eprintln!(
                "No signed-up account — using ephemeral quick tunnel.\n\
                 Run `things-api signup <username>` for a permanent URL.\n"
            );
        }
        (start_quick_tunnel(port).await, "quick (ephemeral)".to_string())
    };

    let url = match account.as_ref() {
        Some(a) if !force_quick => a.url.clone(),
        _ => tunnel.url.clone().unwrap_or_else(|| format!("http://127.0.0.1:{port}")),
    };

    print_banner(&url, &token, &mode_label);
    info!("Listening on http://{}", addr);

    tokio::select! {
        result = axum::serve(listener, router(token)) => {
            result.expect("Server failed");
        }
        _ = tunnel.wait() => {
            eprintln!("cloudflared exited unexpectedly");
        }
    }
}

fn print_banner(url: &str, token: &str, mode: &str) {
    println!();
    println!("  ╭──────────────────────────────────────────────────────────────────────╮");
    println!("  │  things-api {:<55} │", env!("CARGO_PKG_VERSION"));
    println!("  ├──────────────────────────────────────────────────────────────────────┤");
    println!("  │  Mode:   {:<59} │", mode);
    println!("  │  URL:    {:<59} │", url);
    println!("  │  Token:  {:<59} │", token);
    println!("  ╰──────────────────────────────────────────────────────────────────────╯");
    println!();
    println!("  Try it:");
    println!("    curl -H 'Authorization: Bearer {}' \\", token);
    println!("         {}/tasks", url);
    println!();
    println!("  Docs: {}/swagger-ui", url);
    println!();
}

async fn cmd_signup(
    username: &str,
    email: Option<&str>,
    control_plane_override: Option<&str>,
) -> Result<(), String> {
    let cp_url = control_plane_override
        .map(str::to_string)
        .or_else(|| std::env::var("THINGS_API_CONTROL_PLANE").ok())
        .unwrap_or_else(|| DEFAULT_CONTROL_PLANE.to_string());

    if config::read_account().is_some() {
        return Err(
            "An account is already configured. Delete the account file first if you want to switch:\n  rm \"$HOME/Library/Application Support/things-api/account.json\""
                .into(),
        );
    }

    let body = serde_json::json!({
        "username": username,
        "email": email,
    });

    eprintln!("Claiming `{}` via {} ...", username, cp_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let resp = client
        .post(format!("{}/signup", cp_url.trim_end_matches('/')))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.map_err(|e| format!("decode: {e}"))?;
    if !status.is_success() {
        let msg = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("(no error message)");
        return Err(format!("server returned {}: {}", status, msg));
    }

    let url = body
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or("response missing url")?
        .to_string();
    let tunnel_token = body
        .get("tunnel_token")
        .and_then(|v| v.as_str())
        .ok_or("response missing tunnel_token")?
        .to_string();
    let username_out = body
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or(username)
        .to_string();

    let account = config::Account {
        username: username_out.clone(),
        url: url.clone(),
        tunnel_token,
        control_plane_url: cp_url,
        created_at: chrono_now_iso(),
    };
    config::write_account(&account).map_err(|e| format!("write account: {e}"))?;

    // Make sure a bearer token exists for this user.
    let token = config::ensure_auth_token().map_err(|e| format!("write token: {e}"))?;

    println!();
    println!("  ✓ {} → {}", username_out, url);
    println!("  ✓ Saved to {}", config::account_path().display());
    println!();
    println!("  Start serving:  things-api");
    println!("  Your token:     {}", token);
    println!();
    Ok(())
}

fn cmd_status(port: u16) {
    println!();
    println!("  things-api {} status", env!("CARGO_PKG_VERSION"));
    println!();
    match config::read_account() {
        Some(a) => {
            println!("  Account:      {}", a.username);
            println!("  Public URL:   {}", a.url);
            println!("  Signed up:    {}", a.created_at);
            println!("  Control plane: {}", a.control_plane_url);
        }
        None => {
            println!("  Account:      (none — run `things-api signup <username>`)");
            println!("  Public URL:   ephemeral (changes every run)");
        }
    }
    match config::read_auth_token() {
        Some(t) => println!("  Token:        {}", t),
        None => println!("  Token:        (none yet — generated on next run)"),
    }
    println!("  Local URL:    http://127.0.0.1:{}", port);
    println!("  Config dir:   {}", config::config_dir().display());
    println!();
}

fn cmd_rotate_token() {
    let new_token = config::generate_token();
    match config::write_auth_token(&new_token) {
        Ok(()) => {
            println!();
            println!("  ✓ New token: {}", new_token);
            println!();
            println!("  Restart the server (if running) and update any clients.");
            println!();
        }
        Err(e) => {
            eprintln!("failed to write new token: {e}");
            std::process::exit(1);
        }
    }
}

fn chrono_now_iso() -> String {
    // Avoid pulling chrono just for this — use SystemTime + a quick format.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Just emit unix seconds suffixed; good enough for "when did this row arrive".
    format!("epoch:{secs}")
}

struct Tunnel {
    /// Some(url) for quick tunnels (parsed from stderr). None for named tunnels
    /// (the URL comes from the persisted account config instead).
    url: Option<String>,
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

async fn start_quick_tunnel(port: u16) -> Tunnel {
    let cloudflared = ensure_cloudflared().await;

    let mut child = tokio::process::Command::new(&cloudflared)
        .args(["tunnel", "--url", &format!("http://localhost:{}", port)])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start cloudflared");

    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let url = parse_tunnel_url(stderr).await;

    Tunnel {
        url: Some(url),
        child,
    }
}

/// Run cloudflared with a connector token from the control plane.
/// URL is determined by the DNS record the control plane created.
async fn start_named_tunnel(token: &str) -> Tunnel {
    let cloudflared = ensure_cloudflared().await;

    let mut child = tokio::process::Command::new(&cloudflared)
        .args(["tunnel", "run", "--token", token])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start cloudflared");

    // Drain stderr so cloudflared doesn't deadlock on a full pipe.
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[cloudflared] {}", line);
            }
        });
    }

    Tunnel { url: None, child }
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
