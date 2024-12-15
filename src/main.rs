use tracing_subscriber;

mod midi;
mod state;
mod audio;
mod ui;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Create shared application state
    let app_state = AppState::new();

    // Spawn the MIDI listener task
    tokio::spawn(midi::run_midi_listener(app_state.clone()));
    tokio::spawn(audio::run_audio_synthesizer(app_state.clone()));
    
    let _ = ui::run_ui(app_state.clone());

    // Keep the main task alive (add more functionality here if needed)
    loop {
        // This is just a placeholder for keeping the main task running.
        // You can add other logic like interacting with the state or logs here.
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
