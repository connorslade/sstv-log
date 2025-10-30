use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, RingBuffer},
};

pub const HEADER_PULSE: PulseDetectorConfig = PulseDetectorConfig {
    freq: 1900.0,
    range: 100.0,

    threshold: 0.45,
    duration: 0.6,
};

pub const VIS_STOP_PULSE: PulseDetectorConfig = PulseDetectorConfig {
    freq: 1200.0,
    range: 50.0,
    threshold: 0.50,
    duration: 0.03,
};

pub const SYNC_PULSE: PulseDetectorConfig = PulseDetectorConfig {
    freq: 1200.0,
    range: 100.0,

    threshold: 0.45,
    duration: 0.004,
};

pub struct PulseDetectorConfig {
    pub freq: f32,
    pub range: f32,

    pub threshold: f32,
    pub duration: f32,
}

pub struct PulseDetector {
    buffer: HeapRb<bool>,
    config: PulseDetectorConfig,
}

impl PulseDetector {
    pub fn new(config: PulseDetectorConfig, sample_rate: u32) -> Self {
        let samples = (config.duration * sample_rate as f32) as usize;
        Self {
            buffer: HeapRb::new(samples),
            config,
        }
    }

    pub fn update(&mut self, freq: f32) -> bool {
        self.buffer
            .push_overwrite((freq - self.config.freq).abs() < self.config.range);

        let active = self.buffer.iter().filter(|&&x| x).count();
        let fraction = active as f32 / self.buffer.occupied_len() as f32;

        fraction >= self.config.threshold
    }
}
