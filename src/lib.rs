use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FFT;

use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub struct VadFrame {
    pub energy: f32,
    pub dominant_freq: f32,
    pub spectral_flatness_measurement: f32,
}

impl fmt::Display for VadFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Energy:{}, Dominant Freq:{}, SFM:{})",
            self.energy, self.dominant_freq, self.spectral_flatness_measurement
        )
    }
}

impl<'a> VadFrame {
    pub fn new(time_domain: &'a [f32], fft: &Arc<FFT<f32>>) -> VadFrame {
        const SAMPLE_RATE: f32 = 16_000.0;

        assert!(
            time_domain.len() == fft.len(),
            "Time domain frame length must be the same size as FFT frame length"
        );

        let mut input: Vec<Complex<f32>> = Vec::new();
        let mut output: Vec<Complex<f32>> = vec![Complex::zero(); time_domain.len()];

        for i in 0..time_domain.len() {
            input.push(Complex::new(time_domain[i], 0.0));
        }

        let energy = short_term_energy(time_domain);
        fft.process(&mut input, &mut output);
        let dominant_freq = bin_to_freq(peak_bin(&output), time_domain.len(), SAMPLE_RATE);
        let spectral_flatness_measurement = spectral_flatness(&output);

        VadFrame {
            energy: energy,
            dominant_freq: dominant_freq,
            spectral_flatness_measurement: spectral_flatness_measurement,
        }
    }
}

fn short_term_energy(time_domain: &[f32]) -> f32 {
    let mut sum: f32 = 0.0;

    for val in time_domain.iter() {
        let v = val.abs();
        sum = sum + v * v;
    }

    sum / (time_domain.len() as f32)
}

fn peak_bin(frame: &[Complex<f32>]) -> usize {
    let mut max_val = frame[0].re;
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

    mean.powf(1.0 / (vec.len() as f32))
}

fn arithmetic_mean(vec: &[Complex<f32>]) -> f32 {
    let mut mean: f32 = 0.0;

    for v in vec {
        mean = mean + v.re;
    }

    mean / (vec.len() as f32)
}

pub fn spectral_flatness(fft: &[Complex<f32>]) -> f32 {
    10.0 * (geometric_mean(fft) / arithmetic_mean(fft)).log(10.0)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use rustfft::num_complex::Complex;

    #[test]
    fn test_arithmetic_mean() {
        let mut a = Vec::new();
        a.push(Complex::new(1.0, 0.0));
        a.push(Complex::new(0.0, 1.0));

        let a_mean = 0.5;
        let a_prime = arithmetic_mean(&a);

        assert_eq!(a_prime, a_mean);
    }

    #[test]
    fn test_arithmetic_mean2() {
        let mut a = Vec::new();
        a.push(Complex::new(1.0, 0.0));
        a.push(Complex::new(2.0, 1.0));
        a.push(Complex::new(3.0, -3.0));

        let a_mean = 2.0;
        let a_prime = arithmetic_mean(&a);

        assert_eq!(a_prime, a_mean);
    }

    #[test]
    fn test_geometric_mean() {
        let mut a = Vec::new();
        a.push(Complex::new(1.0, 0.0));
        a.push(Complex::new(3.0, 1.0));
        a.push(Complex::new(9.0, -3.0));
        a.push(Complex::new(27.0, -3.0));
        a.push(Complex::new(81.0, 4.0));

        let g_mean = 9.0;
        let a_prime = geometric_mean(&a);

        assert_eq!(a_prime, g_mean);
    }
}
