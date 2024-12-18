use tracing_subscriber;
use clap::Parser;

mod midi;
mod state;
mod audio;
mod ui;

use state::AppState;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    enable_ui: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Create shared application state
    let app_state = AppState::new();

    // Spawn the MIDI listener task
    tokio::spawn(midi::run_midi_listener(app_state.clone()));
    tokio::spawn(audio::run_audio_synthesizer(app_state.clone()));
    
    if args.enable_ui {
        let _ = ui::run_ui(app_state.clone());
    }

    // Keep the main task alive (add more functionality here if needed)
    loop {
        // This is just a placeholder for keeping the main task running.
        // You can add other logic like interacting with the state or logs here.
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
