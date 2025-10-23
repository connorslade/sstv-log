use tokio::sync::broadcast::Sender;

use crate::{
    dsp::{
        extentions::RealExt,
        filters::{LowPassFilter, MovingAverageFilter},
    },
    sstv::{
        image::{Image, ImageBuilder},
        pulse::{HEADER_PULSE, PulseDetector, SYNC_PULSE},
    },
};

const VALUE_RANGE: (f32, f32) = (1500.0, 2300.0);
const ABORT_TIMEOUT: f32 = 3.0;

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
    End(Image),
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
        Self {
            state: DecoderState::idle(sample_rate),
            sample_rate,
            sample: 0,

            f_avg: MovingAverageFilter::new(32),
            f_low_pass: LowPassFilter::new(VALUE_RANGE.1, sample_rate as f32),

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
                    let value = (freq - VALUE_RANGE.0) / (VALUE_RANGE.1 - VALUE_RANGE.0);

                    // todo: test repeating last sample vs saturating
                    if value.abs() > 1.0 {
                        row.push(row.last().copied().unwrap_or_default());
                    } else {
                        row.push(value.saturate());
                    }
                    return;
                }

                *last_sync = self.sample;
                if row.len() > (0.2 * self.sample_rate as f32) as usize {
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
        DecoderState::Decoding {
            sync: PulseDetector::new(SYNC_PULSE, sample_rate),
            last_sync: sample,

            img: ImageBuilder::new(sample_rate, 320, 256),
            row: Vec::new(),
        }
    }
}
