use eframe::egui;
use crate::app::TmezApp;
use crate::theme::ACCENT;
use crate::widgets::cards::memory_card;
use crate::widgets::graph::draw_graph;

// ============ Цвета ============
const SYS_COLOR:   egui::Color32 = egui::Color32::from_rgb(120, 220, 120);
const DISK_COLOR:  egui::Color32 = egui::Color32::from_rgb(120, 220, 120);
const NET_COLOR:   egui::Color32 = egui::Color32::from_rgb(80, 180, 255);
const GPU_COLOR:   egui::Color32 = egui::Color32::from_rgb(180, 180, 200);
const POWER_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 90, 90);
const TOP_COLOR:   egui::Color32 = egui::Color32::from_rgb(200, 200, 200);

const CARD_BG:   egui::Color32 = egui::Color32::from_rgb(12, 12, 16);
const CARD_EDGE: egui::Color32 = egui::Color32::from_rgb(30, 30, 40);

// ============ Размеры ============
const SYS_WIDTH: f32 = 220.0;
const TOP_WIDTH: f32 = 300.0;
const TOP_ROW_HEIGHT: f32 = 200.0;
const BOTTOM_ROW_HEIGHT: f32 = 80.0;
const CARD_GAP: f32 = 8.0;

// ============ Точка входа вкладки ============
pub fn draw(app: &mut TmezApp, ui: &mut egui::Ui, now: f64) {
    draw_header(ui);
    ui.add_space(12.0);

    // Резервируем место под нижние элементы:
    // - полоса памяти: 100 px
    // - нижний ряд: 80 px
    // - два отступа между ними: 8 + 8 px
    // - запас на отступы сверху/снизу: 8 px
    let reserved = 100.0 + 80.0 + 8.0 + 8.0 + 8.0;
    let top_row_height = (ui.available_height() - reserved).max(160.0);

    draw_top_row(app, ui, now, top_row_height);
    ui.add_space(CARD_GAP);
    memory_card(ui, app.mem_smoothed, app.mem_used_gb, app.mem_total_gb);
    ui.add_space(CARD_GAP);
    draw_bottom_row(ui);
}

// ============ Заголовок вкладки ============
fn draw_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("📊").size(20.0).color(ACCENT));
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("Обзор")
                .size(22.0)
                .strong()
                .color(egui::Color32::from_gray(220)),
        );
    });
}

// ============ Верхний ряд ============
fn draw_top_row(app: &TmezApp, ui: &mut egui::Ui, now: f64, h: f32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        let avail = ui.available_width();
        let center_w = (avail - SYS_WIDTH - TOP_WIDTH - CARD_GAP * 2.0).max(200.0);

        ui.allocate_ui_with_layout(
            egui::vec2(SYS_WIDTH, h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| sys_card(ui, app, SYS_WIDTH, h),
        );
        ui.add_space(CARD_GAP);
        ui.allocate_ui_with_layout(
            egui::vec2(center_w, h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| cpu_overview_card(ui, app, now, center_w, h),
        );
        ui.add_space(CARD_GAP);
        ui.allocate_ui_with_layout(
            egui::vec2(TOP_WIDTH, h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| top_processes_card(ui, app, TOP_WIDTH, h),
        );
    });
}

// ============ Нижний ряд ============
fn draw_bottom_row(ui: &mut egui::Ui) {
    let h = BOTTOM_ROW_HEIGHT;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        let avail = ui.available_width();
        let w = ((avail - CARD_GAP * 3.0) / 4.0).max(80.0);

        bottom_card(ui, "Сеть", NET_COLOR, w, h);
        ui.add_space(CARD_GAP);
        bottom_card(ui, "Диски", DISK_COLOR, w, h);
        ui.add_space(CARD_GAP);
        bottom_card(ui, "GPU", GPU_COLOR, w, h);
        ui.add_space(CARD_GAP);
        bottom_card(ui, "Питание ЦП", POWER_COLOR, w, h);
    });
}

fn bottom_card(ui: &mut egui::Ui, title: &str, color: egui::Color32, w: f32, h: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(w, h),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            card_frame(ui, w, h, title, color, |ui| {
                ui.label(
                    egui::RichText::new("—")
                        .size(16.0)
                        .color(egui::Color32::from_gray(140)),
                );
            });
        },
    );
}

