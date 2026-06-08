//! Keep-awake assertion backed by `caffeinate`.
//!
//! M0: tracks engaged state in an atomic, no child process. M1 spawns
//! `caffeinate -s` as a long-lived child, persists its PID, reconciles orphans
//! across daemon restarts (`kill(pid, 0)` liveness plus `ps comm` re-verify
//! against PID reuse), and releases on graceful shutdown.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use wienerenvoy_core::power::{KeepAwake, PowerError};

/// Keep-awake backed by macOS `caffeinate`.
#[derive(Debug, Default)]
pub struct CaffeinateKeeper {
    engaged: AtomicBool,
}

impl CaffeinateKeeper {
    #[must_use]
    pub fn new() -> Self {
        Self {
            engaged: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl KeepAwake for CaffeinateKeeper {
    async fn engage(&self, reason: &str) -> Result<(), PowerError> {
        tracing::info!(
            reason,
            "M0 stub: keep-awake engaged (no caffeinate child until M1)"
        );
        self.engaged.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn release(&self) -> Result<(), PowerError> {
        tracing::info!("M0 stub: keep-awake released");
        self.engaged.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_engaged(&self) -> bool {
        self.engaged.load(Ordering::SeqCst)
    }

    fn holder_pid(&self) -> Option<u32> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn engage_release_toggles_state() {
        let k = CaffeinateKeeper::new();
        assert!(!k.is_engaged());
        k.engage("test").await.unwrap();
        assert!(k.is_engaged());
        k.release().await.unwrap();
        assert!(!k.is_engaged());
    }
}
