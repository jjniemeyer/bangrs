//! UI-thread side effects from the pick-library worker.
//!
//! The dispatched closure (which is `Send + 'static`) cannot capture the
//! `Rc<RefCell<...>>` cells living in `main.rs`. We thread the side effects
//! through a `Send + Sync` trait object instead. The production impl
//! (`MainSideEffects`) holds `Arc<Mutex<...>>` companions for `link`, `vm`,
//! and `handlers`, plus a `slint::Weak<MainWindow>` for `bind` reachability.

use bangrs_core::Track;
use std::path::PathBuf;

pub trait UiSideEffects: Send + Sync {
    /// A successful scan: install the new library, shut down the old engine,
    /// spawn a new one, rewire the selection/activation handlers, refresh the
    /// view model, and call `bind` on the upgraded window.
    fn apply_library(&self, path: PathBuf, tracks: Vec<Track>);

    /// A failed scan: surface the error in the banner without disturbing the
    /// currently-loaded library or its engine.
    fn set_banner(&self, message: String);
}

// Production impl — green agent fills these in.
pub struct MainSideEffects {
    // Green agent: hold Arc<Mutex<Option<EngineLink>>>, Arc<Mutex<ViewModel>>,
    // Arc<Mutex<Option<(SelectionHandler, ActivationHandler)>>>,
    // and slint::Weak<MainWindow>. See temper file Q2 resolution.
}

impl UiSideEffects for MainSideEffects {
    fn apply_library(&self, _path: PathBuf, _tracks: Vec<Track>) {
        todo!("green agent implements")
    }

    fn set_banner(&self, _message: String) {
        todo!("green agent implements")
    }
}
