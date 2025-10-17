use std::f32::consts::PI;

use num_complex::Complex;

pub trait RealSignalExt {
    fn hamming(self) -> impl Iterator<Item = f32>;
    fn to_complex(self) -> impl Iterator<Item = Complex<f32>>;
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
