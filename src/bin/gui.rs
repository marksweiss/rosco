use eframe::egui;
use osc::gui::RoscoGuiApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1200.0, 800.0])
            .with_title("Rosco Synthesizer"),
        ..Default::default()
    };

    eframe::run_native(
        "Rosco Synthesizer",
        options,
        Box::new(|cc| Ok(Box::new(RoscoGuiApp::new(cc)))),
    )
}
