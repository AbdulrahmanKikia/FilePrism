//! Cooperative cancellation for conversion jobs.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::types::Error;

/// Shared cancellation flag for a single job (or a batch of jobs).
///
/// Clone freely; all clones share the same flag. Call [`CancellationToken::cancel`]
/// from the UI thread; the pipeline checks via [`CancellationToken::check`].
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation. Idempotent.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns `true` if cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Returns [`Error::Cancelled`] when cancellation was requested.
    pub fn check(&self) -> Result<(), Error> {
        if self.is_cancelled() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}
