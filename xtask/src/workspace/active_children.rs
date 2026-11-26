use std::collections::BTreeSet;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, PoisonError};

static ACTIVE_PROCESS_IDENTIFIERS: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());

pub struct ActiveChild {
    process_identifier: u32,
}

impl ActiveChild {
    #[must_use]
    pub fn register(process_identifier: u32) -> Self {
        lock_identifiers().insert(process_identifier);
        Self { process_identifier }
    }
}

impl Drop for ActiveChild {
    fn drop(&mut self) {
        lock_identifiers().remove(&self.process_identifier);
    }
}

pub fn terminate_active_children() {
    let identifiers: Vec<u32> = lock_identifiers().iter().copied().collect();
    for identifier in identifiers {
        drop(
            Command::new("kill")
                .args(["-KILL", &identifier.to_string()])
                .status(),
        );
    }
}

fn lock_identifiers() -> MutexGuard<'static, BTreeSet<u32>> {
    ACTIVE_PROCESS_IDENTIFIERS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}
