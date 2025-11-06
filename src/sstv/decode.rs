use std::fs;

use axum::body::Bytes;
use tokio::sync::broadcast::Sender;

use crate::sstv::{
    image::ImageBuilder,
    modes::SstvMode,
    pulse::{HEADER_PULSE, PulseDetector, SYNC_PULSE, VIS_STOP_PULSE},
};

const VIS_BITS: (f32, f32) = (1300.0, 1100.0);
const VALUE_RANGE: (f32, f32) = (1500.0, 2300.0);
const ABORT_TIMEOUT: f32 = 3.0;
const IMAGE_DIMENTIONS: (u32, u32) = (320, 256);

pub struct SstvDecoder {
    state: DecoderState,
    sample_rate: u32,
    sample: u64,

    tx: Sender<SstvEvent>,
    debug: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum SstvEvent {
    Start(SstvMode),
    Progress(f32),
    End(SstvMode, Bytes),
}

enum DecoderState {
    Idle {
        header: PulseDetector,
    },
    Vis {
        stop: PulseDetector,
        bits: Vec<bool>,
    },
    Decoding {
        sync: PulseDetector,
        last_sync: u64,
        mode: SstvMode,

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

            tx,
            debug: Vec::new(),
        }
    }

    pub fn freq(&mut self, freq: f32) {
        self.sample += 1;

        self.debug.push(freq);
        if self.sample == 10 * self.sample_rate as u64 {
            println!("sample rate: {} btw", self.sample_rate);
            let contents = self
                .debug
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            fs::write("debug-noavg.csv", contents).unwrap();
        }

        match &mut self.state {
            DecoderState::Idle { header } => {
                if !header.update(freq) {
                    return;
                }

                self.state = DecoderState::vis(self.sample_rate);
            }
            DecoderState::Vis { stop, bits } => {
                let (zero_freq, one_freq) = VIS_BITS;
                let d0 = (freq - zero_freq).abs();
                let d1 = (freq - one_freq).abs();
                if d0.min(d1) < 50.0 {
                    bits.push(d1 < d0);
                }

                let vis_samples = (0.03 * 7.0 * self.sample_rate as f32) as usize;
                let bit_samples = vis_samples / 7;

                if stop.update(freq) && bits.len() > vis_samples / 2 {
                    let mut value = 0_u8;
                    for chunk in bits[..vis_samples.min(bits.len())].chunks(bit_samples) {
                        let p = chunk.iter().map(|&x| x as u32).sum::<u32>() as f32
                            / chunk.len() as f32;

                        let bit = (p > 0.5) as u8;
                        value = value >> 1 | (bit << 6);
                    }

                    // todo: figure out parity bit...
                    let vis = SstvMode::from_vis(value);

                    self.tx.send(SstvEvent::Start(vis)).unwrap();
                    self.state = DecoderState::decoding(vis, self.sample_rate, self.sample);
                }
            }
            DecoderState::Decoding {
                sync,
                last_sync,
                mode,
                img,
                row,
            } => {
                if (self.sample - *last_sync) as f32 > ABORT_TIMEOUT * self.sample_rate as f32 {
                    self.tx.send(SstvEvent::End(*mode, img.finish())).unwrap();
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
                        row.push(value.clamp(0.0, 1.0));
                    }
                    return;
                }

                *last_sync = self.sample;

                let min_row_samples = (0.1 * self.sample_rate as f32) as usize;
                if row.len() > min_row_samples {
                    self.tx.send(SstvEvent::Progress(img.progress())).unwrap();
                    img.push_row(row);
                    row.clear();

                    if img.finished() {
                        self.tx.send(SstvEvent::End(*mode, img.finish())).unwrap();
                        self.state = DecoderState::idle(self.sample_rate);
                    }
                }
            }
        }
    }
}

impl DecoderState {
    fn idle(sample_rate: u32) -> Self {
        Self::Idle {
            header: PulseDetector::new(HEADER_PULSE, sample_rate),
        }
    }

    fn vis(sample_rate: u32) -> Self {
        Self::Vis {
            stop: PulseDetector::new(VIS_STOP_PULSE, sample_rate),
            bits: Vec::new(),
        }
    }

    fn decoding(mode: SstvMode, sample_rate: u32, sample: u64) -> Self {
        let (width, height) = IMAGE_DIMENTIONS;

        DecoderState::Decoding {
            sync: PulseDetector::new(SYNC_PULSE, sample_rate),
            last_sync: sample,
            mode,

            img: ImageBuilder::new(sample_rate, width, height),
            row: Vec::new(),
        }
    }
}
