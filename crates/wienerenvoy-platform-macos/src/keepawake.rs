//! Keep-awake assertion backed by `caffeinate`.
//!
//! Holds a long-lived `caffeinate` child process. The normal lifecycle is
//! covered by `kill_on_drop(true)` plus an explicit `release()` on graceful
//! shutdown. A hard SIGKILL of the daemon skips both, re-parenting the child to
//! launchd. To cover that, `engage()` records the child PID in a pid-file and
//! `reconcile_orphan()` (called once at startup) kills any surviving caffeinate
//! from a previous run. PID reuse is guarded by re-checking the command name.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tokio::process::{Child, Command};
use wienerenvoy_core::power::{KeepAwake, PowerError};

/// Keep-awake backed by macOS `caffeinate`.
pub struct CaffeinateKeeper {
    flags: String,
    child: Mutex<Option<Child>>,
    pid_file: PathBuf,
}

impl CaffeinateKeeper {
    /// Create a keeper with the given `caffeinate` flags (e.g. `-s`) and a path
    /// for the pid-file used by orphan reconciliation.
    #[must_use]
    pub fn new(flags: impl Into<String>, pid_file: impl Into<PathBuf>) -> Self {
        Self {
            flags: flags.into(),
            child: Mutex::new(None),
            pid_file: pid_file.into(),
        }
    }

    /// Kill any caffeinate orphaned by a previous daemon that was hard-killed.
    /// Safe to call when no pid-file exists. Verified by liveness plus a
    /// command-name re-check so a reused PID is never killed by mistake.
    pub fn reconcile_orphan(&self) {
        let Ok(content) = fs::read_to_string(&self.pid_file) else {
            return;
        };
        if let Ok(pid) = content.trim().parse::<i32>()
            && is_caffeinate(pid)
        {
            let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
            tracing::warn!(pid, "killed an orphaned caffeinate from a previous run");
        }
        let _ = fs::remove_file(&self.pid_file);
    }

    fn write_pid(&self, pid: u32) {
        if let Err(err) = fs::write(&self.pid_file, pid.to_string()) {
            tracing::warn!(error = %err, "could not write caffeinate pid file");
        }
    }

    fn clear_pid(&self) {
        let _ = fs::remove_file(&self.pid_file);
    }
}

impl Default for CaffeinateKeeper {
    fn default() -> Self {
        Self::new(
            "-s",
            std::env::temp_dir().join("wienerenvoy-caffeinate.pid"),
        )
    }
}

/// Whether `pid` is alive and its command is `caffeinate`. The command-name
/// re-check guards against PID reuse between daemon runs.
fn is_caffeinate(pid: i32) -> bool {
    if kill(Pid::from_raw(pid), None).is_err() {
        return false;
    }
    match std::process::Command::new("ps")
        .args(["-o", "comm=", "-p", &pid.to_string()])
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .trim()
            .ends_with("caffeinate"),
        Err(_) => false,
    }
}

#[async_trait]
impl KeepAwake for CaffeinateKeeper {
    async fn engage(&self, reason: &str) -> Result<(), PowerError> {
        let mut guard = self.child.lock().unwrap();
        if guard.is_some() {
            return Ok(());
        }
        let child = Command::new("caffeinate")
            .arg(&self.flags)
            .kill_on_drop(true)
            .spawn()
            .map_err(PowerError::Spawn)?;
        let pid = child.id();
        tracing::info!(reason, pid, "caffeinate engaged");
        if let Some(pid) = pid {
            self.write_pid(pid);
        }
        *guard = Some(child);
        Ok(())
    }

    async fn release(&self) -> Result<(), PowerError> {
        // Take the child out under the lock, then await the kill without holding
        // the (non-Send) guard across the await point.
        let child = self.child.lock().unwrap().take();
        if let Some(mut child) = child {
            let _ = child.start_kill();
            let _ = child.wait().await;
            self.clear_pid();
            tracing::info!("caffeinate released");
        }
        Ok(())
    }

    fn is_engaged(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }

    fn holder_pid(&self) -> Option<u32> {
        self.child.lock().unwrap().as_ref().and_then(Child::id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_is_a_noop_without_pid_file() {
        let keeper = CaffeinateKeeper::new(
            "-s",
            std::env::temp_dir().join("wienerenvoy-test-absent.pid"),
        );
        // Must not panic when the pid-file does not exist.
        keeper.reconcile_orphan();
    }

    #[test]
    fn bogus_pid_is_not_caffeinate() {
        // An almost-certainly-dead PID is not alive, so not our caffeinate.
        assert!(!is_caffeinate(2_000_000_000));
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    #[ignore = "spawns a real caffeinate process; macOS only"]
    async fn engage_release_real_caffeinate() {
        let keeper = CaffeinateKeeper::new(
            "-s",
            std::env::temp_dir().join("wienerenvoy-test-engage.pid"),
        );
        assert!(!keeper.is_engaged());
        keeper.engage("test").await.unwrap();
        assert!(keeper.is_engaged());
        assert!(keeper.holder_pid().is_some());
        keeper.release().await.unwrap();
        assert!(!keeper.is_engaged());
    }
}
