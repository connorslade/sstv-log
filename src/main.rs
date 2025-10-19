use anyhow::Result;
use hound::WavReader;
use rustfft::FftPlanner;

use algo::RealSignalExt;

use crate::{algo::peak_freq, sstv::SstvDecoder};
mod algo;
mod sstv;

const FFT_SIZE: usize = 64;

fn main() -> Result<()> {
    let audio = WavReader::open("input.wav")?;
    let sample_rate = audio.spec().sample_rate;

    let samples = audio
        .into_samples::<i16>()
        .map(|x| x.unwrap() as f32 / i16::MAX as f32)
        .collect::<Vec<_>>();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let mut decoder = SstvDecoder::new(sample_rate);

    let mut idx = 0;
    while idx + FFT_SIZE <= samples.len() {
        let chunk = &samples[idx..(idx + FFT_SIZE)];
        idx += 4;

        let mut samples = (chunk.iter().copied().hamming().to_complex()).collect::<Vec<_>>();
        fft.process(&mut samples);

        let freq = peak_freq(&samples, sample_rate);
        decoder.freq(freq);
    }

    Ok(())
}
