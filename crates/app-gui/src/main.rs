#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod theme;
// mod state;
// mod components;
pub mod panels;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    // Create a background tokio runtime
    let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Ferrum Lens"),
        ..Default::default()
    };

    eframe::run_native(
        "Ferrum Lens",
        native_options,
        Box::new(|cc| Ok(Box::new(app::FerrumApp::new(cc)))),
    )
}
