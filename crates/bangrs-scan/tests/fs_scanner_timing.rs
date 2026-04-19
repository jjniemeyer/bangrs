use bangrs_scan::{FilesystemScanner, Scanner};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[test]
fn scan_under_5s_for_fixtures() {
    let scanner = FilesystemScanner;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures");
    let start = Instant::now();
    let _: Vec<_> = scanner.scan(&root).collect();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(5), "scan took {elapsed:?}, budget is 5s");
}
