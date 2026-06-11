fn main() -> eframe::Result<()> {
    let app = umbra::desktop::App::default();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native("UMBRA", options, Box::new(|_cc| Ok(Box::new(app))))
}
