//! Tests for selection vs. activation wiring.
//!
//! These tests construct the closures from `wire_handlers` directly,
//! bypassing the Slint event loop. The fake `CommandSink` records every
//! `Command` it receives so tests can assert exact playback intent.

use bangrs_audio::{Command, HandleError};
use bangrs_core::TrackId;
use bangrs_ui::handlers::{wire_handlers, ActivationHandler, CommandSink, SelectionHandler};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct FakeSink {
    commands: Mutex<Vec<Command>>,
}

impl CommandSink for FakeSink {
    fn send(&self, cmd: Command) -> Result<(), HandleError> {
        self.commands.lock().unwrap().push(cmd);
        Ok(())
    }
}

impl FakeSink {
    fn commands(&self) -> Vec<Command> {
        self.commands.lock().unwrap().clone()
    }
    fn play_count(&self) -> usize {
        self.commands()
            .iter()
            .filter(|c| matches!(c, Command::Play(_)))
            .count()
    }
}

fn fake_setup(
    track_ids: Vec<TrackId>,
) -> (Arc<FakeSink>, SelectionHandler, ActivationHandler) {
    let fake = Arc::new(FakeSink::default());
    let sink: Arc<dyn CommandSink> = fake.clone();
    let track_ids_arc = Arc::new(track_ids);
    let (sel, act) = wire_handlers(sink, track_ids_arc);
    (fake, sel, act)
}

#[test]
fn selection_does_not_play() {
    let (fake, sel, _act) = fake_setup(vec![TrackId(0), TrackId(1), TrackId(2)]);

    sel(0);
    sel(1);
    sel(2);

    assert_eq!(
        fake.play_count(),
        0,
        "selection events must not emit Command::Play"
    );
}

#[test]
fn bind_does_not_play() {
    // Simulates current-item-changed firing on set_tracks rebind.
    let (fake, sel, _act) = fake_setup(vec![TrackId(10), TrackId(11)]);

    sel(0);

    assert_eq!(
        fake.play_count(),
        0,
        "current-item-changed firing on rebind must not emit Command::Play"
    );
}

#[test]
fn activation_plays_selected() {
    let (fake, _sel, act) = fake_setup(vec![TrackId(100), TrackId(200), TrackId(300)]);

    act(1);

    assert_eq!(
        fake.commands(),
        vec![Command::Play(TrackId(200))],
        "activation must emit exactly Command::Play with the selected track"
    );
}

#[test]
fn activation_with_negative_idx_is_noop() {
    let (fake, _sel, act) = fake_setup(vec![TrackId(0)]);

    act(-1);

    assert_eq!(
        fake.play_count(),
        0,
        "negative index (no selection) must not emit Command::Play"
    );
}

#[test]
fn activation_with_oob_idx_is_noop() {
    let (fake, _sel, act) = fake_setup(vec![TrackId(0)]);

    act(5);

    assert_eq!(
        fake.play_count(),
        0,
        "out-of-bounds index (rebind race) must not emit Command::Play"
    );
}
