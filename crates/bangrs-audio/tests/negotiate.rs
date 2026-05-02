use bangrs_audio::negotiate::{
    negotiate_config, ChosenConfig, ConfigRange, NoMatchingConfig, SampleFormat,
};

fn r(min: u32, max: u32, ch: u16, fmt: SampleFormat) -> ConfigRange {
    ConfigRange {
        min_rate_hz: min,
        max_rate_hz: max,
        channels: ch,
        sample_format: fmt,
    }
}

#[test]
fn picks_matching_rate() {
    let supported = vec![
        r(44100, 44100, 2, SampleFormat::F32),
        r(48000, 48000, 2, SampleFormat::F32),
        r(96000, 96000, 2, SampleFormat::F32),
    ];
    let chosen = negotiate_config(&supported, 44100, SampleFormat::F32, 2).unwrap();
    assert_eq!(
        chosen,
        ChosenConfig {
            channels: 2,
            sample_format: SampleFormat::F32,
            rate_hz: 44100,
        }
    );
}

#[test]
fn picks_within_range() {
    let supported = vec![r(44100, 96000, 2, SampleFormat::F32)];
    let chosen = negotiate_config(&supported, 48000, SampleFormat::F32, 2).unwrap();
    assert_eq!(chosen.rate_hz, 48000);
}

#[test]
fn near_match_within_1_percent_uses_supported_endpoint() {
    let supported = vec![r(44100, 44100, 2, SampleFormat::F32)];
    let chosen = negotiate_config(&supported, 44099, SampleFormat::F32, 2).unwrap();
    assert_eq!(chosen.rate_hz, 44100);
}

#[test]
fn no_match_produces_helpful_error() {
    let supported = vec![
        r(48000, 48000, 2, SampleFormat::F32),
        r(96000, 96000, 2, SampleFormat::F32),
    ];
    let err = negotiate_config(&supported, 44100, SampleFormat::F32, 2).unwrap_err();
    assert_eq!(err.track_rate_hz, 44100);
    assert!(err.nearby_rates_hz.contains(&48000));
    assert!(err.nearby_rates_hz.contains(&96000));
    let msg = format!("{}", err);
    assert!(msg.contains("44100"));
    assert!(msg.contains("48000"));
    assert!(msg.contains("track skipped"));
}

#[test]
fn channel_mismatch_no_match() {
    let supported = vec![r(44100, 44100, 1, SampleFormat::F32)]; // mono only
    let err = negotiate_config(&supported, 44100, SampleFormat::F32, 2).unwrap_err();
    assert_eq!(err.track_rate_hz, 44100);
    assert!(err.nearby_rates_hz.is_empty()); // no entries with stereo
}

#[test]
fn format_mismatch_no_match() {
    let supported = vec![r(44100, 44100, 2, SampleFormat::I16)];
    let err = negotiate_config(&supported, 44100, SampleFormat::F32, 2).unwrap_err();
    assert_eq!(err.track_rate_hz, 44100);
    assert!(err.nearby_rates_hz.is_empty());
}

#[test]
fn empty_supported_no_match() {
    let supported: Vec<ConfigRange> = vec![];
    let err = negotiate_config(&supported, 44100, SampleFormat::F32, 2).unwrap_err();
    assert!(err.nearby_rates_hz.is_empty());
}

#[test]
#[ignore]
fn smoke_against_real_device() {
    // Green: copy the body from the Shape spec — query the default cpal output
    // device's supported configs, run negotiate_config against [22050, 44100,
    // 48000, 96000], eprintln! the results.
}
