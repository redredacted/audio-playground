use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use eframe::{egui, App};
use egui_plot::{Line, Plot, PlotPoints};
use midir::{Ignore, MidiInput};
use std::collections::HashSet;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), eframe::Error> {
    // Shared state between audio and GUI threads
    let active_notes = Arc::new(Mutex::new(HashSet::new()));
    let waveform_buffer = Arc::new(Mutex::new(vec![0.0; 1024])); // Buffer for visualization

    // MIDI setup
    let midi_thread_active_notes = Arc::clone(&active_notes);
    thread::spawn(move || {
        let mut midi_input = MidiInput::new("MIDI Input").expect("Failed to create MIDI input");
        midi_input.ignore(Ignore::None); // Capture all events

        let in_ports = midi_input.ports();
        if in_ports.is_empty() {
            println!("No MIDI input devices found!");
            return;
        }

        println!("Available MIDI input ports:");
        for (i, port) in in_ports.iter().enumerate() {
            println!("Port {}: {}", i, midi_input.port_name(port).unwrap());
        }

        let in_port = &in_ports[1];
        println!("Using MIDI input: {}", midi_input.port_name(in_port).unwrap());

        let _conn = midi_input
            .connect(
                in_port,
                "MIDI Keyboard",
                move |_, message, active_notes| {
                    if message.len() >= 3 {
                        let status = message[0] & 0xF0;
                        let note = message[1];
                        let velocity = message[2];

                        let mut notes = active_notes.lock().unwrap();

                        match (status, velocity) {
                            (0x90, v) if v > 0 => {
                                notes.insert((note, velocity));
                            }
                            (0x80, _) | (0x90, 0) => {
                                notes.retain(|&(n, _)| n != note);
                            }
                            _ => {}
                        }
                    }
                },
                Arc::clone(&midi_thread_active_notes),
            )
            .expect("Failed to connect to MIDI device");

        // Keep thread alive to listen for MIDI input
        loop {
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Audio setup
    let audio_thread_active_notes = Arc::clone(&active_notes);
    let audio_thread_waveform_buffer = Arc::clone(&waveform_buffer);
    thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No output device found");
        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("No supported config");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate();
        let config = supported_config.config();
        let sample_rate = config.sample_rate.0 as u64;

        let mut sample_clock: u64 = 0;
        let base_volume = 0.2; // Base volume reduced to avoid clipping
        let phase_increment = 2.0 * PI / (sample_rate as f32);
        let mut envelopes = std::collections::HashMap::new();
        let attack_time = sample_rate as f32 * 0.01; // Attack time of 10ms
        let release_time = sample_rate as f32 * 0.3; // Increased release time for even smoother fade out
        let release_threshold = 0.0001; // Reduced threshold to make release more gradual

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let active_notes = audio_thread_active_notes.lock().unwrap();
                    let mut buffer = audio_thread_waveform_buffer.lock().unwrap();
                    let mut index = 0;

                    for frame in data.chunks_mut(2) {
                        // Stereo output
                        let mut sample_value: f32 = 0.0;

                        // Process active notes with attack
                        for &(note, _) in active_notes.iter() {
                            let envelope = envelopes.entry(note).or_insert(0.0);
                            if *envelope < 1.0 {
                                *envelope += 1.0 / attack_time;
                                *envelope = envelope.clamp(0.0, 1.0);
                            }

                            let freq = midi_note_to_freq(note);
                            let phase = ((sample_clock as f32) * freq * phase_increment).sin();
                            sample_value += phase * base_volume * *envelope;
                        }

                        // Handle release phase for notes that are no longer active
                        for note in envelopes.keys().cloned().collect::<Vec<_>>() {
                            if !active_notes.contains(&(note, 0)) {
                                let envelope = envelopes.get_mut(&note).unwrap();
                                if *envelope > release_threshold {
                                    *envelope -= 1.0 / release_time;
                                    *envelope = envelope.clamp(0.0, 1.0);
                                    let freq = midi_note_to_freq(note);
                                    let phase = ((sample_clock as f32) * freq * phase_increment).sin();
                                    sample_value += phase * base_volume * *envelope;
                                } else {
                                    *envelope = 0.0;
                                    envelopes.remove(&note);
                                }
                            }
                        }

                        // Apply a simple low-pass filter to reduce clicking noise
                        sample_value = sample_value.clamp(-1.0, 1.0);

                        for sample in frame.iter_mut() {
                            *sample = Sample::from_sample(sample_value);
                        }

                        // Store the sample for visualization
                        if index < buffer.len() {
                            buffer[index] = sample_value;
                            index += 1;
                        }

                        // Increment the sample clock, wrapping to prevent overflow
                        sample_clock = sample_clock.wrapping_add(1);
                    }
                },
                move |err| {
                    println!("An error occurred on the output stream: {}", err);
                },
                None,
            )
            .expect("Failed to build stream");

        stream.play().expect("Failed to play stream");

        // Keep thread alive to play audio indefinitely
        loop {
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    // Run egui app
    eframe::run_native(
        "Waveform Visualizer",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.request_repaint(); // Ensure the UI updates continuously
            Ok(Box::new(WaveformApp {
                buffer: waveform_buffer,
            }))
        }),
    )
}

// Convert MIDI note to frequency
fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * (2.0_f32).powf((note as f32 - 69.0) / 12.0)
}

// eframe app for visualization
struct WaveformApp {
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl App for WaveformApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint(); // Request repaint on every update to ensure continuous refresh
        let buffer = self.buffer.lock().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = Plot::new("Waveform").view_aspect(2.0);
            plot.show(ui, |plot_ui| {
                let values: Vec<_> = buffer
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| [i as f64, v as f64])
                    .collect();
                plot_ui.line(Line::new(PlotPoints::from(values)));
            });
        });
    }
}
