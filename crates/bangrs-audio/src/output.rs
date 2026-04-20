use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::error::AudioError;

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
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| AudioError::Output("no default output device".into()))?;
        let supported = device
            .default_output_config()
            .map_err(|e| AudioError::Output(e.to_string()))?;
        let sample_format = supported.sample_format();
        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels();
        let config: cpal::StreamConfig = supported.into();

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

impl Output for CpalOutput {
    fn write(&mut self, samples: &[f32]) {
        let mut guard = self.ring.lock().expect("ring poisoned");
        guard.extend(samples.iter().copied());
    }
}

pub struct FakeOutput {
    pub buffers: Vec<Vec<f32>>,
}

impl Default for FakeOutput {
    fn default() -> Self {
        Self { buffers: Vec::new() }
    }
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
