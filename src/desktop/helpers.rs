// Zone 1 — Desktop UI Helpers
use eframe::egui::{self, Color32, Vec2, Rounding, scroll_area::ScrollBarVisibility};
use super::HOVER_PURPLE;

pub fn which_exists(name: &str) -> bool {
    std::env::var("PATH").map(|path| {
        path.split(':').any(|dir| {
            let full = format!("{}/{}", dir, name);
            std::fs::metadata(&full).is_ok()
        })
    }).unwrap_or(false)
}

pub fn btn(ui: &mut egui::Ui, text: impl Into<egui::RichText>) -> egui::Response {
    btn_fill(ui, text, Color32::TRANSPARENT, HOVER_PURPLE)
}

pub fn btn_rounded(ui: &mut egui::Ui, text: impl Into<egui::RichText>, rounding: Rounding, min_size: Vec2) -> egui::Response {
    let text: egui::RichText = text.into();
    let id = ui.next_auto_id();
    let prev_hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let bg = if prev_hovered { HOVER_PURPLE } else { Color32::TRANSPARENT };
    let resp = ui.add(egui::Button::new(text).fill(bg).rounding(rounding).min_size(min_size));
    ui.data_mut(|d| d.insert_temp(id, resp.hovered()));
    resp
}

pub fn btn_fill(ui: &mut egui::Ui, text: impl Into<egui::RichText>, normal: Color32, hover: Color32) -> egui::Response {
    let text: egui::RichText = text.into();
    let id = ui.next_auto_id();
    let prev_hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let bg = if prev_hovered { hover } else { normal };
    let resp = ui.add(egui::Button::new(text).fill(bg));
    ui.data_mut(|d| d.insert_temp(id, resp.hovered()));
    resp
}

pub fn scroll_area() -> egui::ScrollArea {
    egui::ScrollArea::vertical()
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
}
