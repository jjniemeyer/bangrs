use camino::Utf8PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("tag read failed for {path}: {source}")]
    TagRead {
        path: Utf8PathBuf,
        #[source]
        source: lofty::error::LoftyError,
    },
    #[error("walk error: {0}")]
    Walk(#[from] walkdir::Error),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}
