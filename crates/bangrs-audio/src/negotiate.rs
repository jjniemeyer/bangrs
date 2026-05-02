//! Sample-rate / format negotiation between a track and a cpal device's
//! supported configurations.
//!
//! The engine queries the device's supported configs once per track-start
//! and asks `negotiate_config(...)` for a `ChosenConfig` it can build a
//! `cpal::StreamConfig` from. If no config matches, the engine emits a
//! `TrackFailed` event whose reason is the `Display` of `NoMatchingConfig`.

/// Minimal, hashable view of a `cpal::SupportedStreamConfigRange` that we can
/// construct in tests without a real audio device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRange {
    pub channels: u16,
    pub sample_format: SampleFormat,
    pub min_rate_hz: u32,
    pub max_rate_hz: u32,
}

/// Subset of `cpal::SampleFormat` we care about. Matches cpal's variants 1:1
/// for the cases the engine uses (i16 fallback, f32 default).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SampleFormat {
    I16,
    F32,
}

/// A negotiated config ready to be turned into a `cpal::StreamConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChosenConfig {
    pub channels: u16,
    pub sample_format: SampleFormat,
    pub rate_hz: u32,
}

/// Returned by `negotiate_config` when no `ConfigRange` matches the track.
/// `Display` produces the reason string surfaced to the UI via `TrackFailed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoMatchingConfig {
    pub track_rate_hz: u32,
    /// Sorted, deduplicated list of rates supported by entries that matched
    /// `desired_channels` and `desired_format` but not the track's rate.
    /// Used by `Display` to suggest "available rates" to the user.
    pub nearby_rates_hz: Vec<u32>,
}

/// Pick a `ChosenConfig` for the given track. See module docs.
pub fn negotiate_config(
    _supported: &[ConfigRange],
    _track_rate_hz: u32,
    _desired_format: SampleFormat,
    _desired_channels: u16,
) -> Result<ChosenConfig, NoMatchingConfig> {
    unimplemented!("green: implement per Shape's three-step algorithm")
}

impl std::fmt::Display for NoMatchingConfig {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!("green: implement \"no cpal config supports rate {{}} Hz; available rates: {{}}; track skipped\"")
    }
}
