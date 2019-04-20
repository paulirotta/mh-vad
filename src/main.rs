// Perform a forward FFT of size 1234


use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::{FFTplanner, FFT};
use std::sync::Arc;

pub fn main() {
    const WINDOW: usize = 1024;

    let time_domain: Vec<f32> = vec![0.0; WINDOW];

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(WINDOW);

    let vad = VadFrame::new(&time_domain, &fft);

    println!("vad silence: {:?}", vad);
}


#[derive(Debug)]
pub struct VadFrame {
    pub energy: f32,
    pub dominant_freq: f32,
    pub spectral_flatness_measurement: f32,
}

impl<'a> VadFrame {
    pub fn new(time_domain: &'a [f32], fft: &Arc<FFT<f32>>) -> VadFrame {
        const WINDOW: usize = 1024;
        const SAMPLE_RATE: f32 = 16_000.0;

        assert!(time_domain.len() == WINDOW);

        let mut input: Vec<Complex<f32>> = Vec::with_capacity(WINDOW);
        let mut output: Vec<Complex<f32>> = vec![Complex::zero(); WINDOW];

        for i in 0..WINDOW {
            input.push(Complex::new(time_domain[i], 0.0));
        }

        let energy = short_term_energy(time_domain);
        fft.process(&mut input, &mut output);
        let dominant_freq = bin_to_freq(peak_bin(&output), WINDOW, SAMPLE_RATE);
        let spectral_flatness_measurement = measure_spectral_flatness(&output);

        VadFrame {
            energy: energy,
            dominant_freq: dominant_freq,
            spectral_flatness_measurement: spectral_flatness_measurement,
        }
    }
}

fn short_term_energy(time_domain: &[f32]) -> f32 {
    let mut sum = 0.0;

    for val in time_domain.iter() {
        let v = val.abs();
        sum = sum + v * v;
    }

    sum / time_domain.len() as f32
}

fn peak_bin(frame: &[Complex<f32>]) -> usize {
    let mut max_val = std::f32::MIN;
    let mut max_bin = 0;

    for (bin, val) in frame.iter().enumerate() {
        if val.re > max_val {
            max_val = val.re;
            max_bin = bin;
        }
    }

    max_bin
}

fn bin_to_freq(bin: usize, window_size: usize, sample_rate: f32) -> f32 {
    (bin as f32 / window_size as f32) * sample_rate
}

fn geometric_mean(vec: &[Complex<f32>]) -> f32 {
    let mut mean: f32 = 1.0;

    for v in vec {
        mean = mean * v.re;
    }

    mean.powf(1.0 / vec.len() as f32)
}

fn arithmetic_mean(vec: &[Complex<f32>]) -> f32 {
    let mut mean: f32 = 0.0;

    for v in vec {
        mean = mean + v.re;
    }

    mean / vec.len() as f32
}

fn measure_spectral_flatness(fft: &[Complex<f32>]) -> f32 {
    10.0 * (geometric_mean(fft) / arithmetic_mean(fft)).log(10.0)
}

