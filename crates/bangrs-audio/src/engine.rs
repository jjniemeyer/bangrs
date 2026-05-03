use crate::command::Command;
use crate::negotiate::{negotiate_config, SampleFormat};
use crate::output::{collect_supported_configs, CpalOutput, Output};
use bangrs_core::{Event, Library, Track, TrackId};
use crossbeam_channel::Sender;
use std::fs::File;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

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

struct ActiveTrack {
    id: TrackId,
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    frames_played: u64,
}

fn open_track(track: &Track) -> Result<ActiveTrack, String> {
    let file = File::open(track.path.as_std_path()).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = track.path.extension() {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions { enable_gapless: true, ..Default::default() },
            &MetadataOptions::default(),
        )
        .map_err(|e| e.to_string())?;
    let format = probed.format;
    let selected = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "no decodable track".to_string())?;
    let track_id = selected.id;
    let sample_rate = selected.codec_params.sample_rate.unwrap_or(0);
    let decoder = symphonia::default::get_codecs()
        .make(&selected.codec_params, &Default::default())
        .map_err(|e| e.to_string())?;
    Ok(ActiveTrack {
        id: track.id,
        format,
        decoder,
        track_id,
        sample_rate,
        frames_played: 0,
    })
}

enum DecodeStep {
    Wrote,
    EndOfStream,
    Failed(String),
}

fn decode_one(active: &mut ActiveTrack, output: &mut CpalOutput) -> DecodeStep {
    let packet = loop {
        match active.format.next_packet() {
            Ok(p) if p.track_id() == active.track_id => break p,
            Ok(_) => continue,
            Err(SymphoniaError::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                return DecodeStep::EndOfStream;
            }
            Err(SymphoniaError::ResetRequired) => return DecodeStep::EndOfStream,
            Err(e) => return DecodeStep::Failed(e.to_string()),
        }
    };

    let decoded = match active.decoder.decode(&packet) {
        Ok(d) => d,
        Err(SymphoniaError::DecodeError(msg)) => {
            tracing::warn!("decode error (skipping packet): {msg}");
            return DecodeStep::Wrote;
        }
        Err(e) => return DecodeStep::Failed(e.to_string()),
    };
    let spec = *decoded.spec();
    let frames = decoded.frames();
    if frames == 0 {
        return DecodeStep::Wrote;
    }
    let mut sample_buf = SampleBuffer::<f32>::new(frames as u64, spec);
    sample_buf.copy_interleaved_ref(decoded);

    let samples = sample_buf.samples();
    let out_channels = output.channels as usize;
    let src_channels = spec.channels.count();

    if src_channels == out_channels {
        output.write(samples);
    } else if src_channels == 1 && out_channels >= 1 {
        let mut up = Vec::with_capacity(frames * out_channels);
        for &s in samples {
            for _ in 0..out_channels {
                up.push(s);
            }
        }
        output.write(&up);
    } else if src_channels == 2 && out_channels == 1 {
        let mut down = Vec::with_capacity(frames);
        for f in samples.chunks_exact(2) {
            down.push((f[0] + f[1]) * 0.5);
        }
        output.write(&down);
    } else {
        let mut mixed = Vec::with_capacity(frames * out_channels);
        for chunk in samples.chunks(src_channels) {
            let mono: f32 = chunk.iter().sum::<f32>() / src_channels as f32;
            for _ in 0..out_channels {
                mixed.push(mono);
            }
        }
        output.write(&mixed);
    }

    active.frames_played += frames as u64;
    DecodeStep::Wrote
}