// ============ SYS ============
fn sys_card(ui: &mut egui::Ui, app: &TmezApp, w: f32, h: f32) {
    card_frame(ui, w, h, "SYS", SYS_COLOR, |ui| {
    let labels_top = ["ЦП", "Частота", "Темп.", "GPU"];
    let labels_bot = [
        format!("{:.1}%", app.cpu_smoothed * 100.0),
        format!("{:.2} GHz", app.cpu_freq_ghz),
        "—".to_string(),
        format!("{:.0}%", app.gpu_nvidia_load * 100.0),
    ];
    let colors = [
        egui::Color32::from_rgb(120, 220, 120),   // ЦП
        egui::Color32::from_rgb(230, 90, 90),     // Частота
        egui::Color32::from_rgb(180, 140, 90),    // Темп.
        egui::Color32::from_rgb(80, 180, 255),    // GPU
    ];
    let freq_frac = (app.cpu_freq_ghz / 3.0).clamp(0.0, 1.0);
    let fracs = [app.cpu_smoothed, freq_frac, 0.0, app.gpu_nvidia_load];

        // Фиксированные высоты элементов
        let label_h = 14.0;
        let spacing_y = 6.0;

        // Высота полоски = вся оставшаяся высота
        let bar_h = (ui.available_height() - label_h * 2.0 - spacing_y * 2.0).max(40.0);

        // Размеры колонок
        let inner_w = ui.available_width();
        let col_gap = 4.0;
        let col_w = ((inner_w - col_gap * 3.0) / 4.0).max(20.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            for i in 0..4 {
                ui.allocate_ui_with_layout(
                    egui::vec2(col_w, bar_h + label_h * 2.0 + spacing_y * 2.0),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        // Отключаем автоотступы внутри колонки
                        ui.spacing_mut().item_spacing.y = 0.0;

                        // Верхняя подпись
                        let (top_rect, _) = ui.allocate_exact_size(
                            egui::vec2(col_w, label_h),
                            egui::Sense::hover(),
                        );
                        ui.painter().text(
                            top_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            labels_top[i],
                            egui::FontId::proportional(10.0),
                            colors[i],
                        );

                        ui.add_space(spacing_y);

                        draw_vertical_bar(ui, col_w, bar_h, fracs[i], colors[i]);

                        ui.add_space(spacing_y);

                        // Нижняя подпись
                        let (bot_rect, _) = ui.allocate_exact_size(
                            egui::vec2(col_w, label_h),
                            egui::Sense::hover(),
                        );
                        ui.painter().text(
                            bot_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &labels_bot[i],
                            egui::FontId::proportional(10.0),
                            egui::Color32::from_gray(180),
                        );
                    },
                );

                if i < 3 {
                    ui.add_space(col_gap);
                }
            }
        });
    });
}

fn draw_vertical_bar(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    frac: f32,
    color: egui::Color32,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(20, 20, 28));

    let frac = frac.clamp(0.0, 1.0);
    if frac > 0.0 {
        let fill_h = rect.height() * frac;
        let fill_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - fill_h),
            egui::pos2(rect.right(), rect.bottom()),
        );
        painter.rect_filled(fill_rect, 2.0, color);
    }
}

// ============ CPU OVERVIEW ============
fn cpu_overview_card(ui: &mut egui::Ui, app: &TmezApp, now: f64, w: f32, h: f32) {
    card_frame(ui, w, h, "ЦП — ОБЗОР", ACCENT, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{:.0}%", app.cpu_smoothed * 100.0))
                    .size(13.0)
                    .color(ACCENT),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("использование")
                        .size(11.0)
                        .color(egui::Color32::from_gray(140)),
                );
            });
        });

        ui.add_space(4.0);

        let graph_h = (ui.available_height() - 4.0).max(40.0);
        draw_graph(ui, &app.cpu_history, app.cpu_smoothed, graph_h, now);
    });
}

// ============ TOP PROCESSES ============
fn top_processes_card(ui: &mut egui::Ui, app: &TmezApp, w: f32, h: f32) {
    card_frame(ui, w, h, "ТОП ПО ЦП", TOP_COLOR, |ui| {
        // Шапка
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("PID").size(11.0).color(egui::Color32::from_gray(140)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("ЦП").size(11.0).color(egui::Color32::from_gray(140)));
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Имя").size(11.0).color(egui::Color32::from_gray(140)));
            });
        });
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(4.0);

        // Сколько строк влезет в оставшуюся высоту
        let row_height = 16.0;
        let available_h = ui.available_height();
        let max_rows = ((available_h / row_height).floor() as usize).clamp(1, 50);

egui::ScrollArea::vertical()
    .auto_shrink([false, false])
    .show(ui, |ui| {
        if app.top_processes.is_empty() {
            ui.label(
                egui::RichText::new("нет данных")
                    .size(11.0)
                    .color(egui::Color32::from_gray(100)),
            );
        } else {
            for p in app.top_processes.iter().take(max_rows) {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(p.pid.to_string())
                            .size(11.0)
                            .color(egui::Color32::from_gray(160)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", p.cpu))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(200, 220, 240)),
                        );
                        ui.add_space(20.0);
                        let name = if p.name.len() > 20 {
                            format!("{}…", &p.name[..19])
                        } else {
                            p.name.clone()
                        };
                        ui.label(
                            egui::RichText::new(name)
                                .size(11.0)
                                .color(egui::Color32::from_gray(200)),
                        );
                    });
                });
            }
        }
    });
    });
}

// ============ Универсальная рамка карточки с точными размерами ============
fn card_frame<F: FnOnce(&mut egui::Ui)>(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    title: &str,
    title_color: egui::Color32,
    content: F,
) {
    let frame = egui::Frame::NONE
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0_f32, CARD_EDGE))
        .inner_margin(egui::Margin::same(8));

    frame.show(ui, |ui| {
        // Жёстко задаём размеры внутренней области
        let inner_w = width - 16.0;
        let inner_h = height - 16.0;
        ui.set_min_size(egui::vec2(inner_w, inner_h));
        ui.set_max_size(egui::vec2(inner_w, inner_h));

        // Заголовок
        ui.label(
            egui::RichText::new(title)
                .size(11.0)
                .strong()
                .color(title_color),
        );
        ui.add_space(6.0);

        content(ui);
    });
}