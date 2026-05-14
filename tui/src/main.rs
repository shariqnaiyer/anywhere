mod api;
mod app;
mod config;
mod keys;
mod ui;

use std::io::{stdout, Write};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::api::ApiClient;
use crate::config::{Mode, TuiConfig};

#[derive(Parser, Debug)]
#[command(name = "things-tui", about = "Terminal UI for things-api", version)]
struct Cli {
    /// Re-run the first-run setup and overwrite tui.json.
    #[arg(long)]
    reconfigure: bool,

    /// Override endpoint for this run only: "local" or "remote".
    #[arg(long)]
    endpoint: Option<String>,

    /// Override the base URL entirely (e.g. "http://192.168.1.10:3333").
    #[arg(long)]
    url: Option<String>,

    /// Override the bearer token. Defaults to reading auth_token from the shared config dir.
    #[arg(long, env = "THINGS_AUTH_TOKEN")]
    token: Option<String>,

    /// Headless mode: hit every endpoint the TUI uses, print counts, then exit.
    /// Useful to verify the API + JSON decode without entering raw mode.
    #[arg(long, hide = true)]
    smoke_test: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve config: either load saved, or run the setup wizard.
    let cfg = if cli.reconfigure || config::read_config().is_none() {
        run_setup_wizard().await?
    } else {
        config::read_config().expect("checked above")
    };

    let probe = config::probe(&cfg).await;

    let base_url = if let Some(u) = cli.url.clone() {
        u
    } else if let Some(force) = cli.endpoint.as_deref() {
        match force {
            "local" => cfg.local_url.clone(),
            "remote" => probe
                .remote_url
                .clone()
                .ok_or_else(|| anyhow!("--endpoint remote requested but no remote URL configured"))?,
            other => return Err(anyhow!("--endpoint must be 'local' or 'remote', got '{}'", other)),
        }
    } else {
        config::resolved_url(&cfg, &probe).ok_or_else(|| anyhow!("no endpoint to talk to"))?
    };

    let token = cli
        .token
        .clone()
        .or_else(|| config::resolved_token(&cfg))
        .ok_or_else(|| {
            anyhow!(
                "no bearer token found.\n\
                 Start the things-api server once to generate one, or pass --token / set THINGS_AUTH_TOKEN."
            )
        })?;

    let client = ApiClient::new(base_url.clone(), token);

    // Make sure the chosen endpoint is at least reachable before clearing the screen,
    // otherwise the user stares at "Loading…" forever.
    if let Err(e) = ApiClient::ping(&base_url, Duration::from_secs(3)).await {
        eprintln!("⚠️  {} is unreachable: {}", base_url, e);
        eprintln!("    Continuing anyway — the TUI will retry on refresh (r).");
    }

    if cli.smoke_test {
        return run_smoke_test(&client).await;
    }

    let mut terminal = enter_tui()?;
    let (app, rx) = app::App::new(client, cfg, probe);
    let run_result = app::run(&mut terminal, app, rx).await;
    leave_tui(&mut terminal)?;
    run_result?;
    Ok(())
}

fn enter_tui() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode().context("enable raw mode")?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture).context("enter alt screen")?;
    Terminal::new(CrosstermBackend::new(out)).context("init terminal")
}

fn leave_tui(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();
    Ok(())
}

