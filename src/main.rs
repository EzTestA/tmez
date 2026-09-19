mod app;
mod theme;
mod widgets;
mod tabs;
mod processes;

use app::TmezApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("TMEZ — Монитор системы"),
        ..Default::default()
    };

    eframe::run_native(
        "tmez",
        options,
        Box::new(|cc| {
            theme::apply_theme(&cc.egui_ctx);
            Ok(Box::new(TmezApp::new()))
        }),
    )
}