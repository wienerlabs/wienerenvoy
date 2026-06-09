//! `wenvoy` subcommand implementations.
//!
//! Read-only commands (status, metrics, info) and control commands (keep-awake,
//! server on/off, power) all go through the authenticated daemon client.

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

/// Print a one-shot telemetry snapshot.
pub async fn metrics(config: &Config) -> Result<()> {
    let client = Client::from_config(config);
    let m = client.metrics().await?;
    ui::section("System");
    ui::kv(
        "cpu",
        format!("{:.0}%  ({} cores)", m.cpu.usage_pct, m.cpu.cores),
    );
    ui::kv(
        "memory",
        format!(
            "{} / {}",
            fmt_bytes(m.memory.used_bytes),
            fmt_bytes(m.memory.total_bytes)
        ),
    );
    ui::kv(
        "network",
        format!(
            "rx {}/s  tx {}/s",
            fmt_bytes(m.network.rx_bytes_per_sec),
            fmt_bytes(m.network.tx_bytes_per_sec)
        ),
    );
    ui::kv("uptime", fmt_uptime(m.uptime_secs));
    for disk in &m.disks {
        ui::kv(
            &disk.mount,
            format!(
                "{} / {}",
                fmt_bytes(disk.used_bytes),
                fmt_bytes(disk.total_bytes)
            ),
        );
    }
    Ok(())
}

/// Print one-shot machine info.
pub async fn info(config: &Config) -> Result<()> {
    let client = Client::from_config(config);
    let i = client.info().await?;
    ui::section("Machine");
    ui::kv("hostname", i.hostname);
    ui::kv("os", i.os_version);
    ui::kv("cpu", i.cpu_model);
    ui::kv("memory", fmt_bytes(i.mem_total));
    ui::kv(
        "tailnet ip",
        i.tailnet_ip.unwrap_or_else(|| "unknown".to_string()),
    );
    ui::kv("daemon", format!("v{}", i.daemon_version));
    Ok(())
}

/// Toggle the keep-awake assertion.
pub async fn keep_awake(config: &Config, enabled: bool) -> Result<()> {
    let client = Client::from_config(config);
    client.keep_awake(enabled).await?;
    ui::success(format!("keep-awake {}", if enabled { "on" } else { "off" }));
    Ok(())
}

/// Turn the server function on or off (presence; daemon stays reachable).
pub async fn server(config: &Config, on: bool) -> Result<()> {
    let client = Client::from_config(config);
    let state = client.server(on).await?;
    ui::success(format!(
        "server {} (presence: {})",
        if on { "on" } else { "off" },
        state.presence
    ));
    Ok(())
}

/// Issue a machine power action. `confirm` gates restart and shutdown.
pub async fn power(config: &Config, action: &str, confirm: bool) -> Result<()> {
    let client = Client::from_config(config);
    match client.power(action, confirm).await {
        Ok(accepted) => {
            ui::success(format!("{action} command sent"));
            if !accepted.recoverable_via.is_empty() {
                ui::kv("recover via", format!("{:?}", accepted.recoverable_via));
            }
            if let Some(warning) = accepted.warning {
                ui::warning(warning);
            }
            Ok(())
        }
        Err(err) => {
            ui::error(format!("{action}: {err}"));
            if action != "sleep" && !confirm {
                ui::hint(format!("add --yes to confirm {action}"));
            }
            anyhow::bail!("power action failed")
        }
    }
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

/// Token rotation is planned for a later milestone (atomic rewrite + reload).
pub fn token_rotate() -> Result<()> {
    ui::warning("token rotation is planned for a later milestone");
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

fn container_status(state: &str) -> ui::Status {
    match state {
        "running" => ui::Status::Ok,
        "paused" | "created" | "restarting" => ui::Status::Warn,
        _ => ui::Status::Fail,
    }
}

/// List Docker stacks and containers.
pub async fn services(config: &Config) -> Result<()> {
    let client = Client::from_config(config);
    let view = client.services().await?;
    ui::banner_heading("WienerEnvoy");
    if !view.docker.available {
        ui::warning("Docker is not available on the server");
        return Ok(());
    }
    if view.stacks.is_empty() && view.standalone.is_empty() {
        ui::section("Services");
        ui::hint("no containers found");
        return Ok(());
    }
    for stack in &view.stacks {
        ui::section(&format!(
            "{} ({}/{} up)",
            stack.name, stack.running, stack.total
        ));
        for container in &stack.containers {
            ui::line(
                container_status(&container.state),
                &container.name,
                &container.status,
            );
        }
    }
    if !view.standalone.is_empty() {
        ui::section("standalone");
        for container in &view.standalone {
            ui::line(
                container_status(&container.state),
                &container.name,
                &container.status,
            );
        }
    }
    Ok(())
}

/// Start, stop, or restart a stack or container.
pub async fn service_control(config: &Config, kind: &str, id: &str, action: &str) -> Result<()> {
    let client = Client::from_config(config);
    let result = client.service_control(kind, id, action).await?;
    ui::success(format!(
        "{action} {kind} '{id}' ({} container(s) affected)",
        result.affected
    ));
    Ok(())
}

/// Tail a container's logs.
pub async fn service_logs(config: &Config, id: &str, tail: usize) -> Result<()> {
    let client = Client::from_config(config);
    let logs = client.service_logs(id, tail).await?;
    for line in logs.lines {
        println!("{line}");
    }
    Ok(())
}

fn fmt_bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if n == 0 {
        return "0 B".to_string();
    }
    let mut value = n as f64;
    let mut exp = 0;
    while value >= 1024.0 && exp < UNITS.len() - 1 {
        value /= 1024.0;
        exp += 1;
    }
    if exp == 0 {
        format!("{n} B")
    } else {
        format!("{value:.1} {}", UNITS[exp])
    }
}

fn fmt_uptime(secs: u64) -> String {
    if secs == 0 {
        return "--".to_string();
    }
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}
