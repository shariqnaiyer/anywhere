//! Persisted TUI configuration. Lives next to the server's files so we share
//! one auth_token + account.json when running on the same Mac:
//!
//!   ~/Library/Application Support/things-api/
//!     auth_token        ← bearer (written by the server)
//!     account.json      ← signed-up account (written by signup flow)
//!     tui.json          ← THIS FILE — the TUI's saved endpoint choice
//!
//! `tui.json` lets the user pick local-only / remote-only / auto on first run
//! so subsequent launches skip the prompt.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::api::ApiClient;

const TUI_CONFIG_FILE: &str = "tui.json";
const TOKEN_FILE: &str = "auth_token";
const ACCOUNT_FILE: &str = "account.json";

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Only ever talk to localhost:3333.
    Local,
    /// Only ever talk to the user's `<username>.<root-domain>` URL.
    Remote,
    /// Try local first; fall back to remote if unreachable.
    Auto,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TuiConfig {
    pub mode: Mode,
    pub local_url: String,
    pub remote_url: Option<String>,
    /// Override the resolved bearer (set only if the user pasted one explicitly).
    /// Normally we read `auth_token` from disk on every launch.
    #[serde(default)]
    pub explicit_token: Option<String>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Auto,
            local_url: "http://127.0.0.1:3333".to_string(),
            remote_url: None,
            explicit_token: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Account {
    #[allow(dead_code)]
    pub username: String,
    pub url: String,
}

pub fn config_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("things-api");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn tui_config_path() -> PathBuf {
    config_dir().join(TUI_CONFIG_FILE)
}

pub fn read_config() -> Option<TuiConfig> {
    let raw = fs::read_to_string(tui_config_path()).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write_config(cfg: &TuiConfig) -> std::io::Result<()> {
    let raw = serde_json::to_string_pretty(cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(tui_config_path(), raw)
}

pub fn read_auth_token() -> Option<String> {
    fs::read_to_string(config_dir().join(TOKEN_FILE))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn read_account_url() -> Option<String> {
    let raw = fs::read_to_string(config_dir().join(ACCOUNT_FILE)).ok()?;
    let acct: Account = serde_json::from_str(&raw).ok()?;
    Some(acct.url)
}

/// What we know about both candidate endpoints at startup.
#[derive(Debug, Clone)]
pub struct Probe {
    pub local_reachable: bool,
    pub remote_url: Option<String>,
    pub remote_reachable: bool,
}

pub async fn probe(cfg: &TuiConfig) -> Probe {
    let local_reachable = ApiClient::ping(&cfg.local_url, Duration::from_secs(2))
        .await
        .is_ok();

    let remote_url = cfg
        .remote_url
        .clone()
        .or_else(read_account_url);

    let remote_reachable = if let Some(url) = remote_url.as_deref() {
        ApiClient::ping(url, Duration::from_secs(4))
            .await
            .is_ok()
    } else {
        false
    };

    Probe {
        local_reachable,
        remote_url,
        remote_reachable,
    }
}

/// Pick the URL to actually connect to given config + probe.
pub fn resolved_url(cfg: &TuiConfig, probe: &Probe) -> Option<String> {
    match cfg.mode {
        Mode::Local => Some(cfg.local_url.clone()),
        Mode::Remote => probe.remote_url.clone(),
        Mode::Auto => {
            if probe.local_reachable {
                Some(cfg.local_url.clone())
            } else if probe.remote_reachable {
                probe.remote_url.clone()
            } else {
                // Last resort: still try local — at least error messages are clearer.
                Some(cfg.local_url.clone())
            }
        }
    }
}

pub fn resolved_token(cfg: &TuiConfig) -> Option<String> {
    if let Some(t) = cfg.explicit_token.clone() {
        return Some(t);
    }
    read_auth_token()
}
