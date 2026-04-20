use bangrs_audio::{Command, HandleError};
use bangrs_core::TrackId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

#[derive(Clone)]
pub struct PlayerHandle {
    tx: mpsc::SyncSender<Command>,
    position: Arc<AtomicU64>,
}

fn send(tx: &mpsc::SyncSender<Command>, cmd: Command) -> Result<(), HandleError> {
    tx.try_send(cmd).map_err(|e| match e {
        mpsc::TrySendError::Full(_) => HandleError::QueueFull,
        mpsc::TrySendError::Disconnected(_) => HandleError::ThreadClosed,
    })
}

impl PlayerHandle {
    pub fn new(tx: mpsc::SyncSender<Command>, position: Arc<AtomicU64>) -> Self {
        Self { tx, position }
    }

    pub fn play(&self, track_id: TrackId) -> Result<(), HandleError> {
        send(&self.tx, Command::Play(track_id))
    }
    pub fn pause(&self) -> Result<(), HandleError> {
        send(&self.tx, Command::Pause)
    }
    pub fn resume(&self) -> Result<(), HandleError> {
        send(&self.tx, Command::Resume)
    }
    pub fn stop(&self) -> Result<(), HandleError> {
        send(&self.tx, Command::Stop)
    }
    pub fn seek(&self, position: Duration) -> Result<(), HandleError> {
        send(&self.tx, Command::Seek(position))
    }
    pub fn position(&self) -> Duration {
        Duration::from_millis(self.position.load(Ordering::Relaxed))
    }
    pub fn shutdown(self) -> Result<(), HandleError> {
        send(&self.tx, Command::Shutdown)
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
