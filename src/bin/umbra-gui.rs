// Zone 0 — Config/Init
use eframe::egui;

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
    eframe::run_native("UMBRA", options, Box::new(|cc| {
        // Load emoji font for icon rendering
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(emoji_data) = std::fs::read("/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf") {
            fonts.font_data.insert("emoji".into(), egui::FontData::from_owned(emoji_data));
            if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                proportional.push("emoji".into());
            }
        } else if let Ok(emoji_data) = std::fs::read("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc") {
            fonts.font_data.insert("cjk".into(), egui::FontData::from_owned(emoji_data));
            if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                proportional.push("cjk".into());
            }
        }
        // Try system emoji fonts at common locations
        for path in &[
            "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ] {
            if let Ok(data) = std::fs::read(path) {
                let name = path.split('/').last().unwrap_or("font");
                fonts.font_data.insert(name.into(), egui::FontData::from_owned(data));
                if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    proportional.push(name.into());
                }
            }
        }
        cc.egui_ctx.set_fonts(fonts);
        Ok(Box::new(app))
    }))
}
