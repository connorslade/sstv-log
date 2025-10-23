use std::f32::consts::TAU;

use ringbuf::{HeapRb, traits::RingBuffer};

pub struct MovingAverageFilter {
    buffer: HeapRb<f32>,
    size: usize,
    sum: f32,
}

impl MovingAverageFilter {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: HeapRb::new(size),
            size,
            sum: 0.0,
        }
    }

    pub fn update(&mut self, value: f32) -> f32 {
        self.sum += value;
        self.sum -= self.buffer.push_overwrite(value).unwrap_or_default();

        self.sum / self.size as f32
    }
}

pub struct LowPassFilter {
    prev_output: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(cutoff: f32, sample_rate: f32) -> Self {
        let rc = (cutoff * TAU).recip();
        let dt = sample_rate.recip();
        let alpha = dt / (rc + dt);

        Self {
            prev_output: 0.0,
            alpha,
        }
    }

    pub fn update(&mut self, value: f32) -> f32 {
        self.prev_output += self.alpha * (value - self.prev_output);
        self.prev_output
    }
}
