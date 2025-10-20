use std::f32::consts::TAU;

use anyhow::Result;
use hound::WavReader;
use num_complex::Complex;
use rustfft::FftPlanner;

use crate::{algo::hilbert_transform, sstv::SstvDecoder};
mod algo;
mod pulse_detector;
mod sstv;

const FFT_SIZE: usize = 1 << 13;

fn main() -> Result<()> {
    let audio = WavReader::open("input2-int.wav")?;
    let sample_rate = audio.spec().sample_rate;

    let samples = audio
        .into_samples::<i16>()
        .map(|x| x.unwrap() as f32 / i16::MAX as f32)
        .collect::<Vec<_>>();

    let mut planner = FftPlanner::new();
    let mut decoder = SstvDecoder::new(sample_rate);

    let mut last = Complex::new(1.0, 0.0);
    for chunk in samples.chunks(FFT_SIZE) {
        let signal = hilbert_transform(&mut planner, chunk);
        for next in signal {
            if last == Complex::ZERO {
                decoder.freq(0.0);
            } else {
                let freq = (next / last).arg() * sample_rate as f32 / TAU;
                decoder.freq(freq);
            }
            last = next;
        }
    }

    Ok(())
}