impl AudioEngine for CpalEngine {
    fn run(
        self,
        library: Arc<Library>,
        rx: mpsc::Receiver<Command>,
        tx: Sender<Event>,
        position: Arc<AtomicU64>,
    ) {
        let (mut output, mut _stream) = match CpalOutput::new() {
            Ok(pair) => pair,
            Err(e) => {
                let _ = tx.send(Event::FatalError(e.to_string()));
                return;
            }
        };
        let mut device_rate = output.sample_rate;

        let mut active: Option<ActiveTrack> = None;
        let mut paused = false;
        let mut last_position_emit = Instant::now();

        loop {
            let poll_timeout = if active.is_some() && !paused {
                Duration::from_millis(5)
            } else {
                Duration::from_millis(50)
            };
            match rx.recv_timeout(poll_timeout) {
                Ok(Command::Shutdown) => break,
                Ok(Command::Play(id)) => {
                    output.clear();
                    position.store(0, Ordering::Relaxed);
                    match library.get(id) {
                        Some(track) => match open_track(track) {
                            Ok(mut at) => {
                                let supported = match collect_supported_configs() {
                                    Ok(s) => s,
                                    Err(e) => {
                                        let _ = tx.send(Event::TrackFailed {
                                            track_id: id,
                                            reason: format!("device error: {e}"),
                                        });
                                        active = None;
                                        continue;
                                    }
                                };
                                match negotiate_config(
                                    &supported,
                                    at.sample_rate,
                                    SampleFormat::F32,
                                    2,
                                ) {
                                    Err(nmc) => {
                                        let _ = tx.send(Event::TrackFailed {
                                            track_id: id,
                                            reason: nmc.to_string(),
                                        });
                                        active = None;
                                    }
                                    Ok(chosen) => {
                                        if chosen.rate_hz != device_rate {
                                            drop(_stream);
                                            drop(output);
                                            match CpalOutput::with_sample_rate(chosen.rate_hz) {
                                                Ok((new_output, new_stream)) => {
                                                    output = new_output;
                                                    _stream = new_stream;
                                                    device_rate = output.sample_rate;
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(Event::FatalError(
                                                        e.to_string(),
                                                    ));
                                                    return;
                                                }
                                            }
                                        }
                                        at.frames_played = 0;
                                        active = Some(at);
                                        paused = false;
                                        let _ = tx.send(Event::PlaybackStarted { track_id: id });
                                    }
                                }
                            }
                            Err(reason) => {
                                let _ = tx.send(Event::TrackFailed { track_id: id, reason });
                                active = None;
                            }
                        },
                        None => {
                            let _ = tx.send(Event::TrackFailed {
                                track_id: id,
                                reason: "track not in library".into(),
                            });
                        }
                    }
                }
                Ok(Command::Pause) => {
                    if active.is_some() && !paused {
                        paused = true;
                        let _ = tx.send(Event::PlaybackPaused);
                    }
                }
                Ok(Command::Resume) => {
                    if active.is_some() && paused {
                        paused = false;
                        let _ = tx.send(Event::PlaybackResumed);
                    }
                }
                Ok(Command::Stop) => {
                    if active.take().is_some() {
                        output.clear();
                        paused = false;
                        position.store(0, Ordering::Relaxed);
                        let _ = tx.send(Event::PlaybackStopped);
                    }
                }
                Ok(Command::Seek(_)) => {
                    // TODO: symphonia seek — no-op for now.
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if let Some(at) = active.as_mut()
                && !paused
            {
                let target_buffered =
                    (at.sample_rate as usize) * (output.channels as usize) / 4; // ~250 ms
                while output.buffered_samples() < target_buffered {
                    match decode_one(at, &mut output) {
                        DecodeStep::Wrote => {}
                        DecodeStep::EndOfStream => {
                            let _ = tx.send(Event::TrackEnded { track_id: at.id });
                            active = None;
                            break;
                        }
                        DecodeStep::Failed(reason) => {
                            let _ = tx.send(Event::TrackFailed {
                                track_id: at.id,
                                reason,
                            });
                            active = None;
                            break;
                        }
                    }
                }

                if let Some(at) = active.as_ref() {
                    let ms = at.frames_played.saturating_mul(1000)
                        / at.sample_rate.max(1) as u64;
                    position.store(ms, Ordering::Relaxed);
                    if last_position_emit.elapsed() >= Duration::from_millis(200) {
                        let _ = tx.send(Event::PositionUpdate { ms });
                        last_position_emit = Instant::now();
                    }
                }
            }
        }
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
        rx: mpsc::Receiver<Command>,
        tx: Sender<Event>,
        _position: Arc<AtomicU64>,
    ) {
        loop {
            match rx.recv() {
                Ok(Command::Shutdown) | Err(_) => break,
                Ok(_) => {
                    for ev in self.scripted_events.iter() {
                        tx.send(ev.clone()).ok();
                    }
                }
            }
        }
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
