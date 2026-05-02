//! UI-thread side effects from the pick-library worker.
//!
//! The dispatched closure (which is `Send + 'static`) cannot capture the
//! `Rc<RefCell<...>>` cells living in `main.rs`. We thread the side effects
//! through a `Send + Sync` trait object instead. The production impl
//! (`MainSideEffects`) holds `Arc<Mutex<...>>` companions for `link`, `vm`,
//! and `handlers`, plus a `slint::Weak<MainWindow>` for `bind` reachability.

use crate::handlers::{wire_handlers, ActivationHandler, CommandSink, SelectionHandler};
use crate::PlayerHandle;
use bangrs_audio::Event;
use bangrs_core::{Library, Track, TrackId, ViewModel};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub trait UiSideEffects: Send + Sync {
    /// A successful scan: install the new library, shut down the old engine,
    /// spawn a new one, rewire the selection/activation handlers, refresh the
    /// view model, and call `bind` on the upgraded window.
    fn apply_library(&self, path: PathBuf, tracks: Vec<Track>);

    /// A failed scan: surface the error in the banner without disturbing the
    /// currently-loaded library or its engine.
    fn set_banner(&self, message: String);
}

pub struct EngineLink {
    pub handle: PlayerHandle,
    pub ev_rx: crossbeam_channel::Receiver<Event>,
    pub track_ids: Vec<TrackId>,
}

fn spawn_engine(library: Arc<Library>, track_ids: Vec<TrackId>) -> EngineLink {
    use bangrs_audio::{AudioEngine, CpalEngine};

    let (cmd_tx, cmd_rx) = mpsc::sync_channel(32);
    let (ev_tx, ev_rx) = crossbeam_channel::bounded(64);
    let position = Arc::new(AtomicU64::new(0));

    let pos_audio = position.clone();
    let lib_audio = library.clone();
    thread::spawn(move || {
        CpalEngine.run(lib_audio, cmd_rx, ev_tx, pos_audio);
    });

    let handle = PlayerHandle::new(cmd_tx, position);
    EngineLink {
        handle,
        ev_rx,
        track_ids,
    }
}

pub struct MainSideEffects {
    pub link: Arc<Mutex<Option<EngineLink>>>,
    pub vm: Arc<Mutex<ViewModel>>,
    pub handlers: Arc<Mutex<Option<(SelectionHandler, ActivationHandler)>>>,
    pub window_weak: slint::Weak<crate::MainWindow>,
}

impl UiSideEffects for MainSideEffects {
    fn apply_library(&self, _path: PathBuf, tracks: Vec<Track>) {
        let track_ids: Vec<TrackId> = tracks.iter().map(|t| t.id).collect();
        let library = Arc::new(Library::new(tracks));

        if let Some(old) = self.link.lock().unwrap().take() {
            let _ = old.handle.shutdown();
        }
        let new_link = spawn_engine(library.clone(), track_ids.clone());
        let sink: Arc<dyn CommandSink> = Arc::new(new_link.handle.clone());
        *self.handlers.lock().unwrap() = Some(wire_handlers(sink, Arc::new(track_ids)));
        *self.link.lock().unwrap() = Some(new_link);

        let updated = self.vm.lock().unwrap().clone().set_library(library);
        *self.vm.lock().unwrap() = updated;

        if let Some(w) = self.window_weak.upgrade() {
            let vm = self.vm.lock().unwrap();
            crate::bind(&w, &vm);
        }
    }

    fn set_banner(&self, message: String) {
        {
            let mut vm = self.vm.lock().unwrap();
            vm.error_banner = Some(message);
        }
        if let Some(w) = self.window_weak.upgrade() {
            let vm = self.vm.lock().unwrap();
            crate::bind(&w, &vm);
        }
    }
}
