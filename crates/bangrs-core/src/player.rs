use crate::error::CoreError;
use crate::library::Track;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Player {
    Stopped,
    Loaded { track: Track },
    Playing { track: Track, position: Duration },
    Paused { track: Track, position: Duration },
}

impl Default for Player {
    fn default() -> Self {
        Player::Stopped
    }
}

impl Player {
    pub fn new() -> Self {
        Player::Stopped
    }

    pub fn name(&self) -> &'static str {
        match self {
            Player::Stopped => "Stopped",
            Player::Loaded { .. } => "Loaded",
            Player::Playing { .. } => "Playing",
            Player::Paused { .. } => "Paused",
        }
    }

    pub fn load(self, track: Track) -> Result<Self, CoreError> {
        match self {
            Player::Stopped => Ok(Player::Loaded { track }),
            other => Err(CoreError::InvalidTransition { from: other.name(), to: "Loaded" }),
        }
    }
    pub fn play(self) -> Result<Self, CoreError> {
        match self {
            Player::Loaded { track } => Ok(Player::Playing { track, position: Duration::ZERO }),
            Player::Paused { track, position } => Ok(Player::Playing { track, position }),
            other => Err(CoreError::InvalidTransition { from: other.name(), to: "Playing" }),
        }
    }
    pub fn pause(self) -> Result<Self, CoreError> {
        match self {
            Player::Playing { track, position } => Ok(Player::Paused { track, position }),
            other => Err(CoreError::InvalidTransition { from: other.name(), to: "Paused" }),
        }
    }
    pub fn stop(self) -> Self {
        Player::Stopped
    }
    pub fn advance(self, delta: Duration) -> Result<Self, CoreError> {
        match self {
            Player::Playing { track, position } => {
                Ok(Player::Playing { track, position: position + delta })
            }
            other => Err(CoreError::InvalidTransition { from: other.name(), to: "Playing" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    fn sample_track() -> Track {
        Track {
            id: crate::library::TrackId(1),
            path: Utf8PathBuf::from("/tmp/1.wav"),
            title: "t".into(),
            artist: None,
            album: None,
            duration: Duration::from_secs(5),
            sample_rate: 44100,
            channels: 2,
            replay_gain_db: None,
        }
    }

    #[test]
    fn new_player_is_stopped() {
        assert_eq!(Player::new(), Player::Stopped);
    }

    #[test]
    fn stopped_loads_to_loaded() {
        let t = sample_track();
        let p = Player::Stopped.load(t.clone()).unwrap();
        assert_eq!(p, Player::Loaded { track: t });
    }

    #[test]
    fn stopped_cannot_play() {
        assert_eq!(
            Player::Stopped.play(),
            Err(CoreError::InvalidTransition { from: "Stopped", to: "Playing" })
        );
    }

    #[test]
    fn loaded_plays_from_position_zero() {
        let t = sample_track();
        let p = Player::Loaded { track: t.clone() }.play().unwrap();
        assert_eq!(p, Player::Playing { track: t, position: Duration::ZERO });
    }

    #[test]
    fn paused_resumes_at_saved_position() {
        let t = sample_track();
        let pos = Duration::from_secs(3);
        let p = Player::Paused { track: t.clone(), position: pos }.play().unwrap();
        assert_eq!(p, Player::Playing { track: t, position: pos });
    }

    #[test]
    fn playing_pauses_preserving_position() {
        let t = sample_track();
        let pos = Duration::from_secs(2);
        let p = Player::Playing { track: t.clone(), position: pos }.pause().unwrap();
        assert_eq!(p, Player::Paused { track: t, position: pos });
    }

    #[test]
    fn stop_always_returns_stopped() {
        let t = sample_track();
        assert_eq!(Player::Stopped.stop(), Player::Stopped);
        assert_eq!(Player::Loaded { track: t.clone() }.stop(), Player::Stopped);
        assert_eq!(
            Player::Playing { track: t.clone(), position: Duration::ZERO }.stop(),
            Player::Stopped
        );
        assert_eq!(
            Player::Paused { track: t, position: Duration::ZERO }.stop(),
            Player::Stopped
        );
    }

    #[test]
    fn advance_on_playing_moves_position() {
        let t = sample_track();
        let p = Player::Playing { track: t.clone(), position: Duration::from_secs(1) }
            .advance(Duration::from_secs(2))
            .unwrap();
        assert_eq!(p, Player::Playing { track: t, position: Duration::from_secs(3) });
    }

    #[test]
    fn advance_on_stopped_is_invalid() {
        assert_eq!(
            Player::Stopped.advance(Duration::from_secs(1)),
            Err(CoreError::InvalidTransition { from: "Stopped", to: "Playing" })
        );
    }
}
