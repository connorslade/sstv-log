use std::f32::consts::PI;

use num_complex::Complex;
use rustfft::FftPlanner;

pub trait RealSignalExt {
    fn hann(self) -> impl Iterator<Item = f32>;
    fn to_complex(self) -> impl Iterator<Item = Complex<f32>>;
}

pub trait RealExt {
    fn saturate(self) -> Self;
}

pub fn hilbert_transform(planner: &mut FftPlanner<f32>, real: &[f32]) -> Vec<Complex<f32>> {
    let (fft, ifft) = (
        planner.plan_fft_forward(real.len()),
        planner.plan_fft_inverse(real.len()),
    );

    let mut hilbert = (real.iter().copied().hann().to_complex()).collect::<Vec<_>>();
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

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl<T: Iterator<Item = f32>> RealSignalExt for T {
    fn hann(self) -> impl Iterator<Item = f32> {
        let len = self.size_hint().0;
        self.enumerate().map(move |(i, x)| {
            let window = 0.5 - 0.5 * (2.0 * PI * i as f32 / (len as f32)).cos();
            x * window
        })
    }

    fn to_complex(self) -> impl Iterator<Item = Complex<f32>> {
        self.map(|x| Complex::new(x, 0.0))
    }
}

impl RealExt for f32 {
    fn saturate(self) -> Self {
        self.clamp(0.0, 1.0)
    }
}
