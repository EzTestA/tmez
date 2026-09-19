use eframe::egui;

/// Применяет кастомную тёмную тему в стиле tmog.
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = egui::Color32::from_rgb(10, 10, 14);
    visuals.window_fill = egui::Color32::from_rgb(14, 14, 18);
    visuals.extreme_bg_color = egui::Color32::from_rgb(6, 6, 10);
    visuals.faint_bg_color = egui::Color32::from_rgb(18, 18, 24);

    visuals.selection.bg_fill = egui::Color32::from_rgb(40, 90, 160);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(80, 180, 255));
    visuals.hyperlink_color = egui::Color32::from_rgb(80, 180, 255);

    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = egui::Color32::from_rgb(16, 16, 22);
    w.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 30, 40));

    w.inactive.bg_fill = egui::Color32::from_rgb(20, 20, 28);
    w.inactive.weak_bg_fill = egui::Color32::from_rgb(16, 16, 22);
    w.inactive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(35, 35, 48));

    w.hovered.bg_fill = egui::Color32::from_rgb(28, 28, 40);
    w.hovered.weak_bg_fill = egui::Color32::from_rgb(24, 24, 34);
    w.hovered.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 60, 90));

    w.active.bg_fill = egui::Color32::from_rgb(40, 90, 160);
    w.active.weak_bg_fill = egui::Color32::from_rgb(35, 75, 140);
    w.active.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(80, 180, 255));

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

// ============ Параметры графиков ============
// Вынесены сюда, чтобы все виджеты использовали одни и те же константы.

/// Секунд между точками истории.
pub const INTERVAL: f64 = 0.5;

/// Пикселей между точками по горизонтали.
pub const STEP_PX: f32 = 20.0;

/// Максимум точек в истории.
pub const MAX_POINTS: usize = 80;

/// Время сглаживания (секунд).
pub const TAU: f64 = 0.25;

/// Цвет акцента (голубой) — используется в заголовках, выделениях.
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 180, 255);

/// Цвет CPU-графика.
pub const CPU_COLOR: egui::Color32 = egui::Color32::from_rgb(80, 180, 255);

/// Цвет памяти.
pub const MEM_COLOR: egui::Color32 = egui::Color32::from_rgb(220, 100, 220);