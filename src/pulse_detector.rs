use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, RingBuffer},
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
