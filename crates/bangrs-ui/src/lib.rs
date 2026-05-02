pub mod dispatcher;
pub mod handle;
pub mod handlers;
pub mod pick_handler;
pub mod pick_state;
pub mod picker;
pub mod side_effects;

pub use handle::PlayerHandle;
pub use pick_handler::on_pick_library;
pub use pick_state::PickState;

slint::include_modules!();

use bangrs_core::{TrackId, ViewModel};

pub fn bind(window: &MainWindow, vm: &ViewModel) {
    window.set_is_playing(vm.is_playing);
    window.set_is_paused(vm.is_paused);
    window.set_current_track_id(vm.current_track.map(|TrackId(id)| id as i32).unwrap_or(-1));
    window.set_position_ms(vm.position_ms as i32);
    window.set_error_banner(vm.error_banner.clone().unwrap_or_default().into());
    let rows: Vec<slint::StandardListViewItem> = vm
        .tracks
        .iter()
        .map(|t| {
            let label = if t.artist.is_empty() {
                t.title.clone()
            } else {
                format!("{} — {}", t.artist, t.title)
            };
            slint::StandardListViewItem::from(slint::SharedString::from(label))
        })
        .collect();
    window.set_tracks(slint::ModelRc::new(slint::VecModel::from(rows)));
}
