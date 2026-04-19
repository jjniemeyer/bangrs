/// Integration test requiring a real output device.
/// Run with: cargo test -p bangrs-audio --test cpal_smoke -- --ignored
#[test]
#[ignore]
fn cpal_stream_builds_on_default_device() {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let _config = device.default_output_config().expect("no default config");
    // Successful if we got here without panic
}
