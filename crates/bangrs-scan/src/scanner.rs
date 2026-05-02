use crate::error::ScanError;
use bangrs_core::{Track, TrackId};
use camino::Utf8PathBuf;
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub trait Scanner: Send + Sync {
    fn scan<'a>(&'a self, root: &Path) -> Box<dyn Iterator<Item = Result<Track, ScanError>> + 'a>;
}

pub struct FilesystemScanner;

fn probe(path: &Path, counter: &AtomicU64) -> Result<Track, ScanError> {
    let utf8_path = Utf8PathBuf::from_path_buf(path.to_path_buf())
        .unwrap_or_else(|p| Utf8PathBuf::from(p.to_string_lossy().into_owned()));
    let tagged = lofty::read_from_path(path).map_err(|source| ScanError::TagRead {
        path: utf8_path.clone(),
        source,
    })?;
    let props = tagged.properties();
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let title = tag
        .and_then(|t| t.title().map(|c| c.to_string()))
        .unwrap_or_else(|| {
            utf8_path
                .file_stem()
                .unwrap_or("")
                .to_string()
        });
    let artist = tag.and_then(|t| t.artist().map(|c| c.to_string()));
    let album = tag.and_then(|t| t.album().map(|c| c.to_string()));
    let id = TrackId(counter.fetch_add(1, Ordering::Relaxed));
    Ok(Track {
        id,
        path: utf8_path,
        title,
        artist,
        album,
        duration: props.duration(),
        sample_rate: props.sample_rate().unwrap_or(0),
        channels: props.channels().map(u16::from).unwrap_or(0),
        replay_gain_db: None,
    })
}

impl Scanner for FilesystemScanner {
    fn scan<'a>(&'a self, root: &Path) -> Box<dyn Iterator<Item = Result<Track, ScanError>> + 'a> {
        let counter = AtomicU64::new(0);
        let iter = walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|entry_res| match entry_res {
                Ok(e) if e.file_type().is_file() => Some(e),
                _ => None,
            })
            .filter(|e| {
                let ext = e
                    .path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                matches!(ext.as_str(), "mp3" | "flac" | "wav" | "ogg")
            })
            .map(move |entry| probe(entry.path(), &counter));
        Box::new(iter)
    }
}

pub struct FakeScanner {
    pub items: Vec<Result<Track, ScanError>>,
}

impl Scanner for FakeScanner {
    fn scan<'a>(&'a self, _root: &Path) -> Box<dyn Iterator<Item = Result<Track, ScanError>> + 'a> {
        let iter = self.items.iter().map(|r| match r {
            Ok(t) => Ok(t.clone()),
            Err(e) => Err(ScanError::UnsupportedFormat(e.to_string())),
        });
        Box::new(iter)
    }
}
