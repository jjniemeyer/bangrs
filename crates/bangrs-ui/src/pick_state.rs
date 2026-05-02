//! Per-click sequence counter for re-pick race resolution.
//!
//! Each pick increments the counter and remembers its issued seq. When the
//! worker thread eventually marshals back to the UI thread, it compares the
//! current seq against its remembered seq; if they differ, a newer pick has
//! superseded this one and the result is dropped silently.

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct PickState {
    seq: AtomicU64,
}

impl PickState {
    pub fn new() -> Self {
        Self { seq: AtomicU64::new(0) }
    }

    /// Issued at the start of each pick. Returns this pick's seq.
    pub fn next(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::SeqCst)
    }

    /// Latest seq issued. Used by the UI-thread callback to check supersession.
    pub fn current(&self) -> u64 {
        self.seq.load(Ordering::SeqCst)
    }
}
