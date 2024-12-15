use midir::{Ignore, MidiInput};
use std::sync::{Arc, Mutex};
use crate::state::AppState;
use tracing::{debug, error, info, warn};

pub async fn run_midi_listener(state: Arc<AppState>) {
    tokio::task::spawn_blocking(move || {
        // Initialize MIDI input
        let mut midi_input = match MidiInput::new("MIDI Input") {
            Ok(input) => input,
            Err(e) => {
                error!("Failed to create MIDI input: {}", e);
                return;
            }
        };
        midi_input.ignore(Ignore::None); // Capture all MIDI events

        // List available MIDI ports
        let in_ports = midi_input.ports();
        if in_ports.is_empty() {
            warn!("No MIDI input devices found!");
            return;
        }

        info!("Available MIDI input ports:");
        for (i, port) in in_ports.iter().enumerate() {
            info!("Port {}: {}", i, midi_input.port_name(port).unwrap_or_else(|_| "Unknown".to_string()));
        }

        // Select the first available port
        let in_port = &in_ports[1];
        info!("Using MIDI input: {}", midi_input.port_name(in_port).unwrap_or_else(|_| "Unknown".to_string()));

        // Connect to the selected MIDI port
        let _conn = midi_input
            .connect(
                in_port,
                "MIDI Listener",
                move |_, message, state| {
                    if message.len() >= 3 {
                        let status = message[0] & 0xF0;
                        let note = message[1];
                        let velocity = message[2];
                        let mut active_notes = state.active_notes.lock().unwrap();

                        match (status, velocity) {
                            (0x90, v) if v > 0 => {
                                // Note On
                                active_notes.insert((note, velocity));
                                debug!("Note On: note={}, velocity={}", note, velocity);
                            }
                            (0x80, _) | (0x90, 0) => {
                                // Note Off
                                active_notes.retain(|&(n, _)| n != note);
                                debug!("Note Off: note={}", note);
                            }
                            _ => {
                                debug!("Unhandled MIDI message: {:?}", message);
                            }
                        }
                    }
                },
                state.clone(),
            )
            .unwrap_or_else(|e| {
                error!("Failed to connect to MIDI input device: {}", e);
                panic!("Failed") 
            });

        info!("MIDI listener connected and running");

        // Keep the thread alive to listen for MIDI events
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })
    .await
    .expect("Failed to run MIDI listener");
}
