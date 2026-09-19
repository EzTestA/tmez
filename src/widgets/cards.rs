use eframe::egui;
use crate::theme::{CPU_COLOR, MEM_COLOR};
use super::graph::{draw_graph, Sample};

/// Карточка «ЦП» с крупным процентом и графиком.
pub fn cpu_card(
    ui: &mut egui::Ui,
    history: &[Sample],
    smoothed: f32,
    now: f64,
) {
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(egui::RichText::new("ЦП").size(16.0).strong());
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("{:.1}%", smoothed * 100.0))
                .size(28.0)
                .color(CPU_COLOR),
        );
        ui.add_space(6.0);
        draw_graph(ui, history, smoothed, 100.0, now);
    });
}

/// Карточка «Память» с процентом, объёмом и прогресс-баром.
pub fn memory_card(
    ui: &mut egui::Ui,
    smoothed: f32,
    used_gb: f64,
    total_gb: f64,
) {
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.label(egui::RichText::new("Память").size(16.0).strong());
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("{:.1}%", smoothed * 100.0))
                .size(28.0)
                .color(MEM_COLOR),
        );
        ui.add_space(6.0);
        ui.label(format!("{:.2} ГБ / {:.2} ГБ", used_gb, total_gb));
        ui.add_space(6.0);
        ui.add(
            egui::ProgressBar::new(smoothed)
                .desired_width(ui.available_width())
                .fill(MEM_COLOR),
        );
    });
}