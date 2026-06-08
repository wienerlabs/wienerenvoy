//! macOS power controller backed by `pmset` and `shutdown`.
//!
//! Every call uses an argv array (never `sh -c`), so untrusted strings cannot be
//! interpolated into a shell. These commands require root; the daemon runs as a
//! LaunchDaemon so no `sudo` is needed in production. In dev (non-root) they
//! return `NotPermitted`, which is why tests use the mock instead.

use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use wienerenvoy_core::power::{PowerController, PowerError, PowerInfo, WakeSpec};

/// Power controller backed by macOS `pmset` and `shutdown`.
#[derive(Debug, Default, Clone, Copy)]
pub struct MacOsPower;

impl MacOsPower {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Run a command with an argv array and capture output, mapping failures to
/// typed `PowerError`s.
async fn run(program: &str, args: &[&str]) -> Result<String, PowerError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(PowerError::Spawn)?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let command = format!("{program} {}", args.join(" "));
    // Permission failures are common in dev (non-root); surface them clearly.
    if stderr.to_lowercase().contains("not permitted")
        || stderr.to_lowercase().contains("must be run as root")
        || stderr.to_lowercase().contains("permission denied")
    {
        return Err(PowerError::NotPermitted);
    }
    Err(PowerError::NonZeroExit {
        command,
        code: output.status.code(),
        stderr,
    })
}

/// Whole minutes of grace, for `shutdown +N`.
fn grace_minutes(grace: Duration) -> u64 {
    grace.as_secs() / 60
}

#[async_trait]
impl PowerController for MacOsPower {
    async fn sleep(&self) -> Result<(), PowerError> {
        run("pmset", &["sleepnow"]).await.map(|_| ())
    }

    async fn restart(&self, grace: Duration) -> Result<(), PowerError> {
        let mins = grace_minutes(grace);
        if mins == 0 {
            run("shutdown", &["-r", "now"]).await.map(|_| ())
        } else {
            run("shutdown", &["-r", &format!("+{mins}")])
                .await
                .map(|_| ())
        }
    }

    async fn shutdown(&self, grace: Duration) -> Result<(), PowerError> {
        let mins = grace_minutes(grace);
        if mins == 0 {
            run("shutdown", &["-h", "now"]).await.map(|_| ())
        } else {
            run("shutdown", &["-h", &format!("+{mins}")])
                .await
                .map(|_| ())
        }
    }

    async fn schedule_wake(&self, spec: WakeSpec) -> Result<(), PowerError> {
        match spec {
            WakeSpec::Relative { secs } => run("pmset", &["relative", "wake", &secs.to_string()])
                .await
                .map(|_| ()),
            WakeSpec::Schedule { at } => {
                // pmset wants local time formatted as MM/dd/yy HH:mm:ss.
                let formatted = at.format("%m/%d/%y %H:%M:%S").to_string();
                run("pmset", &["schedule", "wake", &formatted])
                    .await
                    .map(|_| ())
            }
        }
    }

    async fn power_state(&self) -> Result<PowerInfo, PowerError> {
        let out = run("pmset", &["-g", "ps"]).await?;
        let on_ac_power = out.contains("AC Power");
        Ok(PowerInfo {
            on_ac_power,
            pending_wake: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grace_minutes_floors_to_whole_minutes() {
        assert_eq!(grace_minutes(Duration::from_secs(0)), 0);
        assert_eq!(grace_minutes(Duration::from_secs(59)), 0);
        assert_eq!(grace_minutes(Duration::from_secs(60)), 1);
        assert_eq!(grace_minutes(Duration::from_secs(150)), 2);
    }
}
