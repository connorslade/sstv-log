use num_complex::Complex;
use rustfft::FftPlanner;

use crate::{FFT_SIZE, dsp::extentions::RealSignalExt};

pub mod extentions;
pub mod filters;

pub fn hilbert_transform(
    planner: &mut FftPlanner<f32>,
    real: impl Iterator<Item = f32>,
) -> Vec<Complex<f32>> {
    let (fft, ifft) = (
        planner.plan_fft_forward(FFT_SIZE),
        planner.plan_fft_inverse(FFT_SIZE),
    );

    let mut hilbert = (real.hann().to_complex()).collect::<Vec<_>>();
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
