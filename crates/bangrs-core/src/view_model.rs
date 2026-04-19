use crate::event::Event;
use crate::library::{Library, TrackId, TrackRow};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ViewModel {
    pub is_playing: bool,
    pub is_paused: bool,
    pub current_track: Option<TrackId>,
    pub position_ms: u64,
    pub tracks: Vec<TrackRow>,
    pub error_banner: Option<String>,
}

impl ViewModel {
    pub fn apply(self, _ev: &Event) -> Self {
        todo!("green: exhaustive match on Event")
    }
    pub fn set_library(self, _lib: Arc<Library>) -> Self {
        todo!("green: populate tracks from lib.iter()")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{Track, TrackId};
    use camino::Utf8PathBuf;
    use std::time::Duration;

    fn sample_track(id: u64) -> Track {
        Track {
            id: TrackId(id),
            path: Utf8PathBuf::from(format!("/tmp/{id}.wav")),
            title: format!("t{id}"),
            artist: None,
            album: None,
            duration: Duration::from_secs(1),
            sample_rate: 44100,
            channels: 2,
            replay_gain_db: None,
        }
    }

    #[test]
    fn default_is_empty() {
        let vm = ViewModel::default();
        assert!(!vm.is_playing && !vm.is_paused);
        assert!(vm.current_track.is_none());
        assert_eq!(vm.position_ms, 0);
        assert!(vm.tracks.is_empty());
    }

    #[test]
    fn playback_started_sets_is_playing() {
        let vm = ViewModel::default().apply(&Event::PlaybackStarted { track_id: TrackId(7) });
        assert!(vm.is_playing);
        assert!(!vm.is_paused);
        assert_eq!(vm.current_track, Some(TrackId(7)));
    }

    #[test]
    fn playback_paused_clears_is_playing() {
        let vm = ViewModel { is_playing: true, is_paused: false, ..Default::default() }
            .apply(&Event::PlaybackPaused);
        assert!(!vm.is_playing);
        assert!(vm.is_paused);
    }

    #[test]
    fn playback_stopped_clears_current_track() {
        let vm = ViewModel {
            is_playing: true,
            current_track: Some(TrackId(3)),
            ..Default::default()
        }
        .apply(&Event::PlaybackStopped);
        assert!(!vm.is_playing);
        assert!(vm.current_track.is_none());
    }

    #[test]
    fn track_failed_sets_error_banner() {
        let vm = ViewModel::default().apply(&Event::TrackFailed {
            track_id: TrackId(1),
            reason: "decode error".into(),
        });
        assert_eq!(vm.error_banner.as_deref(), Some("decode error"));
    }

    #[test]
    fn position_update_sets_position_ms() {
        let vm = ViewModel::default().apply(&Event::PositionUpdate { ms: 12_345 });
        assert_eq!(vm.position_ms, 12_345);
    }

    #[test]
    fn set_library_populates_tracks() {
        let lib = Arc::new(Library::new(vec![sample_track(1), sample_track(2), sample_track(3)]));
        let vm = ViewModel::default().set_library(lib);
        assert_eq!(vm.tracks.len(), 3);
    }
}
