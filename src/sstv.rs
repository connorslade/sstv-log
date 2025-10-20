use image::{ImageBuffer, Rgb};

use crate::{
    algo::{RealExt, lerp},
    pulse_detector::{PulseDetector, PulseDetectorConfig},
};

pub struct SstvDecoder {
    state: DecoderState,
    sample_rate: u32,
}

struct ImageBuilder {
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    y: u32,
}

enum DecoderState {
    Idle {
        header: PulseDetector,
    },
    Decoding {
        sync: PulseDetector,
        img: ImageBuilder,
        row: Vec<f32>,
    },
}

const HEADER_PULSE: PulseDetectorConfig = PulseDetectorConfig {
    freq: 1900.0,
    range: 50.0,

    threshold: 0.9,
    duration: 0.6,
};

const SYNC_PULSE: PulseDetectorConfig = PulseDetectorConfig {
    freq: 1200.0,
    range: 50.0,

    threshold: 0.45,
    duration: 0.002,
};

impl SstvDecoder {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            state: DecoderState::Idle {
                header: PulseDetector::new(HEADER_PULSE, sample_rate),
            },
            sample_rate,
        }
    }

    pub fn freq(&mut self, freq: f32) {
        match &mut self.state {
            DecoderState::Idle { header } => {
                if !header.update(freq) {
                    return;
                }

                self.state = DecoderState::Decoding {
                    sync: PulseDetector::new(SYNC_PULSE, self.sample_rate),
                    img: ImageBuilder {
                        img: ImageBuffer::new(320, 256),
                        y: 0,
                    },
                    row: Vec::new(),
                };
            }
            DecoderState::Decoding { sync, img, row } => {
                if sync.update(freq) {
                    img.push_row(row);
                    return;
                }

                let value = (freq - 1500.0) / (2300.0 - 1500.0);
                row.push(value.saturate());
            }
        }
    }
}

impl ImageBuilder {
    pub fn push_row(&mut self, row: &mut Vec<f32>) {
        if !row.is_empty() && self.y < self.img.height() {
            let get = |x: f32| {
                let idx = row.len() as f32 * x;

                let prev = row[idx as usize];
                let next = row[(idx.ceil() as usize).min(row.len() - 1)];
                (lerp(prev, next, idx.fract()) * 255.0) as u8
            };

            let width = self.img.width();
            for x in 0..width {
                let t = x as f32 / width as f32 / 3.0;
                let color = Rgb([get(t + 2. / 3.), get(t), get(t + 1. / 3.)]);
                self.img.put_pixel(x, self.y, color);
            }

            self.y += 1;
            row.clear();
        }
    }
}

impl Drop for DecoderState {
    fn drop(&mut self) {
        match self {
            DecoderState::Idle { .. } => {}
            DecoderState::Decoding { img, .. } => {
                img.img.save("out.png").unwrap();
            }
        }
    }
}
