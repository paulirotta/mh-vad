use portaudio as pa;
use sample::{signal, Frame, Sample, Signal, ToFrameSliceMut};

const CHANNELS: i32 = 1;
const SAMPLE_RATE: f64 = 16_000.0;
const FRAME: u32 = 512;

pub fn play_synth() -> Result<(), pa::Error> {
    // Create a signal chain to play back 1 second of each oscillator at A4.
    let hz = signal::rate(SAMPLE_RATE).const_hz(440.0);
    let one_sec = SAMPLE_RATE as usize;
    let mut waves = hz
        .clone()
        .sine()
        .take(one_sec)
        .chain(hz.clone().saw().take(one_sec))
        .chain(hz.clone().square().take(one_sec))
        .chain(hz.clone().noise_simplex().take(one_sec))
        .chain(signal::noise(0).take(one_sec))
        .map(|f| f.map(|s| s.to_sample::<f32>() * 0.5));

    // Initialise PortAudio.
    let pa = portaudio::PortAudio::new()?;
    let settings = pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_RATE, FRAME)?;

    // Initialize VAD
    //    let time_domain: Vec<f32> = vec![0.0; FRAMES as usize];

    let vad = ::mh_vad::Vad::new(FRAME as usize);

    // Define the callback which provides PortAudio the audio.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        let buff: &mut [[f32; 1]] = buffer.to_frame_slice_mut().unwrap();
        for out_frame in buff {
            match waves.next() {
                Some(frame) => *out_frame = frame,
                None => return pa::Complete,
            }
        }

        let vad_frame = ::mh_vad::VadFrame::new(&buffer, &vad);

        println!("{}", vad_frame);
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;
    stream.start()?;

    while let Ok(true) = stream.is_active() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    stream.stop()?;
    stream.close()?;

    Ok(())
}