/// First-run / --reconfigure: a tiny *non-TUI* prompt before we take over the terminal.
/// Keeps the setup readable in scrollback and avoids ratatui flashing during init.
async fn run_setup_wizard() -> Result<TuiConfig> {
    let mut cfg = TuiConfig::default();

    // Probe both candidates so we can recommend the best default.
    cfg.remote_url = config::read_account_url();
    let probe = config::probe(&cfg).await;

    println!();
    println!("  things-tui — first-run setup");
    println!("  ────────────────────────────");
    println!();
    println!(
        "    Local server: {}",
        status_label(probe.local_reachable, &cfg.local_url)
    );
    match (&probe.remote_url, probe.remote_reachable) {
        (Some(url), true) => println!("    Remote:       reachable @ {}", url),
        (Some(url), false) => println!("    Remote:       unreachable @ {}", url),
        (None, _) => println!("    Remote:       (no account.json found — run `things-api signup`)"),
    }
    println!();
    println!("  Pick an endpoint mode:");
    println!("    [1] Local only (localhost:3333)");
    println!("    [2] Remote only (your <username>.<root-domain> URL)");
    println!("    [3] Auto: try local, fall back to remote   (recommended)");
    println!();
    print!("  Choice [3]: ");
    stdout().flush().ok();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let mode = match line.trim() {
        "1" => Mode::Local,
        "2" => Mode::Remote,
        _ => Mode::Auto,
    };
    cfg.mode = mode;

    if matches!(mode, Mode::Remote) && cfg.remote_url.is_none() {
        print!("  Paste the remote URL: ");
        stdout().flush().ok();
        let mut url_line = String::new();
        std::io::stdin().read_line(&mut url_line)?;
        let u = url_line.trim().to_string();
        if u.is_empty() {
            return Err(anyhow!("remote mode requires a URL"));
        }
        cfg.remote_url = Some(u);
    }

    if config::read_auth_token().is_none() {
        println!();
        println!("  No bearer token found at the shared config path.");
        print!("  Paste the bearer (or press Enter to use $THINGS_AUTH_TOKEN at runtime): ");
        stdout().flush().ok();
        let mut t = String::new();
        std::io::stdin().read_line(&mut t)?;
        let t = t.trim().to_string();
        if !t.is_empty() {
            cfg.explicit_token = Some(t);
        }
    }

    config::write_config(&cfg).context("write tui.json")?;
    println!();
    println!("  Saved → {}", config::tui_config_path().display());
    println!();
    Ok(cfg)
}

fn status_label(ok: bool, url: &str) -> String {
    if ok {
        format!("reachable @ {}", url)
    } else {
        format!("not reachable @ {}", url)
    }
}

async fn run_smoke_test(client: &ApiClient) -> Result<()> {
    println!("things-tui smoke test\n");

    let (areas, projects, tags) = tokio::join!(
        client.list_areas(),
        client.list_projects(),
        client.list_tags(),
    );
    let areas = areas.map_err(|e| anyhow!("list_areas: {e}"))?;
    let projects = projects.map_err(|e| anyhow!("list_projects: {e}"))?;
    let tags = tags.map_err(|e| anyhow!("list_tags: {e}"))?;
    println!("  sidebar: {} areas · {} projects · {} tags", areas.len(), projects.len(), tags.len());

    for list in ["inbox", "today", "upcoming", "anytime", "someday", "logbook", "trash"] {
        let tasks = client.list_tasks(Some(list)).await
            .map_err(|e| anyhow!("list_tasks({list}): {e}"))?;
        println!("  /tasks?list={list}: {} tasks", tasks.len());
    }

    if let Some(area) = areas.first() {
        let n = client.list_area_tasks(&area.id).await
            .map_err(|e| anyhow!("list_area_tasks: {e}"))?;
        println!("  /areas/{}/tasks ({}): {} tasks", area.id, area.title, n.len());
    }
    if let Some(p) = projects.first() {
        let n = client.list_project_tasks(&p.id).await
            .map_err(|e| anyhow!("list_project_tasks: {e}"))?;
        println!("  /projects/{}/tasks ({}): {} tasks", p.id, p.title, n.len());
    }
    if let Some(t) = tags.first() {
        let n = client.list_tag_tasks(&t.id).await
            .map_err(|e| anyhow!("list_tag_tasks: {e}"))?;
        println!("  /tags/{}/tasks (#{}): {} tasks", t.id, t.name, n.len());
    }

    println!("\n✓ all endpoints reachable, JSON decoded cleanly");
    Ok(())
}

