//! macOS power controller.
//!
//! M0: every action is a logged no-op returning `Ok`, so the daemon can hold a
//! `PowerController` without risking the machine. M1 replaces each body with the
//! real `pmset sleepnow` / `shutdown -r` / `shutdown -h` / `pmset schedule`
//! shellouts (argv arrays, never `sh -c`), with the sleep-default and
//! shutdown-confirm asymmetry enforced one layer up in the daemon.

use std::time::Duration;

use async_trait::async_trait;
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

#[async_trait]
impl PowerController for MacOsPower {
    async fn sleep(&self) -> Result<(), PowerError> {
        tracing::info!("M0 stub: sleep requested (no-op until M1)");
        Ok(())
    }

    async fn restart(&self, grace: Duration) -> Result<(), PowerError> {
        tracing::info!(grace_secs = grace.as_secs(), "M0 stub: restart requested");
        Ok(())
    }

    async fn shutdown(&self, grace: Duration) -> Result<(), PowerError> {
        tracing::info!(grace_secs = grace.as_secs(), "M0 stub: shutdown requested");
        Ok(())
    }

    async fn schedule_wake(&self, _spec: WakeSpec) -> Result<(), PowerError> {
        tracing::info!("M0 stub: schedule_wake requested");
        Ok(())
    }

    async fn power_state(&self) -> Result<PowerInfo, PowerError> {
        Ok(PowerInfo {
            on_ac_power: true,
            pending_wake: None,
        })
    }
}
