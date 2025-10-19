use image::{ImageBuffer, Rgb};

use crate::algo::RealExt;

pub struct SstvDecoder {
    state: DecoderState,
    sample_rate: u32,
}

enum DecoderState {
    Idle {
        header: Vec<bool>,
    },
    Decoding {
        img: ImageBuffer<Rgb<u8>, Vec<u8>>,
        row: Vec<f32>,
        y: u32,

        tmp: Vec<bool>,
    },
}

impl Drop for DecoderState {
    fn drop(&mut self) {
        match self {
            DecoderState::Idle { .. } => {}
            DecoderState::Decoding { img, .. } => {
                img.save("out.png").unwrap();
            }
        }
    }
}

impl SstvDecoder {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            state: DecoderState::Idle { header: Vec::new() },
            sample_rate,
        }
    }

    pub fn freq(&mut self, freq: f32) {
        match &mut self.state {
            DecoderState::Idle { header } => {
                // header_duration: 600ms 90% on
                let samples = (600 * self.sample_rate / 1000) as usize;

                header.push((freq - 1900.0) < 50.0);
                while header.len() > samples {
                    header.remove(0);
                }

                if header.len() < samples {
                    return;
                }

                let active = header.iter().filter(|&&x| x).count();
                let fraction = active as f32 / samples as f32;

                if fraction >= 0.9 {
                    self.state = DecoderState::Decoding {
                        img: ImageBuffer::new(320, 300),
                        row: Vec::new(),
                        y: 0,
                        tmp: Vec::new(),
                    };
                }
            }
            DecoderState::Decoding { img, row, y, tmp } => {
                //  horizontal sync pulse
                tmp.push((freq - 1200.0).abs() < 50.0);
                let samples = 90;
                while tmp.len() > samples {
                    tmp.remove(0);
                }

                let active = tmp.iter().filter(|&&x| x).count();
                let fraction = active as f32 / samples as f32;

                if fraction >= 0.9 {
                    if !row.is_empty() && *y < img.height() {
                        let get = |x: f32| {
                            let idx = row.len() as f32 * x;

                            let prev = row[idx as usize];
                            let next = row[(idx.ceil() as usize).min(row.len() - 1)];
                            (lerp(prev, next, idx.fract()) * 255.0) as u8
                        };

                        for x in 0..img.width() {
                            let t = x as f32 / img.width() as f32 / 3.0;
                            let color = Rgb([get(t + 2. / 3.), get(t), get(t + 1. / 3.)]);
                            // let color = Rgb([get(t), get(t), get(t)]);
                            img.put_pixel(x, *y, color);
                        }

                        row.clear();
                        *y += 1;
                    }

                    return;
                }

                let value = (freq - 1500.0) / (2300.0 - 1500.0);
                row.push(value.saturate());
            }
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
