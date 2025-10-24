use axum::body::Bytes;
use tokio::sync::broadcast::Sender;

use crate::{
    dsp::{
        extentions::RealExt,
        filters::{LowPassFilter, MovingAverageFilter},
    },
    sstv::{
        image::ImageBuilder,
        pulse::{HEADER_PULSE, PulseDetector, SYNC_PULSE},
    },
};

const VALUE_RANGE: (f32, f32) = (1500.0, 2300.0);
const ABORT_TIMEOUT: f32 = 3.0;
const MIN_ROW_DURATION: f32 = 0.2; // ← verify this
const IMAGE_DIMENTIONS: (u32, u32) = (320, 256);

pub struct SstvDecoder {
    state: DecoderState,
    sample_rate: u32,
    sample: u64,

    f_avg: MovingAverageFilter,
    f_low_pass: LowPassFilter,

    tx: Sender<SstvEvent>,
}

#[derive(Debug, Clone)]
pub enum SstvEvent {
    Start,
    Progress(f32),
    End(Bytes),
}

enum DecoderState {
    Idle {
        header: PulseDetector,
    },
    Decoding {
        sync: PulseDetector,
        last_sync: u64,

        img: ImageBuilder,
        row: Vec<f32>,
    },
}

impl SstvDecoder {
    pub fn new(sample_rate: u32, tx: Sender<SstvEvent>) -> Self {
        let (_, max_freq) = VALUE_RANGE;

        Self {
            state: DecoderState::idle(sample_rate),
            sample_rate,
            sample: 0,

            f_avg: MovingAverageFilter::new(32),
            f_low_pass: LowPassFilter::new(max_freq, sample_rate as f32),

            tx,
        }
    }

    pub fn freq(&mut self, freq: f32) {
        let freq = self.f_avg.update(self.f_low_pass.update(freq));
        self.sample += 1;

        match &mut self.state {
            DecoderState::Idle { header } => {
                if !header.update(freq) {
                    return;
                }

                self.tx.send(SstvEvent::Start).unwrap();
                self.state = DecoderState::decoding(self.sample_rate, self.sample);
            }
            DecoderState::Decoding {
                sync,
                last_sync,
                img,
                row,
            } => {
                if (self.sample - *last_sync) as f32 > ABORT_TIMEOUT * self.sample_rate as f32 {
                    self.tx.send(SstvEvent::End(img.finish())).unwrap();
                    self.state = DecoderState::idle(self.sample_rate);
                    return;
                }

                if !sync.update(freq) {
                    let (min, max) = VALUE_RANGE;
                    let value = (freq - min) / (max - min);

                    // todo: test repeating last sample vs saturating
                    if value.abs() > 1.0 {
                        row.push(row.last().copied().unwrap_or_default());
                    } else {
                        row.push(value.saturate());
                    }
                    return;
                }

                *last_sync = self.sample;

                let min_row_samples = (MIN_ROW_DURATION * self.sample_rate as f32) as usize;
                if row.len() > min_row_samples {
                    self.tx.send(SstvEvent::Progress(img.progress())).unwrap();
                    img.push_row(row);
                    row.clear();

                    if img.finished() {
                        self.tx.send(SstvEvent::End(img.finish())).unwrap();
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

    pub fn decoding(sample_rate: u32, sample: u64) -> Self {
        let (width, height) = IMAGE_DIMENTIONS;

        DecoderState::Decoding {
            sync: PulseDetector::new(SYNC_PULSE, sample_rate),
            last_sync: sample,

            img: ImageBuilder::new(sample_rate, width, height),
            row: Vec::new(),
        }
    }
}
