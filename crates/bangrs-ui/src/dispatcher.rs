//! Cross-thread closure dispatcher.
//!
//! `SlintDispatcher` (production) marshals onto the Slint event loop via
//! `slint::invoke_from_event_loop`. `DirectDispatcher` and `SpyDispatcher`
//! (test-only) run the closure synchronously on the caller's thread; the spy
//! variant queues closures so tests can flush them at controlled points.

use std::sync::Mutex;

pub trait Dispatcher: Send + Sync {
    fn dispatch(&self, f: Box<dyn FnOnce() + Send + 'static>);
}

pub struct SlintDispatcher;

impl Dispatcher for SlintDispatcher {
    fn dispatch(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        // The Err case only triggers after the event loop has shut down,
        // i.e. the app is dying — discard.
        let _ = slint::invoke_from_event_loop(move || f());
    }
}

#[cfg(test)]
pub struct DirectDispatcher;

#[cfg(test)]
impl Dispatcher for DirectDispatcher {
    fn dispatch(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        f();
    }
}

pub struct SpyDispatcher {
    inner: Mutex<SpyState>,
}

#[derive(Default)]
struct SpyState {
    queued: Vec<Box<dyn FnOnce() + Send + 'static>>,
}

impl Default for SpyDispatcher {
    fn default() -> Self {
        Self { inner: Mutex::new(SpyState::default()) }
    }
}

impl Dispatcher for SpyDispatcher {
    fn dispatch(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        self.inner.lock().unwrap().queued.push(f);
    }
}

impl SpyDispatcher {
    /// Run every queued closure on the calling thread, in order.
    pub fn flush(&self) {
        let queued: Vec<_> = std::mem::take(&mut self.inner.lock().unwrap().queued);
        for f in queued {
            f();
        }
    }

    /// Number of closures queued but not yet flushed.
    pub fn dispatched_count(&self) -> usize {
        self.inner.lock().unwrap().queued.len()
    }
}
