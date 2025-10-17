use anyhow::Result;
use hound::WavReader;
use image::{ImageBuffer, Rgb};
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;

use algo::RealSignalExt;
mod algo;

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
    let bin = |freq: f32| (freq * 64.0 / sample_rate as f32) as usize;

    let mut img = ImageBuffer::new(500, 300);
    let mut row = Vec::new();
    let mut y = 0;

    let mut idx = 0;
    while idx + FFT_SIZE <= samples.len() {
        let chunk = &samples[idx..(idx + FFT_SIZE)];
        idx += 4;

        let mut samples = (chunk.iter().copied().hamming().to_complex()).collect::<Vec<_>>();
        fft.process(&mut samples);

        // found horizontal sync pulse
        if samples[bin(1200.0)].abs() >= 5.0 {
            if !row.is_empty() {
                let third = row.len() / 3;
                for i in 0..third {
                    let color = Rgb([row[2 * third + i], row[i], row[third + i]]);
                    img.put_pixel(i as u32, y, color);
                }
                row.clear();
                y += 1;
            }
            continue;
        }

        // values range from 1,500 and 2,300 Hz
        let (bl, bu) = (bin(1500.0), bin(2300.0));
        let w_avg = weighted_index(&samples[bl..=bu]);
        row.push((w_avg * 255.0) as u8);
    }

    img.save("out.png")?;

    Ok(())
}

fn weighted_index(samples: &[Complex<f32>]) -> f32 {
    let mut w_avg = 0.0;
    let mut sum = 0.0;

    for (i, sample) in samples.iter().enumerate() {
        let mag = sample.abs();
        w_avg += i as f32 * mag;
        sum += mag;
    }

    w_avg / sum / samples.len() as f32
}
