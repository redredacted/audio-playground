use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};

/// Centralized state shared across MIDI, audio, and UI components.
#[derive(Debug)]
pub struct AppState {
    /// A set of active notes, represented as (note, velocity).
    pub active_notes: Mutex<HashSet<(u8, u8)>>,

    /// a buffer for note phases
    pub phase_accumulators: Mutex<HashMap<u8, f32>>,

    /// A buffer to store audio waveform data for visualization.
    pub waveform_buffer: Mutex<Vec<f32>>,
}

impl AppState {
    /// Create a new `AppState` with default values.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            active_notes: Mutex::new(HashSet::new()),
            waveform_buffer: Mutex::new(vec![0.0; 1024]), // Initialize with 1024 samples
            phase_accumulators: Mutex::new(std::collections::HashMap::new()), // Initialize with 1024 samples
        })
    }
}
