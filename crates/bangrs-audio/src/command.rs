use bangrs_core::TrackId;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Play(TrackId),
    Pause,
    Resume,
    Stop,
    Seek(Duration),
    Shutdown,
}
