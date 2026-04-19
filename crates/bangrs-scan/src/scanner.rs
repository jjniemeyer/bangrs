use crate::error::ScanError;
use bangrs_core::Track;
use std::path::Path;

pub trait Scanner {
    fn scan<'a>(&'a self, root: &Path) -> Box<dyn Iterator<Item = Result<Track, ScanError>> + 'a>;
}

pub struct FilesystemScanner;

impl Scanner for FilesystemScanner {
    fn scan<'a>(&'a self, _root: &Path) -> Box<dyn Iterator<Item = Result<Track, ScanError>> + 'a> {
        todo!("green: walkdir, filter audio ext, lofty probe, emit Track or ScanError")
    }
}

pub struct FakeScanner {
    pub items: Vec<Result<Track, ScanError>>,
}

impl Scanner for FakeScanner {
    fn scan<'a>(&'a self, _root: &Path) -> Box<dyn Iterator<Item = Result<Track, ScanError>> + 'a> {
        todo!("green: drain self.items into an iterator")
    }
}
