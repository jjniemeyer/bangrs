use crate::negotiate::NoMatchingConfig;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("decode failed: {0}")]
    Decode(String),
    #[error("output device error: {0}")]
    Output(String),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("{0}")]
    NoMatchingConfig(NoMatchingConfig),
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum HandleError {
    #[error("audio thread closed")]
    ThreadClosed,
    #[error("command queue full")]
    QueueFull,
}
