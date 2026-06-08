//! `wenvoy` subcommand implementations. M0 ships `status`, `token show`, and
//! install/uninstall guidance; the power and token-rotate commands land in M1.
//!
//! Several commands return `Result` even though their M0 bodies cannot fail:
//! the dispatcher handles every command uniformly, and these gain real error
//! paths in M1 (token rotation IO, install-script invocation).
#![allow(clippy::unnecessary_wraps)]

use anyhow::Result;
use secrecy::ExposeSecret;
use wienerenvoy_core::Config;
use wienerenvoy_core::auth::read_token_file;

use crate::client::Client;
use crate::ui;

/// Show daemon liveness and server-level state.
pub async fn status(config: &Config) -> Result<()> {
    ui::banner_heading("WienerEnvoy");
    let client = Client::from_config(config);

    ui::section("Daemon");
    match client.health().await {
        Ok(health) => ui::line(
            ui::Status::Ok,
            "daemon",
            &format!("{}  v{}", health.status, health.version),
        ),
        Err(err) => {
            ui::line(ui::Status::Fail, "daemon", "offline");
            ui::hint(format!("could not reach daemon: {err}"));
            ui::hint("start it with the wienerenvoy-daemon binary, or `sudo ./install/install.sh`");
            return Ok(());
        }
    }

    match client.state().await {
        Ok(state) => {
            ui::section("Server");
            ui::kv("level", format!("{:?}", state.level));
            ui::kv("presence", state.presence);
            ui::kv("since", state.since.to_rfc3339());
        }
        Err(err) => ui::hint(format!("server state unavailable: {err}")),
    }

    Ok(())
}

/// Print the bearer token (read directly from the token file).
pub fn token_show(config: &Config) -> Result<()> {
    match read_token_file(&config.auth.token_path) {
        Ok(secret) => {
            println!("{}", secret.expose_secret());
            Ok(())
        }
        Err(err) => {
            ui::error(format!(
                "could not read token at {}: {err}",
                config.auth.token_path.display()
            ));
            ui::hint("the daemon generates the token on first run; ensure it has started");
            ui::hint("in dev, set WIENERENVOY_AUTH__TOKEN_PATH to a path you can read");
            anyhow::bail!("token unavailable")
        }
    }
}

/// Token rotation lands in M1 (atomic rewrite plus daemon reload).
pub fn token_rotate() -> Result<()> {
    ui::warning("token rotation lands in M1");
    Ok(())
}

/// Print install guidance. The privileged work lives in the script, not the CLI.
pub fn install() -> Result<()> {
    ui::section("Install");
    ui::kv("run", "sudo ./install/install.sh");
    ui::hint("installs wienerenvoy-daemon as a root LaunchDaemon and prints the dashboard token");
    Ok(())
}

/// Print uninstall guidance.
pub fn uninstall() -> Result<()> {
    ui::section("Uninstall");
    ui::kv("run", "sudo ./install/uninstall.sh");
    ui::hint("removes the LaunchDaemon and binaries; prompts before deleting config and token");
    Ok(())
}
