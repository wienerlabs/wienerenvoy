//! Keep-awake assertion backed by `caffeinate`.
//!
//! Holds a long-lived `caffeinate` child process. `kill_on_drop(true)` plus an
//! explicit `release()` on graceful shutdown cover the normal lifecycle; a hard
//! SIGKILL of the daemon can still orphan the child, which a future PID-file
//! reconciliation (M1.1) will clean up on startup.

use std::sync::Mutex;

use async_trait::async_trait;
use tokio::process::{Child, Command};
use wienerenvoy_core::power::{KeepAwake, PowerError};

/// Keep-awake backed by macOS `caffeinate`.
pub struct CaffeinateKeeper {
    flags: String,
    child: Mutex<Option<Child>>,
}

impl CaffeinateKeeper {
    /// Create a keeper with the given `caffeinate` flags (e.g. `-s`).
    #[must_use]
    pub fn new(flags: impl Into<String>) -> Self {
        Self {
            flags: flags.into(),
            child: Mutex::new(None),
        }
    }
}

impl Default for CaffeinateKeeper {
    fn default() -> Self {
        Self::new("-s")
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
        tracing::info!(reason, pid = child.id(), "caffeinate engaged");
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

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "spawns a real caffeinate process; macOS only"]
    async fn engage_release_real_caffeinate() {
        let keeper = CaffeinateKeeper::new("-s");
        assert!(!keeper.is_engaged());
        keeper.engage("test").await.unwrap();
        assert!(keeper.is_engaged());
        assert!(keeper.holder_pid().is_some());
        keeper.release().await.unwrap();
        assert!(!keeper.is_engaged());
    }
}
