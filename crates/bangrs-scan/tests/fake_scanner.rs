use bangrs_core::{Track, TrackId};
use bangrs_scan::{FakeScanner, Scanner};
use camino::Utf8PathBuf;
use std::path::Path;
use std::time::Duration;

fn sample(id: u64) -> Track {
    Track {
        id: TrackId(id),
        path: Utf8PathBuf::from(format!("/tmp/{id}.wav")),
        title: "t".into(),
        artist: None,
        album: None,
        duration: Duration::from_secs(1),
        sample_rate: 44100,
        channels: 2,
        replay_gain_db: None,
    }
}

#[test]
fn fake_yields_scripted_items() {
    let fake = FakeScanner { items: vec![Ok(sample(1)), Ok(sample(2))] };
    let out: Vec<_> = fake.scan(Path::new("/ignored")).collect();
    assert_eq!(out.len(), 2);
    assert!(out.iter().all(|r| r.is_ok()));
}
