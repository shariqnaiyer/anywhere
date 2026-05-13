//! Thin wrapper over the Cloudflare API endpoints we need:
//!  - POST /accounts/:account_id/cfd_tunnel       (create tunnel)
//!  - DELETE /accounts/:account_id/cfd_tunnel/:id (cleanup on rollback)
//!  - POST /zones/:zone_id/dns_records           (create CNAME)

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum CfError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Cloudflare API error: {0}")]
    Api(String),
}

#[derive(Clone)]
pub struct CloudflareClient {
    http: Client,
    api_token: String,
    account_id: String,
    zone_id: String,
}

#[derive(Debug, Deserialize)]
struct CfEnvelope<T> {
    success: bool,
    errors: Vec<CfApiError>,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct CfApiError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct TunnelCreated {
    pub id: String,
    /// Connector token (`--token` for `cloudflared tunnel run`).
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct TunnelCreateResult {
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct DnsRecord {
    pub id: String,
}

impl CloudflareClient {
    pub fn new(api_token: String, account_id: String, zone_id: String) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .expect("reqwest client"),
            api_token,
            account_id,
            zone_id,
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.bearer_auth(&self.api_token)
    }

    async fn decode<T: for<'de> Deserialize<'de>>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T, CfError> {
        let status = resp.status();
        let text = resp.text().await?;
        let env: CfEnvelope<T> = serde_json::from_str(&text)
            .map_err(|e| CfError::Api(format!("decode failed (status {}): {} — body: {}", status, e, text)))?;
        if !env.success {
            let msgs = env
                .errors
                .iter()
                .map(|e| format!("[{}] {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(CfError::Api(format!("status {}: {}", status, msgs)));
        }
        env.result
            .ok_or_else(|| CfError::Api("missing result field".to_string()))
    }

    /// Create a Cloudflare-managed tunnel and return its UUID and connector token.
    pub async fn create_tunnel(&self, name: &str) -> Result<TunnelCreated, CfError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel",
            self.account_id
        );
        // tunnel_secret is unused for `cloudflared tunnel run --token` flows but required by the
        // endpoint; pass a random base64-encoded value.
        let mut secret = [0u8; 32];
        getrandom_fill(&mut secret);
        let tunnel_secret = base64_encode(&secret);

        let body = json!({
            "name": name,
            "tunnel_secret": tunnel_secret,
            "config_src": "cloudflare",
        });
        let resp = self
            .auth(self.http.post(&url).json(&body))
            .send()
            .await?;
        let created: TunnelCreateResult = self.decode(resp).await?;

        // Fetch the token via the dedicated endpoint.
        let token_url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel/{}/token",
            self.account_id, created.id
        );
        let resp = self.auth(self.http.get(&token_url)).send().await?;
        let token: String = self.decode(resp).await?;

        Ok(TunnelCreated {
            id: created.id,
            token,
        })
    }

    /// Configure the tunnel to route the given hostname to a local origin.
    /// Required so cloudflared knows where to send incoming traffic.
    pub async fn configure_tunnel(
        &self,
        tunnel_id: &str,
        hostname: &str,
        service: &str,
    ) -> Result<(), CfError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel/{}/configurations",
            self.account_id, tunnel_id
        );
        let body = json!({
            "config": {
                "ingress": [
                    { "hostname": hostname, "service": service },
                    { "service": "http_status:404" }
                ]
            }
        });
        let resp = self.auth(self.http.put(&url).json(&body)).send().await?;
        let _: serde_json::Value = self.decode(resp).await?;
        Ok(())
    }

    pub async fn create_dns_cname(
        &self,
        name: &str,
        tunnel_id: &str,
    ) -> Result<DnsRecord, CfError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        );
        let body = json!({
            "type": "CNAME",
            "name": name,
            "content": format!("{}.cfargotunnel.com", tunnel_id),
            "proxied": true,
            "ttl": 1,
        });
        let resp = self.auth(self.http.post(&url).json(&body)).send().await?;
        self.decode(resp).await
    }

    pub async fn delete_tunnel(&self, tunnel_id: &str) -> Result<(), CfError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel/{}",
            self.account_id, tunnel_id
        );
        let resp = self.auth(self.http.delete(&url)).send().await?;
        let _: serde_json::Value = self.decode(resp).await?;
        Ok(())
    }

    pub async fn delete_dns(&self, record_id: &str) -> Result<(), CfError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.zone_id, record_id
        );
        let resp = self.auth(self.http.delete(&url)).send().await?;
        let _: serde_json::Value = self.decode(resp).await?;
        Ok(())
    }
}

fn getrandom_fill(buf: &mut [u8]) {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Cheap seed; we only need this to pass CF's nonempty-secret check.
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xC0FFEE);
    for b in buf.iter_mut() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *b = (seed >> 33) as u8;
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const T: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8) | (bytes[i + 2] as u32);
        out.push(T[((n >> 18) & 0x3F) as usize] as char);
        out.push(T[((n >> 12) & 0x3F) as usize] as char);
        out.push(T[((n >> 6) & 0x3F) as usize] as char);
        out.push(T[(n & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let n = (bytes[i] as u32) << 16;
        out.push(T[((n >> 18) & 0x3F) as usize] as char);
        out.push(T[((n >> 12) & 0x3F) as usize] as char);
        out.push_str("==");
    } else if rem == 2 {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8);
        out.push(T[((n >> 18) & 0x3F) as usize] as char);
        out.push(T[((n >> 12) & 0x3F) as usize] as char);
        out.push(T[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}
