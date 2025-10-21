use std::{collections::VecDeque, f32::consts::TAU};

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use num_complex::Complex;
use rustfft::FftPlanner;

use crate::{
    algo::hilbert_transform,
    sstv::{Image, SstvDecoder},
};
mod algo;
mod filters;
mod pulse;
mod sstv;

const FFT_SIZE: usize = 1 << 13;

fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let mut configs = device.supported_input_configs().unwrap();
    let config = configs.next().unwrap().with_max_sample_rate();
    let sample_rate = config.sample_rate().0;

    let (tx, rx) = crossbeam_channel::unbounded::<Image>();

    let mut planner = FftPlanner::new();
    let mut decoder = SstvDecoder::new(sample_rate, tx);

    let mut buffer = VecDeque::new();

    let stream = device
        .build_input_stream(
            &config.into(),
            move |chunk: &[f32], _info| {
                buffer.extend(chunk);
                while buffer.len() > FFT_SIZE {
                    let chunk = buffer.drain(..FFT_SIZE);
                    let signal = hilbert_transform(&mut planner, chunk);

                    for pair in signal.windows(2) {
                        if pair[1] == Complex::ZERO {
                            continue;
                        }

                        let freq = (pair[1] / pair[0]).arg() * sample_rate as f32 / TAU;
                        decoder.freq(freq);
                    }
                }
            },
            |err| println!("Error: {err}"),
            None,
        )
        .unwrap();
    stream.play().unwrap();

    for img in rx.iter() {
        img.save("out.png").unwrap();
    }

    Ok(())
}
