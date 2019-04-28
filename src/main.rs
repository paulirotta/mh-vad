// Perform a forward FFT of size 1234

mod lib;
mod synth;

pub fn main() {
    const WINDOW: usize = 1024;

    let time_domain: Vec<f32> = vec![0.0; WINDOW];
    let vad = lib::Vad::new(WINDOW);
    let vad_frame = lib::VadFrame::new(&time_domain, &vad);

    println!("{:?}", vad_frame);

    let _result = synth::play_synth();
}
