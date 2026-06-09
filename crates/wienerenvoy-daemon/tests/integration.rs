//! HTTP integration tests. They spin a real server bound to an ephemeral
//! loopback port and drive it with reqwest, using the recording mocks so no
//! real power action ever runs. Marked `#[ignore]` so the default `cargo test`
//! stays fast; CI runs them in a dedicated `--run-ignored ignored-only` pass.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::Utc;
use secrecy::SecretString;
use tokio::sync::{Mutex, broadcast};
use wienerenvoy_core::testutil::{MockKeepAwake, MockPower};
use wienerenvoy_core::{AuthStore, Config, ServerStateMachine};
use wienerenvoy_daemon::router::build_router;
use wienerenvoy_daemon::state::AppState;

const TOKEN: &str = "test-token-0123456789abcdef";

async fn spawn_server() -> SocketAddr {
    let auth = AuthStore::from_secret(SecretString::from(TOKEN.to_string()));
    let (events, _) = broadcast::channel(16);
    let state = AppState {
        config: Arc::new(Config::default()),
        auth: Arc::new(auth),
        state: Arc::new(Mutex::new(ServerStateMachine::new(Utc::now()))),
        power: Arc::new(MockPower::new()),
        keepawake: Arc::new(MockKeepAwake::new()),
        snapshot: Arc::new(Mutex::new(None)),
        events,
        docker: Arc::new(wienerenvoy_docker::DockerHandle::disconnected()),
        version: "test".to_string(),
    };
    let router = build_router(state, None);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    addr
}

#[tokio::test]
#[ignore = "integration: spins a server"]
async fn health_is_public() {
    let addr = spawn_server().await;
    let res = reqwest::get(format!("http://{addr}/health")).await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
#[ignore = "integration: spins a server"]
async fn state_requires_bearer() {
    let addr = spawn_server().await;
    let client = reqwest::Client::new();

    let unauth = client
        .get(format!("http://{addr}/api/v1/state"))
        .send()
        .await
        .unwrap();
    assert_eq!(unauth.status(), 401);

    let authed = client
        .get(format!("http://{addr}/api/v1/state"))
        .bearer_auth(TOKEN)
        .send()
        .await
        .unwrap();
    assert_eq!(authed.status(), 200);
}

#[tokio::test]
#[ignore = "integration: spins a server"]
async fn sleep_needs_no_confirmation() {
    let addr = spawn_server().await;
    let res = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/power/sleep"))
        .bearer_auth(TOKEN)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["action"], "sleep");
}

#[tokio::test]
#[ignore = "integration: spins a server"]
async fn shutdown_requires_confirmation() {
    let addr = spawn_server().await;
    let client = reqwest::Client::new();

    let unconfirmed = client
        .post(format!("http://{addr}/api/v1/power/shutdown"))
        .bearer_auth(TOKEN)
        .json(&serde_json::json!({ "confirm": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(unconfirmed.status(), 409);

    let confirmed = client
        .post(format!("http://{addr}/api/v1/power/shutdown"))
        .bearer_auth(TOKEN)
        .json(&serde_json::json!({ "confirm": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(confirmed.status(), 200);
    let body: serde_json::Value = confirmed.json().await.unwrap();
    assert_eq!(body["recoverableVia"][0], "wol_only");
}

#[tokio::test]
#[ignore = "integration: spins a server"]
async fn keep_awake_toggles() {
    let addr = spawn_server().await;
    let res = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/presence/keep-awake"))
        .bearer_auth(TOKEN)
        .json(&serde_json::json!({ "enabled": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}
