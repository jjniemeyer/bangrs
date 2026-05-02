//! Worker-thread orchestration for the Pick Library button.
//!
//! `on_pick_library` is the entry point: it spawns a worker thread that
//! invokes the picker, runs the scanner, and dispatches a UI-thread closure
//! that either installs the library (success) or surfaces an error (failure).
//! The supersession check via `PickState` ensures that a slow scan whose
//! result arrives after a newer pick has been issued is dropped silently.

use crate::dispatcher::Dispatcher;
use crate::pick_state::PickState;
use crate::picker::FolderPicker;
use crate::side_effects::UiSideEffects;
use bangrs_scan::Scanner;
use std::sync::Arc;

/// Spawn a worker thread to run the pick + scan + dispatch pipeline.
///
/// Returns immediately. The dispatched closure runs on the UI thread (per the
/// `Dispatcher` contract) and either calls `side_effects.apply_library(...)`
/// on success or `side_effects.set_banner(...)` on failure. Superseded picks
/// drop silently with no side effect.
pub fn on_pick_library(
    picker: Arc<dyn FolderPicker>,
    scanner: Arc<dyn Scanner + Send + Sync>,
    dispatcher: Arc<dyn Dispatcher>,
    pick_state: Arc<PickState>,
    side_effects: Arc<dyn UiSideEffects>,
) {
    let my_seq = pick_state.next() + 1;

    std::thread::spawn(move || {
        let Some(path) = picker.pick_folder() else {
            return;
        };

        let (oks, errs): (Vec<_>, Vec<_>) = scanner.scan(&path).partition(Result::is_ok);
        let tracks: Vec<_> = oks.into_iter().map(Result::unwrap).collect();
        let first_err = errs.into_iter().filter_map(Result::err).next();

        let pick_state_for_closure = pick_state.clone();
        let side_effects_for_closure = side_effects.clone();
        let path_for_closure = path.clone();

        dispatcher.dispatch(Box::new(move || {
            if pick_state_for_closure.current() != my_seq {
                return;
            }
            match first_err {
                Some(e) => {
                    side_effects_for_closure.set_banner(format!("Library scan failed: {e}"));
                }
                None => {
                    side_effects_for_closure.apply_library(path_for_closure, tracks);
                }
            }
        }));
    });
}
