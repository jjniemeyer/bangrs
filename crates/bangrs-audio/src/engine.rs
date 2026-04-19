use crate::command::Command;
use bangrs_core::{Event, Library};
use crossbeam_channel::Sender;
use std::sync::mpsc;
use std::sync::{atomic::AtomicU64, Arc};

pub trait AudioEngine: Send {
    fn run(
        self,
        library: Arc<Library>,
        rx: mpsc::Receiver<Command>,
        tx: Sender<Event>,
        position: Arc<AtomicU64>,
    );
}

pub struct CpalEngine;
impl AudioEngine for CpalEngine {
    fn run(
        self,
        _library: Arc<Library>,
        _rx: mpsc::Receiver<Command>,
        _tx: Sender<Event>,
        _position: Arc<AtomicU64>,
    ) {
        todo!("green: full cpal + symphonia pipeline")
    }
}

/// Records all commands received and echoes scripted events.
/// Used in tests to drive deterministic event sequences.
pub struct FakeEngine {
    pub scripted_events: Vec<Event>,
}

impl AudioEngine for FakeEngine {
    fn run(
        self,
        _library: Arc<Library>,
        _rx: mpsc::Receiver<Command>,
        _tx: Sender<Event>,
        _position: Arc<AtomicU64>,
    ) {
        todo!("green: drain rx, emit self.scripted_events, then return on Command::Shutdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bangrs_core::{Library, TrackId};

    #[test]
    fn fake_engine_forwards_scripted_events_on_play() {
        let (cmd_tx, cmd_rx) = mpsc::sync_channel(32);
        let (ev_tx, ev_rx) = crossbeam_channel::bounded(64);
        let position = Arc::new(AtomicU64::new(0));
        let lib = Arc::new(Library::new(vec![]));

        let engine = FakeEngine {
            scripted_events: vec![Event::PlaybackStarted { track_id: TrackId(1) }],
        };

        let h = std::thread::spawn(move || engine.run(lib, cmd_rx, ev_tx, position));
        cmd_tx.send(Command::Play(TrackId(1))).unwrap();
        cmd_tx.send(Command::Shutdown).unwrap();
        h.join().unwrap();

        let received: Vec<_> = ev_rx.try_iter().collect();
        assert!(received.contains(&Event::PlaybackStarted { track_id: TrackId(1) }));
    }
}
