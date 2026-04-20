use bangrs_scan::{FilesystemScanner, Scanner};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

#[test]
fn scans_all_fixture_files() {
    let scanner = FilesystemScanner;
    let results: Vec<_> = scanner.scan(&fixtures_dir()).collect();
    // 5 fixtures: silence-1s.wav, sine-440-1s.mp3, sine-880-1s.flac, tagless.wav, corrupt-tags.mp3
    let ok_count = results.iter().filter(|r| r.is_ok()).count();
    let err_count = results.iter().filter(|r| r.is_err()).count();
    assert_eq!(ok_count + err_count, 5, "expected 5 total results, got {ok_count} ok + {err_count} err");
    assert!(ok_count >= 4, "expected at least 4 scannable fixtures, got {ok_count}");
}

#[test]
fn scan_returns_err_for_corrupt_tags() {
    let scanner = FilesystemScanner;
    let results: Vec<_> = scanner.scan(&fixtures_dir()).collect();
    assert!(results.iter().any(|r| r.is_err()), "corrupt-tags.mp3 must surface an Err");
}
