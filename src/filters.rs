use std::f32::consts::TAU;

pub struct MovingAverageFilter {
    buffer: Vec<f32>,
    index: usize,
    size: usize,
    sum: f32,
}

impl MovingAverageFilter {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            size,
            sum: 0.0,
        }
    }

    pub fn update(&mut self, value: f32) -> f32 {
        self.sum -= self.buffer[self.index];
        self.sum += value;

        self.buffer[self.index] = value;
        self.index = (self.index + 1) % self.size;

        self.sum / self.size as f32
    }
}

pub struct LowPassFilter {
    prev_output: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(cutoff: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (TAU * cutoff);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);
        Self {
            prev_output: 0.0,
            alpha,
        }
    }

    pub fn update(&mut self, input: f32) -> f32 {
        self.prev_output += self.alpha * (input - self.prev_output);
        self.prev_output
    }
}
