use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat, FromSample};
use std::f32::consts::PI;
use rand::Rng;

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let mut supported_configs_range = device.supported_output_configs()
        .expect("No Supported config");
    let supported_config = supported_configs_range.next()
        .expect("no supported config")
        .with_max_sample_rate();
    let config = supported_config.config();
    let sample_rate = config.sample_rate.0 as f32;
    let mut sample_clock = 0f32;
    let mut frequency = 220.0; // Frequency in Hz (A4 note)

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [u8], _: &cpal::OutputCallbackInfo| {
            // react to stream events and read or write stream data here.
            for sample in data.iter_mut() {
                let value = sample_clock.sin();
                *sample = Sample::from_sample(value);
                if sample_clock % 0.02 == 0.0 {
                    frequency = random_piano_frequency();
                    println!("{frequency}")
                }
                sample_clock = (sample_clock + (frequency * 2.0 * PI / sample_rate)) % (2.0 * PI);
            }
        },
        move |err| {
            // react to errors here.
            println!("an error occurred on the output audio stream: {}", err);

        },
        None // None=blocking, Some(Duration)=timeout
    );

    println!("Playing sine wave... Press Enter to stop.");
    let _ = std::io::stdin().read_line(&mut String::new());

    stream.expect("where stream at").play().unwrap();
}

fn random_piano_frequency() -> f32 {
    let piano_notes = [
        27.5, 30.87, 32.7, 36.71, 41.2, 43.65, 49.0, 55.0, 61.74, 65.41, 73.42, 82.41, 87.31, 98.0, 110.0,
        123.47, 130.81, 146.83, 164.81, 174.61, 196.0, 220.0, 246.94, 261.63, 293.66, 329.63, 349.23, 392.0,
        440.0, 493.88, 523.25, 587.33, 659.25, 698.46, 784.0, 880.0, 987.77, 1046.5, 1174.66, 1318.51, 1396.91,
        1568.0, 1760.0, 1975.53, 2093.0, 2349.32, 2637.02, 2793.83, 3136.0, 3520.0, 3951.07, 4186.01
    ];
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..piano_notes.len());
    piano_notes[index]
}
