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
    fn from(t: &Track) -> Self {
        TrackRow {
            id: t.id,
            title: t.title.clone(),
            artist: t.artist.clone().unwrap_or_default(),
            album: t.album.clone().unwrap_or_default(),
            duration_ms: t.duration.as_millis() as u64,
        }
    }
}

pub struct Library {
    tracks: Vec<Track>,
    by_id: HashMap<TrackId, usize>,
}

impl Library {
    pub fn new(tracks: Vec<Track>) -> Self {
        let by_id = tracks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id, i))
            .collect();
        Self { tracks, by_id }
    }
    pub fn get(&self, id: TrackId) -> Option<&Track> {
        self.by_id.get(&id).map(|&i| &self.tracks[i])
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Track> {
        self.tracks.iter()
    }
    pub fn len(&self) -> usize {
        self.tracks.len()
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
