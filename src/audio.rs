use std::sync::Arc;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, StreamConfig,
};
use crate::state::AppState;
use tracing::{debug, error, info};
use std::collections::HashMap;

/// Number of steps to use in smoothing the waveform
const SMOOTHING_STEPS: usize = 4;
/// Reverb delay in samples
const REVERB_DELAY: usize = 48000; // 1 second at 48kHz
/// Reverb decay factor
const REVERB_DECAY: f32 = 0.5;

/// Starts the audio synthesizer task
pub async fn run_audio_synthesizer(state: Arc<AppState>) {
    tokio::task::spawn_blocking(move || {
        // Initialize the audio host and device
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(device) => {
                info!("Default output device found: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
                device
            },
            None => {
                error!("No output device found");
                return;
            }
        };

        // Get supported stream configuration
        let supported_config = match device.default_output_config() {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to get default output config: {}", e);
                return;
            }
        };

        let config = StreamConfig {
            channels: supported_config.channels(),
            sample_rate: supported_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        info!("Audio stream configuration: {:?}", config);

        // Start the audio stream
        let stream = match device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                process_audio_data(data, &state, config.sample_rate.0 as f32);
            },
            |err| {
                error!("An error occurred on the audio stream: {}", err);
            },
            None,
        ) {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to build audio stream: {}", e);
                return;
            }
        };

        info!("Starting audio stream...");
        if let Err(e) = stream.play() {
            error!("Failed to play audio stream: {}", e);
            return;
        }

        // Keep the thread alive to play audio indefinitely
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    })
    .await
    .expect("Failed to run audio synthesizer");
}

/// Processes audio data for the output stream
fn process_audio_data(data: &mut [f32], state: &AppState, sample_rate: f32) {
    let active_notes = state.active_notes.lock().unwrap();
    let mut waveform_buffer = state.waveform_buffer.lock().unwrap();

    // Clear waveform buffer
    waveform_buffer.fill(0.0);

    let mut phase_accumulators = std::collections::HashMap::new();
    let mut smoothing_buffer = vec![0.0; SMOOTHING_STEPS];
    let mut buffer_index = 0; // Tracks where to write in the waveform buffer

    // Reverb effect buffer
    let mut reverb_buffer = vec![0.0; REVERB_DELAY];
    let mut reverb_index = 0;

    for frame in data.chunks_mut(2) { // Stereo output
        let mut sample_value: f32 = 0.0;

        // Generate audio sample from active notes
        for &(note, velocity) in active_notes.iter() {
            let freq = midi_note_to_freq(note);
            let phase_accumulator = phase_accumulators.entry(note).or_insert(0.0);
            let phase_increment = freq / sample_rate;
            *phase_accumulator = (*phase_accumulator + phase_increment) % 1.0;

            // 16-bit era: Use a combination of square and sine waves
            let square_wave = if *phase_accumulator < 0.5 { 1.0 } else { -1.0 };
            let sine_wave = (2.0 * std::f32::consts::PI * *phase_accumulator).sin();

            // Mix square and sine waves for a richer tone
            let mixed_wave = 0.7 * square_wave + 0.3 * sine_wave;

            // Apply velocity as volume control (scaled from 0-127 to 0.0-1.0)
            let volume = velocity as f32 / 127.0;
            sample_value += mixed_wave * volume;
        }

        // Apply reverb effect
        let reverb_sample = reverb_buffer[reverb_index];
        let wet_sample = sample_value + reverb_sample * REVERB_DECAY;
        reverb_buffer[reverb_index] = wet_sample;
        reverb_index = (reverb_index + 1) % REVERB_DELAY;

        sample_value = wet_sample;

        // Smooth the waveform
        smoothing_buffer.rotate_left(1); // Shift buffer values
        smoothing_buffer[SMOOTHING_STEPS - 1] = sample_value; // Add the new value
        sample_value = smoothing_buffer.iter().sum::<f32>() / SMOOTHING_STEPS as f32;

        sample_value = sample_value.clamp(-1.0, 1.0); // Prevent clipping

        for sample in frame.iter_mut() {
            *sample = Sample::from_sample(sample_value);
        }

        // Update waveform buffer (for visualization)
        if buffer_index < waveform_buffer.len() {
            waveform_buffer[buffer_index] = sample_value;
            buffer_index += 1;
        }
    }
}

/// Converts a MIDI note to frequency
fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * (2.0_f32).powf((note as f32 - 69.0) / 12.0)
}
