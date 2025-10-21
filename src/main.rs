#![feature(array_windows)]

use std::{collections::VecDeque, f32::consts::TAU};

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use num_complex::Complex;
use rustfft::FftPlanner;
use tokio::{runtime::Runtime, sync::broadcast};

use crate::{
    dsp::hilbert_transform,
    sstv::decode::{SstvDecoder, SstvEvent},
    web::web_server,
};
mod dsp;
mod sstv;
mod web;

const FFT_SIZE: usize = 1 << 13;

fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let mut configs = device.supported_input_configs().unwrap();
    let config = configs.next().unwrap().with_max_sample_rate();
    let sample_rate = config.sample_rate().0;

    let (tx, rx) = broadcast::channel::<SstvEvent>(16);

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

                    for [prev, next] in signal.array_windows() {
                        if *prev == Complex::ZERO {
                            continue;
                        }

                        let freq = (next / prev).arg() * sample_rate as f32 / TAU;
                        decoder.freq(freq);
                    }
                }
            },
            |err| println!("Error: {err}"),
            None,
        )
        .unwrap();
    stream.play().unwrap();

    Runtime::new()?.block_on(web_server(rx))?;

    Ok(())
}
