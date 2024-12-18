use std::sync::Arc;
use eframe::{App, CreationContext};
use egui::{CentralPanel, Context};
use egui_plot::{Line, Plot, PlotPoints, PlotBounds};
use crate::state::AppState;

/// A struct representing the application UI.
pub struct WaveformApp {
    state: Arc<AppState>,
}

impl WaveformApp {
    /// Creates a new instance of the WaveformApp.
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl App for WaveformApp {
    /// The update method is called every frame to update and render the UI.
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        // Lock the waveform buffer for reading
        let buffer = self.state.waveform_buffer.lock().unwrap();
        ctx.request_repaint();

        // Create the UI
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Waveform Visualizer");

            let plot = Plot::new("Waveform")
                .view_aspect(2.0) // Set aspect ratio for the plot
                .show_axes([true, true]); // Show X and Y axes

            plot.show(ui, |plot_ui| {
                // Convert the waveform buffer to points for plotting
                let points: Vec<_> = buffer.iter()
                    .enumerate()
                    .map(|(i, &v)| [i as f64, v as f64])
                    .collect();

                // Plot the waveform
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [0.0, -1.1],
                    [1024.0, 1.1],
                ));
                plot_ui.line(Line::new(PlotPoints::from(points)));
            });
        });
    }
}

/// Initializes and runs the eframe application.
pub fn run_ui(state: Arc<AppState>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Waveform Visualizer",
        options,
        Box::new(|_cc: &CreationContext| Ok(Box::new(WaveformApp::new(state.clone())))),

    )
}
