// Perform a forward FFT of size 1234

use rustfft::FFTplanner;

mod lib;
mod synth;

const CHANNELS: i32 = 1;
const SAMPLE_RATE: f64 = 16_000.0;
const FRAME: u32 = 512;

pub fn main() {
    const WINDOW: usize = 1024;

    let time_domain: Vec<f32> = vec![0.0; WINDOW];

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(WINDOW);

    let vad = lib::VadFrame::new(&time_domain, &fft);

    println!("vad silence: {:?}", vad);

    let _result = synth::play_synth();
}
