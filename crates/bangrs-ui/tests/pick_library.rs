use bangrs_core::{Track, TrackId};
use bangrs_scan::{FakeScanner, ScanError, Scanner};
use bangrs_ui::dispatcher::SpyDispatcher;
use bangrs_ui::pick_handler::on_pick_library;
use bangrs_ui::pick_state::PickState;
use bangrs_ui::picker::FolderPicker;
use bangrs_ui::side_effects::UiSideEffects;
use camino::Utf8PathBuf;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

// ----- Fakes -----

struct FakeFolderPicker {
    result: Option<PathBuf>,
    sleep: Duration,
    ran_on: Mutex<Option<ThreadId>>,
}

impl FakeFolderPicker {
    fn new(result: Option<PathBuf>) -> Arc<Self> {
        Arc::new(Self {
            result,
            sleep: Duration::ZERO,
            ran_on: Mutex::new(None),
        })
    }

    fn with_sleep(self: Arc<Self>, d: Duration) -> Arc<Self> {
        // Construct a new Arc; mutating shared state via Arc::get_mut would
        // race with worker threads.
        Arc::new(Self {
            result: self.result.clone(),
            sleep: d,
            ran_on: Mutex::new(None),
        })
    }

    fn ran_on(&self) -> Option<ThreadId> {
        *self.ran_on.lock().unwrap()
    }
}

impl FolderPicker for FakeFolderPicker {
    fn pick_folder(&self) -> Option<PathBuf> {
        *self.ran_on.lock().unwrap() = Some(std::thread::current().id());
        std::thread::sleep(self.sleep);
        self.result.clone()
    }
}

#[derive(Default)]
struct RecordingSideEffects {
    inner: Mutex<RecordingState>,
}

#[derive(Default)]
struct RecordingState {
    applied: Vec<(PathBuf, Vec<Track>)>,
    banners: Vec<String>,
}

impl RecordingSideEffects {
    fn applied_paths(&self) -> Vec<PathBuf> {
        self.inner
            .lock()
            .unwrap()
            .applied
            .iter()
            .map(|(p, _)| p.clone())
            .collect()
    }

    fn banners(&self) -> Vec<String> {
        self.inner.lock().unwrap().banners.clone()
    }
}

impl UiSideEffects for RecordingSideEffects {
    fn apply_library(&self, path: PathBuf, tracks: Vec<Track>) {
        self.inner.lock().unwrap().applied.push((path, tracks));
    }

    fn set_banner(&self, message: String) {
        self.inner.lock().unwrap().banners.push(message);
    }
}

fn track(id: u64, title: &str) -> Track {
    Track {
        id: TrackId(id),
        path: Utf8PathBuf::from(format!("/tmp/{title}.flac")),
        title: title.to_string(),
        artist: None,
        album: None,
        duration: Duration::from_secs(30),
        sample_rate: 44_100,
        channels: 2,
        replay_gain_db: None,
    }
}

// ----- Tests -----

#[test]
fn pick_handler_returns_immediately() {
    // Slow picker (200 ms) + slow scanner (default).
    let picker = FakeFolderPicker::new(Some("/lib".into())).with_sleep(Duration::from_millis(200));
    let scanner: Arc<dyn Scanner + Send + Sync> = Arc::new(FakeScanner { items: vec![] });
    let dispatcher = Arc::new(SpyDispatcher::default());
    let pick_state = Arc::new(PickState::new());
    let side_effects: Arc<dyn UiSideEffects> = Arc::new(RecordingSideEffects::default());

    let start = Instant::now();
    on_pick_library(picker, scanner, dispatcher, pick_state, side_effects);
    assert!(
        start.elapsed() < Duration::from_millis(50),
        "on_pick_library must return immediately, not block on the picker"
    );
}

#[test]
fn dialog_runs_off_caller_thread() {
    let caller = std::thread::current().id();
    let picker = FakeFolderPicker::new(Some("/lib".into()));
    let scanner: Arc<dyn Scanner + Send + Sync> = Arc::new(FakeScanner { items: vec![] });
    let dispatcher = Arc::new(SpyDispatcher::default());
    let pick_state = Arc::new(PickState::new());
    let side_effects: Arc<dyn UiSideEffects> = Arc::new(RecordingSideEffects::default());

    on_pick_library(
        picker.clone(),
        scanner,
        dispatcher,
        pick_state,
        side_effects,
    );
    // Give the worker thread time to call into picker.
    std::thread::sleep(Duration::from_millis(50));

    assert_ne!(
        picker.ran_on().expect("picker should have been called"),
        caller,
        "picker.pick_folder must run on a worker thread, not the caller"
    );
}

