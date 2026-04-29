//! Wire-up helpers for selection vs. activation handling.
//!
//! Tests in `tests/wiring.rs` exercise these closures directly without a
//! Slint event loop. The `CommandSink` indirection lets tests record
//! commands without spinning up a real `PlayerHandle` / audio thread.

use bangrs_audio::{Command, HandleError};
use bangrs_core::TrackId;
use std::sync::Arc;

/// Anything that can accept playback commands. Implemented for
/// `bangrs_ui::PlayerHandle` in production and for a recording fake in tests.
pub trait CommandSink: Send + Sync {
    fn send(&self, cmd: Command) -> Result<(), HandleError>;
}

impl CommandSink for crate::PlayerHandle {
    fn send(&self, cmd: Command) -> Result<(), HandleError> {
        match cmd {
            Command::Play(id) => self.play(id),
            Command::Pause => self.pause(),
            Command::Resume => self.resume(),
            Command::Stop => self.stop(),
            Command::Seek(d) => self.seek(d),
            Command::Shutdown => Err(HandleError::ThreadClosed),
        }
    }
}

pub type SelectionHandler = Box<dyn Fn(i32) + Send + Sync>;
pub type ActivationHandler = Box<dyn Fn(i32) + Send + Sync>;

/// Construct the selection-changed and track-activated closures.
///
/// The returned `selection_handler` is a no-op for audio purposes — selection
/// alone never plays. The `activation_handler` reads `track_ids[idx]` and
/// emits `Command::Play(track_id)` via the sink. Negative or out-of-bounds
/// indices are silently dropped (defensive — Slint can fire activation
/// during a `set_tracks` rebind).
pub fn wire_handlers(
    _sink: Arc<dyn CommandSink>,
    _track_ids: Arc<Vec<TrackId>>,
) -> (SelectionHandler, ActivationHandler) {
    todo!("green agent implements")
}
