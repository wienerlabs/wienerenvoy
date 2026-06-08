//! Recording mocks for the power and keep-awake traits, behind the `testutil`
//! feature. Tests assert against recorded calls so they never touch real power.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;

use crate::power::{KeepAwake, PowerController, PowerError, PowerInfo, WakeSpec};

/// One recorded power call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerCall {
    Sleep,
    Restart(u64),
    Shutdown(u64),
    ScheduleWake,
    PowerState,
}

/// A `PowerController` that records calls instead of touching the machine.
pub struct MockPower {
    calls: Mutex<Vec<PowerCall>>,
    on_ac: bool,
}

impl Default for MockPower {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            on_ac: true,
        }
    }
}

impl MockPower {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of the calls recorded so far.
    #[must_use]
    pub fn calls(&self) -> Vec<PowerCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl PowerController for MockPower {
    async fn sleep(&self) -> Result<(), PowerError> {
        self.calls.lock().unwrap().push(PowerCall::Sleep);
        Ok(())
    }

    async fn restart(&self, grace: Duration) -> Result<(), PowerError> {
        self.calls
            .lock()
            .unwrap()
            .push(PowerCall::Restart(grace.as_secs()));
        Ok(())
    }

    async fn shutdown(&self, grace: Duration) -> Result<(), PowerError> {
        self.calls
            .lock()
            .unwrap()
            .push(PowerCall::Shutdown(grace.as_secs()));
        Ok(())
    }

    async fn schedule_wake(&self, _spec: WakeSpec) -> Result<(), PowerError> {
        self.calls.lock().unwrap().push(PowerCall::ScheduleWake);
        Ok(())
    }

    async fn power_state(&self) -> Result<PowerInfo, PowerError> {
        self.calls.lock().unwrap().push(PowerCall::PowerState);
        Ok(PowerInfo {
            on_ac_power: self.on_ac,
            pending_wake: None,
        })
    }
}

/// A `KeepAwake` that tracks engaged state in memory.
#[derive(Default)]
pub struct MockKeepAwake {
    engaged: AtomicBool,
}

impl MockKeepAwake {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KeepAwake for MockKeepAwake {
    async fn engage(&self, _reason: &str) -> Result<(), PowerError> {
        self.engaged.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn release(&self) -> Result<(), PowerError> {
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
