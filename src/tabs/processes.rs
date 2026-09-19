use eframe::egui;
use super::draw_placeholder;

pub fn draw(ui: &mut egui::Ui) {
    draw_placeholder(ui, "Процессы");
}