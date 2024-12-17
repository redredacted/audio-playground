use std::sync::Arc;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, StreamConfig,
};
use crate::state::AppState;
use tracing::{error, info};

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
    let mut buffer_index = 0;

    // Clear waveform buffer
    waveform_buffer.fill(0.0);

    let mut phase_accumulators = state.phase_accumulators.lock().unwrap();

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
            let sawtooth_wave = 2.0 * *phase_accumulator - 1.0;
            let triangle_wave = (2.0 * *phase_accumulator - 1.0).abs() * 2.0 - 1.0;

            // Mix square and sine waves for a richer tone
            let mixed_wave = 0.3 * sine_wave + 0.3 * square_wave + 0.2 * sawtooth_wave + 0.2 * triangle_wave;

            // Apply velocity as volume control (scaled from 0-127 to 0.0-1.0)
            let volume = velocity as f32 / 127.0;
            sample_value += mixed_wave * volume;
        }

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
