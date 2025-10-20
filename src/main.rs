use std::{f32::consts::TAU, thread};

use anyhow::Result;
use hound::WavReader;
use num_complex::Complex;
use rustfft::FftPlanner;

use crate::{
    algo::hilbert_transform,
    filters::{LowPassFilter, MovingAverageFilter},
    sstv::{Image, SstvDecoder},
};
mod algo;
mod filters;
mod pulse;
mod sstv;

const FFT_SIZE: usize = 1 << 13;

fn main() -> Result<()> {
    let audio = WavReader::open("input2-noise-int.wav")?;
    let sample_rate = audio.spec().sample_rate;

    let samples = audio
        .into_samples::<i16>()
        .map(|x| x.unwrap() as f32 / i16::MAX as f32)
        .collect::<Vec<_>>();

    let (tx, rx) = crossbeam_channel::unbounded::<Image>();
    thread::spawn(move || {
        for img in rx.iter() {
            img.save("out.png").unwrap();
        }
    });

    let mut planner = FftPlanner::new();
    let mut decoder = SstvDecoder::new(sample_rate, tx);

    let mut avg = MovingAverageFilter::new(32);
    let mut low_pass = LowPassFilter::new(2300.0, sample_rate as f32);

    let mut last = Complex::ZERO;
    for chunk in samples.chunks(FFT_SIZE) {
        let signal = hilbert_transform(&mut planner, chunk);
        for next in signal {
            if last == Complex::ZERO {
                decoder.freq(0.0);
            } else {
                let freq = (next / last).arg() * sample_rate as f32 / TAU;
                decoder.freq(avg.update(low_pass.update(freq)));
            }
            last = next;
        }
    }

    Ok(())
}
