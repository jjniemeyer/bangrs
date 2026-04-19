#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("decode failed: {0}")]
    Decode(String),
    #[error("output device error: {0}")]
    Output(String),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("sample rate mismatch: source={source_hz}, device={device_hz}")]
    SampleRateMismatch { source_hz: u32, device_hz: u32 },
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum HandleError {
    #[error("audio thread closed")]
    ThreadClosed,
    #[error("command queue full")]
    QueueFull,
}
