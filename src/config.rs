//! Persisted client state lives at:
//!   macOS:  ~/Library/Application Support/things-api/
//!   Linux:  ~/.config/things-api/  (xdg fallback)
//!
//! Two files:
//!   - auth_token        single line, the bearer the user plugs into clients
//!   - account.json      { username, url, tunnel_token, control_plane_url, created_at }

use std::fs;
use std::path::PathBuf;

use rand::Rng;
use serde::{Deserialize, Serialize};

const TOKEN_FILE: &str = "auth_token";
const ACCOUNT_FILE: &str = "account.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub username: String,
    pub url: String,
    pub tunnel_token: String,
    pub control_plane_url: String,
    pub created_at: String,
}

pub fn config_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("things-api");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn generate_token() -> String {
    let bytes: [u8; 24] = rand::rng().random();
    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    format!("thingsapi_{}", hex)
}

pub fn read_auth_token() -> Option<String> {
    let path = config_dir().join(TOKEN_FILE);
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

pub fn write_auth_token(token: &str) -> std::io::Result<()> {
    let path = config_dir().join(TOKEN_FILE);
    fs::write(&path, token)?;
    // Best-effort chmod 600 on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Read the bearer token, generating and persisting one if it doesn't exist yet.
pub fn ensure_auth_token() -> std::io::Result<String> {
    if let Some(t) = read_auth_token() {
        return Ok(t);
    }
    let t = generate_token();
    write_auth_token(&t)?;
    Ok(t)
}

pub fn read_account() -> Option<Account> {
    let path = config_dir().join(ACCOUNT_FILE);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write_account(account: &Account) -> std::io::Result<()> {
    let path = config_dir().join(ACCOUNT_FILE);
    let raw = serde_json::to_string_pretty(account)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&path, raw)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn account_path() -> PathBuf {
    config_dir().join(ACCOUNT_FILE)
}
