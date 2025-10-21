use std::mem;

use crossbeam_channel::Sender;

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

pub struct SstvDecoder {
    state: DecoderState,
    sample_rate: u32,

    f_avg: MovingAverageFilter,
    f_low_pass: LowPassFilter,

    tx: Sender<Image>,
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

impl SstvDecoder {
    pub fn new(sample_rate: u32, tx: Sender<Image>) -> Self {
        Self {
            state: DecoderState::idle(sample_rate),
            sample_rate,

            f_avg: MovingAverageFilter::new(32),
            f_low_pass: LowPassFilter::new(2300.0, sample_rate as f32),

            tx,
        }
    }

    pub fn freq(&mut self, freq: f32) {
        let freq = self.f_avg.update(self.f_low_pass.update(freq));

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
                    println!("Row {}/255", img.y);
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
            img: ImageBuilder::new(sample_rate, 320, 256),
            row: Vec::new(),
        }
    }
}
