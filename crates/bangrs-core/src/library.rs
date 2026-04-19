use camino::Utf8PathBuf;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlbumId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: TrackId,
    pub path: Utf8PathBuf,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Duration,
    pub sample_rate: u32,
    pub channels: u16,
    pub replay_gain_db: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackRow {
    pub id: TrackId,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
}

impl From<&Track> for TrackRow {
    fn from(_t: &Track) -> Self {
        todo!("green: map Track to TrackRow, default missing fields to empty string")
    }
}

pub struct Library {
    tracks: Vec<Track>,
    by_id: HashMap<TrackId, usize>,
}

impl Library {
    pub fn new(_tracks: Vec<Track>) -> Self {
        todo!("green: store tracks, build by_id index")
    }
    pub fn get(&self, _id: TrackId) -> Option<&Track> {
        todo!("green")
    }
    pub fn iter(&self) -> impl Iterator<Item = &Track> {
        // Note to green: replace with `self.tracks.iter()` once bodies are filled in.
        std::iter::empty::<&Track>()
    }
    pub fn len(&self) -> usize {
        todo!("green")
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track(id: u64, title: &str) -> Track {
        Track {
            id: TrackId(id),
            path: format!("/tmp/{id}.wav").into(),
            title: title.into(),
            artist: Some("TestArtist".into()),
            album: Some("TestAlbum".into()),
            duration: Duration::from_secs(1),
            sample_rate: 44100,
            channels: 2,
            replay_gain_db: None,
        }
    }

    #[test]
    fn new_library_indexes_tracks_by_id() {
        let lib = Library::new(vec![sample_track(1, "a"), sample_track(2, "b")]);
        assert_eq!(lib.len(), 2);
        assert_eq!(lib.get(TrackId(1)).map(|t| t.title.as_str()), Some("a"));
        assert_eq!(lib.get(TrackId(2)).map(|t| t.title.as_str()), Some("b"));
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let lib = Library::new(vec![sample_track(1, "a")]);
        assert_eq!(lib.get(TrackId(9999)), None);
    }

    #[test]
    fn empty_library_is_empty() {
        let lib = Library::new(vec![]);
        assert!(lib.is_empty());
        assert_eq!(lib.len(), 0);
    }
}
