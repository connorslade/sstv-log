use std::mem;

use crossbeam_channel::Sender;
use image::{ImageBuffer, Rgb};

use crate::{
    algo::{RealExt, lerp},
    pulse::{PulseDetector, PulseDetectorConfig},
};

pub type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;

pub struct SstvDecoder {
    state: DecoderState,
    sample_rate: u32,

    tx: Sender<Image>,
}

struct ImageBuilder {
    img: Image,
    sample_rate: u32,
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
    range: 100.0,

    threshold: 0.45,
    duration: 0.6,
};

const SYNC_PULSE: PulseDetectorConfig = PulseDetectorConfig {
    freq: 1200.0,
    range: 200.0,

    threshold: 0.45,
    duration: 0.002,
};

impl SstvDecoder {
    pub fn new(sample_rate: u32, tx: Sender<Image>) -> Self {
        Self {
            state: DecoderState::idle(sample_rate),
            sample_rate,
            tx,
        }
    }

    pub fn freq(&mut self, freq: f32) {
        match &mut self.state {
            DecoderState::Idle { header } => {
                if !header.update(freq) {
                    return;
                }

                println!("starting decode");
                self.state = DecoderState::decoding(self.sample_rate);
            }
            DecoderState::Decoding { sync, img, row } => {
                if !sync.update(freq) {
                    let value = (freq - 1500.0) / (2300.0 - 1500.0);
                    row.push(value.saturate());
                    return;
                }

                if row.len() > (0.2 * self.sample_rate as f32) as usize {
                    img.push_row(row);
                    row.clear();

                    if img.finished() {
                        println!("decoded image");
                        self.tx.send(mem::take(&mut img.img)).unwrap();
                        self.state = DecoderState::idle(self.sample_rate);
                    }
                }
            }
        }
    }
}

impl DecoderState {
    pub fn idle(sample_rate: u32) -> Self {
        Self::Idle {
            header: PulseDetector::new(HEADER_PULSE, sample_rate),
        }
    }

    pub fn decoding(sample_rate: u32) -> Self {
        DecoderState::Decoding {
            sync: PulseDetector::new(SYNC_PULSE, sample_rate),
            img: ImageBuilder {
                img: ImageBuffer::new(320, 256),
                sample_rate,
                y: 0,
            },
            row: Vec::new(),
        }
    }
}

impl ImageBuilder {
    pub fn push_row(&mut self, row: &[f32]) {
        let samples_per_row = 0.48 * self.sample_rate as f32;
        let rows = (row.len() as f32 / samples_per_row).round() as usize;
        if rows == 0 {
            return;
        }

        for row in row.chunks(row.len() / rows) {
            if self.y >= self.img.height() {
                return;
            }

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
                // let color = Rgb([get(t), get(t), get(t)]);
                self.img.put_pixel(x, self.y, color);
            }

            self.y += 1;
        }
    }

    pub fn finished(&self) -> bool {
        self.y >= self.img.height()
    }
}
