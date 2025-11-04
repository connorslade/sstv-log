#![feature(array_windows)]

use std::{collections::VecDeque, f32::consts::TAU};

use anyhow::Result;
use cpal::{
    SampleRate,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use num_complex::Complex;
use rustfft::FftPlanner;
use tokio::{runtime::Runtime, sync::broadcast};

use crate::{
    dsp::{LowPassFilter, hilbert_transform},
    sstv::decode::{SstvDecoder, SstvEvent},
    web::web_server,
};
mod dsp;
mod sstv;
mod web;

const FFT_SIZE: usize = 1 << 13;
const SAMPLE_RATE: SampleRate = SampleRate(44_100);
const MAX_FREQ: f32 = 2300.0;

fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let mut configs = device.supported_input_configs()?;
    let config = configs.next().unwrap().with_sample_rate(SAMPLE_RATE);
    let sample_rate = config.sample_rate().0;

    let (tx, rx) = broadcast::channel::<SstvEvent>(128);

    let mut planner = FftPlanner::new();
    let mut decoder = SstvDecoder::new(sample_rate, tx);

    let mut buffer = VecDeque::new();
    let mut low_pass = LowPassFilter::new(MAX_FREQ, sample_rate);

    let stream = device.build_input_stream(
        &config.into(),
        move |chunk: &[f32], _info| {
            buffer.extend(chunk);
            while buffer.len() > FFT_SIZE {
                let chunk = buffer.drain(..FFT_SIZE).map(|x| low_pass.update(x));
                let signal = hilbert_transform(&mut planner, chunk);

                for [prev, next] in signal.array_windows() {
                    let freq = if *prev == Complex::ZERO {
                        0.0
                    } else {
                        (next / prev).arg() * sample_rate as f32 / TAU
                    };

                    decoder.freq(freq);
                }
            }
        },
        |err| eprintln!("Error: {err}"),
        None,
    )?;
    stream.play()?;

    Runtime::new()?.block_on(web_server(rx))?;

    Ok(())
}
