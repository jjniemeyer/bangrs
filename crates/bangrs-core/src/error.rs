use crate::library::TrackId;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CoreError {
    #[error("track id {0:?} not in library")]
    UnknownTrack(TrackId),
    #[error("invalid state transition: {from} → {to}")]
    InvalidTransition { from: &'static str, to: &'static str },
}
