use bangrs_audio::{Command, HandleError};
use bangrs_core::TrackId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

pub struct PlayerHandle {
    tx: mpsc::SyncSender<Command>,
    position: Arc<AtomicU64>,
}

impl PlayerHandle {
    pub fn new(tx: mpsc::SyncSender<Command>, position: Arc<AtomicU64>) -> Self {
        Self { tx, position }
    }

    pub fn play(&self, _track_id: TrackId) -> Result<(), HandleError> {
        todo!("green: try_send Play")
    }
    pub fn pause(&self) -> Result<(), HandleError> {
        todo!("green: try_send Pause")
    }
    pub fn resume(&self) -> Result<(), HandleError> {
        todo!("green: try_send Resume")
    }
    pub fn stop(&self) -> Result<(), HandleError> {
        todo!("green: try_send Stop")
    }
    pub fn seek(&self, _position: Duration) -> Result<(), HandleError> {
        todo!("green: try_send Seek, coalesce")
    }
    pub fn position(&self) -> Duration {
        Duration::from_millis(self.position.load(Ordering::Relaxed))
    }
    pub fn shutdown(self) -> Result<(), HandleError> {
        todo!("green: try_send Shutdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_handle() -> (PlayerHandle, mpsc::Receiver<Command>) {
        let (tx, rx) = mpsc::sync_channel(32);
        let pos = Arc::new(AtomicU64::new(0));
        (PlayerHandle::new(tx, pos), rx)
    }

    #[test]
    fn play_sends_play_command() {
        let (h, rx) = fresh_handle();
        h.play(TrackId(5)).unwrap();
        assert_eq!(rx.try_recv().unwrap(), Command::Play(TrackId(5)));
    }

    #[test]
    fn pause_sends_pause_command() {
        let (h, rx) = fresh_handle();
        h.pause().unwrap();
        assert_eq!(rx.try_recv().unwrap(), Command::Pause);
    }

    #[test]
    fn position_reads_atomic() {
        let (tx, _rx) = mpsc::sync_channel(32);
        let pos = Arc::new(AtomicU64::new(12_345));
        let h = PlayerHandle::new(tx, pos);
        assert_eq!(h.position(), Duration::from_millis(12_345));
    }

    #[test]
    fn queue_full_returns_error() {
        let (tx, _rx) = mpsc::sync_channel(1);
        let pos = Arc::new(AtomicU64::new(0));
        let h = PlayerHandle::new(tx, pos);
        h.pause().unwrap();
        // Second send fills the queue; third must fail with QueueFull
        assert_eq!(h.pause().err(), Some(HandleError::QueueFull));
    }
}
