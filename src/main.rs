use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, FromSample};
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
    let mut frequency = 220.0; // Initial frequency
    let mut target_frequency = frequency; // Frequency to transition to
    let volume = 0.5; // Volume control

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Handle audio stream
            let mut rng = rand::thread_rng();
            for frame in data.chunks_mut(2) { // Stereo output (2 samples per frame)
                // Smooth frequency transition
                frequency += (target_frequency - frequency) * 0.01; // Smooth interpolation

                // Generate sample for sine wave
                let sample_value = (sample_clock * 2.0 * PI).sin() * volume;
                for sample in frame.iter_mut() {
                    *sample = Sample::from_sample(sample_value);
                }

                // Update sample clock
                sample_clock = (sample_clock + (frequency / sample_rate)) % 1.0;

                // Randomly pick a new target frequency occasionally
                if sample_clock % 0.5 == 0.0 { // Approximately once per second
                    target_frequency = random_piano_frequency();
                    println!("Changing frequency to: {:.2} Hz", target_frequency);
                }
            }
        },
        move |err| {
            // Error handling
            println!("An error occurred on the output stream: {}", err);
        },
        None,
    ).expect("Failed to build stream");

    println!("Playing dynamic stereo sine wave... Press Enter to stop.");
    stream.play().expect("Failed to play stream");
    let _ = std::io::stdin().read_line(&mut String::new());
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
