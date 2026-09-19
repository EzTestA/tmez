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

    // Точный расчёт: сколько места занимают память и нижний ряд
    // - memory_card: ~116
    // - gap между верхним рядом и памятью: 8
    // - gap между памятью и нижним рядом: 8
    // - bottom_row: 80
    // - запас: 10
    let reserved = 116.0 + 8.0 + 8.0 + 80.0 + 30.0;
    let top_row_height = (ui.available_height() - reserved).max(160.0);

    draw_top_row(app, ui, now, top_row_height);
    ui.add_space(CARD_GAP);
    memory_card(ui, app.mem_smoothed, app.mem_used_gb, app.mem_total_gb);
    ui.add_space(CARD_GAP);
    draw_bottom_row(ui, app);
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
fn draw_bottom_row(ui: &mut egui::Ui, app: &TmezApp) {
    let h = BOTTOM_ROW_HEIGHT;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        let avail = ui.available_width();
        let w = ((avail - CARD_GAP * 3.0) / 4.0).max(80.0);

        // Сеть
        bottom_card(ui, "Сеть", NET_COLOR, w, h, &[
            (format!("↓ {}", fmt_bps(app.net_rx_bps)), egui::Color32::from_gray(200)),
            (format!("↑ {}", fmt_bps(app.net_tx_bps)), egui::Color32::from_gray(160)),
        ]);

        ui.add_space(CARD_GAP);

        // Диски
        bottom_card(ui, "Диски", DISK_COLOR, w, h, &[
            (format!("Ч {}", fmt_bps(app.disk_read_bps)), egui::Color32::from_gray(200)),
            (format!("З {}", fmt_bps(app.disk_write_bps)), egui::Color32::from_gray(160)),
        ]);

        ui.add_space(CARD_GAP);

        // GPU
        let gpu_temp = if app.gpu_temp_c > 0.0 {
            format!("{:.0} °C", app.gpu_temp_c)
        } else {
            "—".to_string()
        };
        bottom_card(ui, "GPU", GPU_COLOR, w, h, &[
            (format!("Загр: {:.0}%", app.gpu_nvidia_load * 100.0), egui::Color32::from_gray(200)),
            (format!("Темп: {}", gpu_temp), egui::Color32::from_gray(160)),
        ]);

        ui.add_space(CARD_GAP);

        // Питание ЦП — заглушка
        bottom_card(ui, "Питание ЦП", POWER_COLOR, w, h, &[
            ("—".to_string(), egui::Color32::from_gray(140)),
        ]);
    });
}

fn bottom_card(
    ui: &mut egui::Ui,
    title: &str,
    color: egui::Color32,
    w: f32,
    h: f32,
    lines: &[(String, egui::Color32)],
) {
    ui.allocate_ui_with_layout(
        egui::vec2(w, h),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            card_frame(ui, w, h, title, color, |ui| {
                for (text, c) in lines {
                    ui.label(egui::RichText::new(text).size(11.0).color(*c));
                }
            });
        },
    );
}

/// Красиво форматирует байты/сек в КБ/с, МБ/с, ГБ/с.
fn fmt_bps(bps: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bps >= GB {
        format!("{:.2} ГБ/с", bps / GB)
    } else if bps >= MB {
        format!("{:.2} МБ/с", bps / MB)
    } else if bps >= KB {
        format!("{:.1} КБ/с", bps / KB)
    } else {
        format!("{:.0} Б/с", bps)
    }
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
        ui.spacing_mut().item_spacing.y = 0.0;

        // Фиксированные ширины колонок
        const PID_W: f32 = 55.0;
        const NAME_W: f32 = 110.0;
        const CPU_W: f32 = 55.0;
        const MEM_W: f32 = 65.0;

        // Расчёт количества строк, которые влезут
        let used_height = 135.0;
        let rows_area = (h - used_height).max(0.0);
        let row_height = 16.0;
        let max_rows = (rows_area / row_height).floor() as usize;
        let take_n = max_rows.min(app.top_processes.len());

        egui::Grid::new("top_processes_grid")
            .num_columns(4)
            .spacing([4.0, 3.0])
            .min_col_width(55.0)
            .max_col_width(200.0)
            .show(ui, |ui| {
                let header_color = egui::Color32::from_gray(140);

                // === Шапка ===
                cell(ui, PID_W, egui::Align::LEFT, |ui| {
                    ui.label(egui::RichText::new("PID").size(11.0).color(header_color));
                });
                cell(ui, NAME_W, egui::Align::LEFT, |ui| {
                    ui.label(egui::RichText::new("Имя").size(11.0).color(header_color));
                });
                cell(ui, CPU_W, egui::Align::RIGHT, |ui| {
                    ui.label(egui::RichText::new("ЦП").size(11.0).color(header_color));
                });
                cell(ui, MEM_W, egui::Align::RIGHT, |ui| {
                    ui.label(egui::RichText::new("ОЗУ").size(11.0).color(header_color));
                });
                ui.end_row();

                // === Строки ===
                for p in app.top_processes.iter().take(take_n) {
                    // PID
                    cell(ui, PID_W, egui::Align::LEFT, |ui| {
                        ui.label(
                            egui::RichText::new(p.pid.to_string())
                                .size(11.0)
                                .color(egui::Color32::from_gray(160)),
                        );
                    });

                    // Имя
                    let name = truncate_name(&p.name, 16);
                    cell(ui, NAME_W, egui::Align::LEFT, |ui| {
                        ui.label(
                            egui::RichText::new(name)
                                .size(11.0)
                                .color(egui::Color32::from_gray(200)),
                        );
                    });

                    // ЦП
                    cell(ui, CPU_W, egui::Align::RIGHT, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", p.cpu))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(200, 220, 240)),
                        );
                    });

                    // ОЗУ
                    let mem_str = if p.memory_mb >= 1024.0 {
                        format!("{:.1} ГБ", p.memory_mb / 1024.0)
                    } else {
                        format!("{:.0} МБ", p.memory_mb)
                    };
                    cell(ui, MEM_W, egui::Align::RIGHT, |ui| {
                        ui.label(
                            egui::RichText::new(mem_str)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(180, 200, 220)),
                        );
                    });

                    ui.end_row();
                }
            });
    });
}

/// Рисует содержимое ячейки в блоке фиксированной ширины `w`
/// с выравниванием по горизонтали `align`.
fn cell<F: FnOnce(&mut egui::Ui)>(
    ui: &mut egui::Ui,
    w: f32,
    align: egui::Align,
    content: F,
) {
    let layout = if align == egui::Align::RIGHT {
        egui::Layout::right_to_left(egui::Align::Center)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    };
    ui.allocate_ui_with_layout(egui::vec2(w, 14.0), layout, |ui| {
        content(ui);
    });
}

/// Обрезает имя процесса до `max_chars` символов,
/// добавляя «…» в конце, если не влезло.
fn truncate_name(name: &str, max_chars: usize) -> String {
    if name.chars().count() > max_chars {
        let truncated: String = name.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    } else {
        name.to_string()
    }
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