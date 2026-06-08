//! Minimal HTTP client to the daemon. Reads the daemon URL from
//! `WIENERENVOY_DAEMON_URL` (or defaults to loopback) and the bearer token from
//! the configured token file.

use anyhow::{Context, Result};
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use wienerenvoy_core::auth::read_token_file;
use wienerenvoy_core::{Config, ServerState};

pub struct Client {
    base: String,
    token: Option<String>,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResp {
    pub status: String,
    pub version: String,
}

impl Client {
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let base = std::env::var("WIENERENVOY_DAEMON_URL")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{}", config.server.http_port));
        let token = read_token_file(&config.auth.token_path)
            .ok()
            .map(|s| s.expose_secret().to_string());
        Self {
            base,
            token,
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let mut req = self.http.get(format!("{}{path}", self.base));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let resp = req.send().await.with_context(|| format!("GET {path}"))?;
        let status = resp.status();
        anyhow::ensure!(status.is_success(), "GET {path} returned HTTP {status}");
        resp.json::<T>()
            .await
            .with_context(|| format!("decoding {path}"))
    }

    pub async fn health(&self) -> Result<HealthResp> {
        self.get("/health").await
    }

    pub async fn state(&self) -> Result<ServerState> {
        self.get("/api/v1/state").await
    }
}
