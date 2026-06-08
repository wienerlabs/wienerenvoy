//! WienerEnvoy daemon library: configuration, binding, telemetry sampling, and
//! the server run loop.
//!
//! The security posture lives here: the daemon binds only loopback and explicit
//! (tailnet) addresses, never `0.0.0.0`; a peer-guard rejects non-tailnet
//! callers; and a bearer-token middleware gates every privileged route.

pub mod auth;
pub mod bind;
pub mod error;
pub mod router;
pub mod routes;
pub mod state;
pub mod static_assets;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use chrono::Utc;
use secrecy::ExposeSecret;
use tokio::sync::{Mutex, Notify, broadcast};
use tokio::task::JoinSet;
use tokio::time::MissedTickBehavior;
use tracing_subscriber::EnvFilter;
use wienerenvoy_core::auth::read_token_file;
use wienerenvoy_core::{AuthStore, Config, KeepAwake, ServerStateMachine, SystemSnapshot, WsFrame};
use wienerenvoy_platform_macos::{CaffeinateKeeper, MacOsPower, MetricsSampler};

use crate::state::AppState;

/// Daemon crate version, surfaced over `/health`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default config path for a root LaunchDaemon. A root daemon's `$HOME` is
/// `/var/root`, so config and token live under system `/Library`, never `~`.
const DEFAULT_CONFIG_PATH: &str = "/Library/Application Support/WienerEnvoy/config.toml";

/// Run the daemon to completion (until SIGINT or SIGTERM).
pub async fn run() -> anyhow::Result<()> {
    init_tracing();

    let config_path =
        std::env::var("WIENERENVOY_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    let config = Config::load(Path::new(&config_path))
        .with_context(|| format!("loading config from {config_path}"))?;

    let auth = load_or_create_auth(&config);
    let web_dir = std::env::var("WIENERENVOY_WEB_DIR").ok().map(PathBuf::from);

    let (events, _) = broadcast::channel::<WsFrame>(256);
    let snapshot: Arc<Mutex<Option<SystemSnapshot>>> = Arc::new(Mutex::new(None));
    let keepawake: Arc<dyn KeepAwake> =
        Arc::new(CaffeinateKeeper::new(config.power.keep_awake_flags.clone()));

    let app_state = AppState {
        config: Arc::new(config.clone()),
        auth: Arc::new(auth),
        state: Arc::new(Mutex::new(ServerStateMachine::new(Utc::now()))),
        power: Arc::new(MacOsPower::new()),
        keepawake: keepawake.clone(),
        snapshot: snapshot.clone(),
        events: events.clone(),
        version: VERSION.to_string(),
    };

    spawn_sampler(config.metrics.sample_interval_ms, snapshot, events);

    let binds = bind::resolve_binds(
        config.server.http_port,
        config.server.allow_loopback,
        &config.server.extra_bind,
    );
    anyhow::ensure!(
        !binds.is_empty(),
        "no bind addresses resolved; refusing to start"
    );

    let router = router::build_router(app_state, web_dir);

    let notify = Arc::new(Notify::new());
    let mut servers = JoinSet::new();
    for addr in binds {
        let app = router.clone();
        let n = notify.clone();
        servers.spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .with_context(|| format!("binding {addr}"))?;
            tracing::info!(%addr, "wienerenvoy daemon listening");
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(async move {
                n.notified().await;
            })
            .await
            .with_context(|| format!("serving on {addr}"))?;
            Ok::<(), anyhow::Error>(())
        });
    }

    shutdown_signal().await;
    tracing::info!("shutdown signal received; draining");
    notify.notify_waiters();
    // Release the keep-awake assertion so the machine can idle-sleep again.
    let _ = keepawake.release().await;

    while let Some(joined) = servers.join_next().await {
        match joined {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::error!(error = %err, "server task error"),
            Err(err) => tracing::error!(error = %err, "server task panicked"),
        }
    }
    tracing::info!("wienerenvoy daemon stopped");
    Ok(())
}

/// Spawn the background telemetry sampler. It builds the sampler off the async
/// thread (the CPU warm-up blocks), then samples on an interval and publishes to
/// both the snapshot cache and the broadcast channel.
fn spawn_sampler(
    interval_ms: u64,
    snapshot: Arc<Mutex<Option<SystemSnapshot>>>,
    events: broadcast::Sender<WsFrame>,
) {
    tokio::spawn(async move {
        let mut sampler = match tokio::task::spawn_blocking(MetricsSampler::new).await {
            Ok(sampler) => sampler,
            Err(err) => {
                tracing::error!(error = %err, "failed to start metrics sampler");
                return;
            }
        };
        let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms.max(250)));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let snap = sampler.sample();
            *snapshot.lock().await = Some(snap.clone());
            let _ = events.send(WsFrame::Metrics(snap));
        }
    });
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}

/// Load the bearer token from the configured path, or generate and persist a
/// fresh one. Never logs the token value.
fn load_or_create_auth(config: &Config) -> AuthStore {
    let path = &config.auth.token_path;
    match read_token_file(path) {
        Ok(secret) => {
            tracing::info!(path = %path.display(), "loaded bearer token");
            AuthStore::from_secret(secret)
        }
        Err(load_err) => {
            tracing::info!(reason = %load_err, "no usable token found; generating a new one");
            let secret = AuthStore::generate_token();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match persist_token(path, secret.expose_secret()) {
                Ok(()) => tracing::info!(path = %path.display(), "persisted new bearer token"),
                Err(err) => tracing::warn!(
                    error = %err,
                    "could not persist token; using an ephemeral token for this run. \
                     Set WIENERENVOY_AUTH__TOKEN_PATH to a writable path to keep it."
                ),
            }
            AuthStore::from_secret(secret)
        }
    }
}

/// Write the token atomically with restrictive permissions (0600).
fn persist_token(path: &Path, token: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Resolve when both SIGINT (Ctrl-C) and SIGTERM (launchd `bootout`) fire.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(err) => {
                tracing::warn!(error = %err, "could not install SIGTERM handler");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
