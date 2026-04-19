pub trait Output: Send {
    fn write(&mut self, samples: &[f32]);
}

pub struct CpalOutput {
    // Populated by green agent. Drains a ring buffer into the cpal stream and
    // converts f32 → device format (i16 / f32 etc) at the boundary.
}

impl Output for CpalOutput {
    fn write(&mut self, _samples: &[f32]) {
        todo!("green: write to ring buffer")
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
