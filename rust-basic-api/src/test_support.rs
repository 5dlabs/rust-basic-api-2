//! Test-only helpers shared across modules.

#[cfg(test)]
pub(crate) mod env {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    /// Acquire a global mutex that serializes access to process-wide environment variables.
    ///
    /// Tests that manipulate environment variables should hold this guard for the duration of
    /// their changes to avoid cross-test interference when executing in parallel.
    pub(crate) fn guard() -> MutexGuard<'static, ()> {
        ENV_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("environment mutex poisoned")
    }
}
