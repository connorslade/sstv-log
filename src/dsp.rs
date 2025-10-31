use std::f32::consts::TAU;

use num_complex::Complex;
use rustfft::FftPlanner;

use crate::FFT_SIZE;

pub struct LowPassFilter {
    prev_output: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(cutoff: f32, sample_rate: u32) -> Self {
        let rc = (cutoff * TAU).recip();
        let dt = (sample_rate as f32).recip();
        let alpha = dt / (rc + dt);

        Self {
            prev_output: 0.0,
            alpha,
        }
    }

    pub fn update(&mut self, value: f32) -> f32 {
        self.prev_output += self.alpha * (value - self.prev_output);
        self.prev_output
    }
}

pub fn hilbert_transform(
    planner: &mut FftPlanner<f32>,
    real: impl Iterator<Item = f32>,
) -> Vec<Complex<f32>> {
    let (fft, ifft) = (
        planner.plan_fft_forward(FFT_SIZE),
        planner.plan_fft_inverse(FFT_SIZE),
    );

    let mut hilbert = real.map(|x| Complex::new(x, 0.0)).collect::<Vec<_>>();
    fft.process(&mut hilbert);

    let n = hilbert.len();
    for (i, sample) in hilbert.iter_mut().enumerate() {
        if i > 0 && i < n / 2 {
            *sample *= 2.0;
        } else if !(i == 0 || (n % 2 == 0 && i == n / 2)) {
            *sample *= 0.0;
        }
    }

    ifft.process(&mut hilbert);

    hilbert
}
