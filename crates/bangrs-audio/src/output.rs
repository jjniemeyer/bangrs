use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::error::AudioError;
use crate::negotiate::{ConfigRange, SampleFormat as NegFormat};

pub trait Output: Send {
    fn write(&mut self, samples: &[f32]);
}

pub struct CpalOutput {
    ring: Arc<Mutex<VecDeque<f32>>>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl CpalOutput {
    /// Open the default output device. Returns the output handle and the
    /// underlying cpal `Stream`, which the caller must keep alive for the
    /// duration of playback (stream lives on the audio thread; not `Send`).
    pub fn new() -> Result<(Self, Stream), AudioError> {
        Self::open(None)
    }

    /// Open the default output device with a specific sample rate, overriding
    /// the device default. Format and channels come from `default_output_config`.
    pub fn with_sample_rate(rate_hz: u32) -> Result<(Self, Stream), AudioError> {
        Self::open(Some(rate_hz))
    }

    fn open(rate_override_hz: Option<u32>) -> Result<(Self, Stream), AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| AudioError::Output("no default output device".into()))?;
        let supported = device
            .default_output_config()
            .map_err(|e| AudioError::Output(e.to_string()))?;
        let sample_format = supported.sample_format();
        let channels = supported.channels();
        let mut config: cpal::StreamConfig = supported.into();
        if let Some(rate) = rate_override_hz {
            config.sample_rate = cpal::SampleRate(rate);
        }
        let sample_rate = config.sample_rate.0;

        let ring: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(1 << 16)));
        let err_fn = |e| tracing::error!("cpal stream error: {e}");

        let stream = match sample_format {
            SampleFormat::F32 => {
                let ring = ring.clone();
                device.build_output_stream(
                    &config,
                    move |data: &mut [f32], _| drain_into(&ring, data),
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                let ring = ring.clone();
                device.build_output_stream(
                    &config,
                    move |data: &mut [i16], _| drain_into_convert(&ring, data, f32_to_i16),
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                let ring = ring.clone();
                device.build_output_stream(
                    &config,
                    move |data: &mut [u16], _| drain_into_convert(&ring, data, f32_to_u16),
                    err_fn,
                    None,
                )
            }
            other => {
                return Err(AudioError::UnsupportedFormat(format!(
                    "sample format {other:?}"
                )))
            }
        }
        .map_err(|e| AudioError::Output(e.to_string()))?;

        stream.play().map_err(|e| AudioError::Output(e.to_string()))?;

        Ok((
            Self {
                ring,
                sample_rate,
                channels,
            },
            stream,
        ))
    }
}

/// Query the default output device's supported configs and translate them to
/// `ConfigRange`. Sample formats outside `SampleFormat::{I16, F32}` are dropped.
pub fn collect_supported_configs() -> Result<Vec<ConfigRange>, AudioError> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| AudioError::Output("no default output device".into()))?;
    let ranges = device
        .supported_output_configs()
        .map_err(|e| AudioError::Output(e.to_string()))?;
    let collected = ranges
        .filter_map(|range| {
            let fmt = match range.sample_format() {
                SampleFormat::I16 => Some(NegFormat::I16),
                SampleFormat::F32 => Some(NegFormat::F32),
                _ => None,
            }?;
            Some(ConfigRange {
                channels: range.channels(),
                sample_format: fmt,
                min_rate_hz: range.min_sample_rate().0,
                max_rate_hz: range.max_sample_rate().0,
            })
        })
        .collect();
    Ok(collected)
}

fn drain_into(ring: &Arc<Mutex<VecDeque<f32>>>, out: &mut [f32]) {
    let mut guard = ring.lock().expect("ring poisoned");
    for slot in out.iter_mut() {
        *slot = guard.pop_front().unwrap_or(0.0);
    }
}

fn drain_into_convert<T: Copy>(
    ring: &Arc<Mutex<VecDeque<f32>>>,
    out: &mut [T],
    conv: fn(f32) -> T,
) {
    let mut guard = ring.lock().expect("ring poisoned");
    for slot in out.iter_mut() {
        *slot = conv(guard.pop_front().unwrap_or(0.0));
    }
}

fn f32_to_i16(s: f32) -> i16 {
    (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

fn f32_to_u16(s: f32) -> u16 {
    ((s.clamp(-1.0, 1.0) * 0.5 + 0.5) * u16::MAX as f32) as u16
}

impl CpalOutput {
    pub fn buffered_samples(&self) -> usize {
        self.ring.lock().expect("ring poisoned").len()
    }
    pub fn clear(&mut self) {
        self.ring.lock().expect("ring poisoned").clear();
    }
}

impl Output for CpalOutput {
    fn write(&mut self, samples: &[f32]) {
        let mut guard = self.ring.lock().expect("ring poisoned");
        guard.extend(samples.iter().copied());
    }
}

#[derive(Default)]
pub struct FakeOutput {
    pub buffers: Vec<Vec<f32>>,
}

impl Output for FakeOutput {
    fn write(&mut self, samples: &[f32]) {
        self.buffers.push(samples.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_output_records_writes() {
        let mut out = FakeOutput::default();
        out.write(&[0.1, 0.2, 0.3]);
        out.write(&[0.4]);
        assert_eq!(out.buffers.len(), 2);
        assert_eq!(out.buffers[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(out.buffers[1], vec![0.4]);
    }
}