#[test]
fn cancel_is_noop() {
    let picker = FakeFolderPicker::new(None); // user cancels
    let scanner: Arc<dyn Scanner + Send + Sync> = Arc::new(FakeScanner { items: vec![] });
    let dispatcher = Arc::new(SpyDispatcher::default());
    let pick_state = Arc::new(PickState::new());
    let side_effects = Arc::new(RecordingSideEffects::default());
    let side_effects_dyn: Arc<dyn UiSideEffects> = side_effects.clone();

    on_pick_library(
        picker,
        scanner,
        dispatcher.clone(),
        pick_state,
        side_effects_dyn,
    );
    std::thread::sleep(Duration::from_millis(50));
    dispatcher.flush();

    assert_eq!(
        dispatcher.dispatched_count(),
        0,
        "cancelled pick must not dispatch a UI-thread closure"
    );
    assert!(
        side_effects.applied_paths().is_empty(),
        "cancelled pick must not call apply_library"
    );
    assert!(
        side_effects.banners().is_empty(),
        "cancelled pick must not set a banner"
    );
}

#[test]
fn superseded_pick_dropped() {
    let picker_slow =
        FakeFolderPicker::new(Some("/library1".into())).with_sleep(Duration::from_millis(200));
    let picker_fast = FakeFolderPicker::new(Some("/library2".into()));
    let scanner: Arc<dyn Scanner + Send + Sync> = Arc::new(FakeScanner { items: vec![] });
    let dispatcher = Arc::new(SpyDispatcher::default());
    let pick_state = Arc::new(PickState::new());
    let side_effects = Arc::new(RecordingSideEffects::default());
    let side_effects_dyn: Arc<dyn UiSideEffects> = side_effects.clone();

    // First pick (slow): worker runs picker, sleeps 200 ms, then dispatches.
    on_pick_library(
        picker_slow,
        scanner.clone(),
        dispatcher.clone(),
        pick_state.clone(),
        side_effects_dyn.clone(),
    );
    // Brief gap, then second pick (fast). This bumps pick_state.next() to 2.
    std::thread::sleep(Duration::from_millis(20));
    on_pick_library(
        picker_fast,
        scanner,
        dispatcher.clone(),
        pick_state,
        side_effects_dyn,
    );
    // Wait for both workers to finish dispatching.
    std::thread::sleep(Duration::from_millis(300));
    dispatcher.flush();

    // Both workers dispatched (count == 2 before flush), but only one
    // apply_library landed: the second pick (with seq == 1, matching current).
    let paths = side_effects.applied_paths();
    assert_eq!(
        paths.len(),
        1,
        "exactly one apply_library should land; got {paths:?}"
    );
    assert_eq!(paths[0], PathBuf::from("/library2"));
}

#[test]
fn scan_error_routes_to_banner() {
    let picker = FakeFolderPicker::new(Some("/lib".into()));
    let scanner: Arc<dyn Scanner + Send + Sync> = Arc::new(FakeScanner {
        items: vec![Err(ScanError::UnsupportedFormat("disk full".into()))],
    });
    let dispatcher = Arc::new(SpyDispatcher::default());
    let pick_state = Arc::new(PickState::new());
    let side_effects = Arc::new(RecordingSideEffects::default());
    let side_effects_dyn: Arc<dyn UiSideEffects> = side_effects.clone();

    on_pick_library(
        picker,
        scanner,
        dispatcher.clone(),
        pick_state,
        side_effects_dyn,
    );
    std::thread::sleep(Duration::from_millis(50));
    dispatcher.flush();

    let banners = side_effects.banners();
    assert_eq!(banners.len(), 1, "scan error must produce exactly one banner");
    assert!(
        banners[0].starts_with("Library scan failed:"),
        "banner text format wrong: {:?}",
        banners[0]
    );
    assert!(
        side_effects.applied_paths().is_empty(),
        "scan error must NOT call apply_library (don't show a partial library)"
    );
}

#[test]
fn successful_scan_calls_apply_library() {
    let picker = FakeFolderPicker::new(Some("/lib".into()));
    let scanner: Arc<dyn Scanner + Send + Sync> = Arc::new(FakeScanner {
        items: vec![Ok(track(1, "Alpha")), Ok(track(2, "Beta"))],
    });
    let dispatcher = Arc::new(SpyDispatcher::default());
    let pick_state = Arc::new(PickState::new());
    let side_effects = Arc::new(RecordingSideEffects::default());
    let side_effects_dyn: Arc<dyn UiSideEffects> = side_effects.clone();

    on_pick_library(
        picker,
        scanner,
        dispatcher.clone(),
        pick_state,
        side_effects_dyn,
    );
    std::thread::sleep(Duration::from_millis(50));
    dispatcher.flush();

    let applied = side_effects.applied_paths();
    assert_eq!(applied, vec![PathBuf::from("/lib")]);
    assert!(side_effects.banners().is_empty());
}
