use std::f32::consts::PI;

use num_complex::{Complex, ComplexFloat};

pub trait RealSignalExt {
    fn hamming(self) -> impl Iterator<Item = f32>;
    fn to_complex(self) -> impl Iterator<Item = Complex<f32>>;
}

pub trait RealExt {
    fn saturate(self) -> Self;
}

pub fn peak_freq(spectrum: &[Complex<f32>], sample_rate: u32) -> f32 {
    let mags = spectrum.iter().map(|c| c.abs()).collect::<Vec<_>>();
    let (max_index_offset, _) = (mags[1..spectrum.len() / 2].iter().enumerate())
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let x = max_index_offset + 1;

    let y1 = mags[x - 1];
    let y2 = mags[x];
    let y3 = mags[x + 1];

    let delta = 0.5 * (y1 - y3) / (y1 - 2.0 * y2 + y3);
    ((x as f32 + delta) * sample_rate as f32) / spectrum.len() as f32
}

impl<T: Iterator<Item = f32>> RealSignalExt for T {
    fn hamming(self) -> impl Iterator<Item = f32> {
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
