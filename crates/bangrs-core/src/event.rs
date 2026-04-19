use crate::library::TrackId;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    PlaybackStarted { track_id: TrackId },
    PlaybackPaused,
    PlaybackResumed,
    PlaybackStopped,
    TrackEnded { track_id: TrackId },
    TrackFailed { track_id: TrackId, reason: String },
    FatalError(String),
    PositionUpdate { ms: u64 },
}
