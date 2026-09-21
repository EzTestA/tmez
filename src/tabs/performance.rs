use eframe::egui;
use crate::app::TmezApp;
use crate::theme::{ACCENT, CPU_COLOR, MEM_COLOR, INTERVAL, STEP_PX};
use crate::widgets::graph::gradient_color;

// Цвета датчиков
const DISK_COLOR:  egui::Color32 = egui::Color32::from_rgb(120, 220, 120);
const NET_COLOR:   egui::Color32 = egui::Color32::from_rgb(80, 200, 255);
const GPU_COLOR:   egui::Color32 = egui::Color32::from_rgb(160, 180, 210);

const CARD_BG_ACTIVE: egui::Color32 = egui::Color32::from_rgb(40, 70, 130);
const CARD_BG_HOVER:  egui::Color32 = egui::Color32::from_rgb(24, 24, 34);
const CARD_BG:        egui::Color32 = egui::Color32::from_rgb(12, 12, 16);
const CARD_EDGE:      egui::Color32 = egui::Color32::from_rgb(30, 30, 40);

const SENSOR_WIDTH: f32 = 220.0;

/// Датчики во вкладке «Производительность».
/// Список формируется динамически — для каждого тома добавляется Disk(idx).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sensor {
    Cpu,
    Memory,
    Disk(usize),
    Network,
    Nvidia,
}

impl Sensor {
    /// Формирует список датчиков с учётом числа томов.
    pub fn build_list(disk_count: usize) -> Vec<Sensor> {
        let mut v = vec![Sensor::Cpu, Sensor::Memory];
        for i in 0..disk_count {
            v.push(Sensor::Disk(i));
        }
        v.push(Sensor::Network);
        v.push(Sensor::Nvidia);
        v
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Sensor::Cpu => CPU_COLOR,
            Sensor::Memory => MEM_COLOR,
            Sensor::Disk(_) => DISK_COLOR,
            Sensor::Network => NET_COLOR,
            Sensor::Nvidia => GPU_COLOR,
        }
    }
}

pub fn draw(app: &mut TmezApp, ui: &mut egui::Ui, now: f64) {
    draw_header(ui);
    ui.add_space(12.0);

    let h = ui.available_height();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        // === Левая колонка: список датчиков ===
        ui.allocate_ui_with_layout(
            egui::vec2(SENSOR_WIDTH, h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                draw_sensor_list(app, ui, now);
            },
        );

        ui.add_space(8.0);

        // === Правая колонка: детальная панель ===
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                match app.performance_sensor {
                    Sensor::Cpu => draw_cpu_detail(app, ui, now),
                    Sensor::Memory => draw_memory_detail(app, ui, now),
                    Sensor::Disk(i) => draw_disk_detail(app, ui, i, now),
                    Sensor::Network => draw_placeholder(ui, "Сеть"),
                    Sensor::Nvidia => draw_placeholder(ui, "NVIDIA"),
                }
            },
        );
    });
}

// ============ Заголовок вкладки ============
fn draw_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("📈").size(20.0).color(ACCENT));
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("Производительность")
                .size(22.0)
                .strong()
                .color(egui::Color32::from_gray(220)),
        );
    });
}

// ============ Список датчиков слева ============
fn draw_sensor_list(app: &mut TmezApp, ui: &mut egui::Ui, now: f64) {
    let sensors = Sensor::build_list(app.disks.len());
    for sensor in sensors {
        let is_active = app.performance_sensor == sensor;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(SENSOR_WIDTH, 64.0),
            egui::Sense::click(),
        );

        if response.clicked() {
            app.performance_sensor = sensor;
        }

        let bg = if is_active {
            CARD_BG_ACTIVE
        } else if response.hovered() {
            CARD_BG_HOVER
        } else {
            CARD_BG
        };
        ui.painter().rect_filled(rect, 4.0, bg);
        ui.painter().rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0_f32, CARD_EDGE),
            egui::StrokeKind::Outside,
        );

        if is_active {
            let accent = egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(0.0, 6.0),
                egui::vec2(3.0, rect.height() - 12.0),
            );
            ui.painter().rect_filled(accent, 1.0, sensor.color());
        }

        // Иконка
        let icon_rect = egui::Rect::from_min_size(
            rect.left_top() + egui::vec2(12.0, 12.0),
            egui::vec2(20.0, 20.0),
        );
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            sensor_icon(&sensor),
            egui::FontId::proportional(16.0),
            sensor.color(),
        );

        // Название
        let title = sensor_title(app, &sensor);
        ui.painter().text(
            rect.left_top() + egui::vec2(40.0, 12.0),
            egui::Align2::LEFT_TOP,
            &title,
            egui::FontId::proportional(13.0),
            egui::Color32::from_gray(220),
        );

        // Подпись
        let subtitle = sensor_subtitle(app, &sensor);
        ui.painter().text(
            rect.left_top() + egui::vec2(40.0, 32.0),
            egui::Align2::LEFT_TOP,
            &subtitle,
            egui::FontId::proportional(10.0),
            egui::Color32::from_gray(160),
        );

        // Мини-график
        let mini_rect = egui::Rect::from_min_size(
            rect.right_top() + egui::vec2(-76.0, 8.0),
            egui::vec2(68.0, 48.0),
        );
        let history = sensor_history(app, &sensor);
        draw_mini_graph(ui, mini_rect, history, sensor.color(), now);
    }
}

// ============ Мини-график ============
fn draw_mini_graph(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    history: &[crate::widgets::Sample],
    color: egui::Color32,
    now: f64,
) {
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(8, 8, 12));

    if history.len() < 2 {
        return;
    }

    let right_x = rect.right() - 4.0;
    let bottom = rect.bottom() - 2.0;
    let h = rect.height() - 4.0;

    let to_screen = |t: f64, v: f32| -> egui::Pos2 {
        let x = right_x - ((now - t) / INTERVAL) as f32 * (STEP_PX * 0.4);
        let y = bottom - v.clamp(0.0, 1.0) * h;
        egui::pos2(x, y)
    };

    let mut pts: Vec<egui::Pos2> = history
        .iter()
        .map(|s| to_screen(s.time, s.value))
        .collect();

    let last_val = history.last().map(|s| s.value).unwrap_or(0.0);
    pts.push(to_screen(now, last_val));

    if pts.len() < 2 {
        return;
    }

    painter.add(egui::Shape::line(
        pts,
        egui::Stroke::new(1.2_f32, color),
    ));
}

// ============ Детальная панель CPU ============
fn draw_cpu_detail(app: &TmezApp, ui: &mut egui::Ui, now: f64) {
    egui::Frame::NONE
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0_f32, CARD_EDGE))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height());

            ui.label(
                egui::RichText::new(&app.cpu_brand)
                    .size(26.0)
                    .color(egui::Color32::from_gray(230)),
            );
            ui.add_space(8.0);

            draw_core_bar(ui, app);
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} логических процессора",
                        app.per_core_usage.len()
                    ))
                    .size(11.0)
                    .color(egui::Color32::from_gray(160)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("Загрузка по логическим процессорам")
                            .size(11.0)
                            .color(egui::Color32::from_gray(160)),
                    );
                });
            });

            ui.add_space(10.0);
            draw_cores_grid(ui, app, now);
            ui.add_space(12.0);
            draw_cpu_stats_table(ui, app);
        });
}

fn draw_core_bar(ui: &mut egui::Ui, app: &TmezApp) {
    let h = 28.0;
    let total_w = ui.available_width();
    let value_w = 90.0;
    let bar_w = (total_w - value_w - 8.0).max(100.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        let (bar_rect, _) = ui.allocate_exact_size(
            egui::vec2(bar_w, h),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(bar_rect);

        let n = app.per_core_smoothed.len().max(1);
        let seg_gap = 2.0;
        let seg_w = (bar_rect.width() - seg_gap * (n as f32 - 1.0)) / n as f32;

        for (i, &usage) in app.per_core_smoothed.iter().enumerate() {
            let x = bar_rect.left() + i as f32 * (seg_w + seg_gap);
            let seg_rect = egui::Rect::from_min_size(
                egui::pos2(x, bar_rect.top()),
                egui::vec2(seg_w, bar_rect.height()),
            );

            painter.rect_filled(seg_rect, 2.0, egui::Color32::from_rgb(20, 20, 28));

            let frac = usage.clamp(0.0, 1.0);
            if frac > 0.0 {
                let fill_w = seg_rect.width() * frac;
                let fill_rect = egui::Rect::from_min_size(
                    seg_rect.min,
                    egui::vec2(fill_w, seg_rect.height()),
                );
                painter.rect_filled(fill_rect, 2.0, gradient_color(usage));
            }
        }

        let (val_rect, _) = ui.allocate_exact_size(
            egui::vec2(value_w, h),
            egui::Sense::hover(),
        );
        ui.painter().text(
            val_rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            format!("{:.1}%", app.cpu_smoothed * 100.0),
            egui::FontId::proportional(24.0),
            egui::Color32::from_gray(230),
        );
    });
}

fn draw_cores_grid(ui: &mut egui::Ui, app: &TmezApp, now: f64) {
    let n = app.per_core_history.len();
    if n == 0 {
        return;
    }

    let avail_w = ui.available_width();
    let min_graph_w = 180.0;
    let gap = 8.0;
    let cols = ((avail_w + gap) / (min_graph_w + gap)).floor().max(1.0) as usize;
    let cols = cols.min(n);

    let graph_w = (avail_w - gap * (cols as f32 - 1.0)) / cols as f32;
    let graph_h = 120.0;

    let rows = (n + cols - 1) / cols;

    for row in 0..rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for col in 0..cols {
                let idx = row * cols + col;
                if idx >= n {
                    break;
                }

                ui.allocate_ui_with_layout(
                    egui::vec2(graph_w, graph_h + 18.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.label(
                            egui::RichText::new(format!("CPU {}", idx))
                                .size(11.0)
                                .color(egui::Color32::from_gray(180)),
                        );

                        let hist = &app.per_core_history[idx];
                        let live = app
                            .per_core_smoothed
                            .get(idx)
                            .copied()
                            .unwrap_or(0.0);
                        draw_core_graph(ui, hist, live, graph_w, graph_h, now);
                    },
                );
            }
        });
        ui.add_space(6.0);
    }
}

fn draw_core_graph(
    ui: &mut egui::Ui,
    history: &[crate::widgets::Sample],
    live: f32,
    w: f32,
    h: f32,
    now: f64,
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(w, h),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);

    painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(6, 6, 10));
    painter.rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 30, 40)),
        egui::StrokeKind::Inside,
    );

    let right_x = rect.right() - 4.0;
    let left_x = rect.left() + 2.0;
    let top = rect.top() + 4.0;
    let bottom = rect.bottom() - 4.0;
    let inner_h = bottom - top;

    let grid_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(20, 20, 30));
    for frac in [0.25_f32, 0.5, 0.75, 1.0] {
        let y = bottom - frac * inner_h;
        painter.line_segment(
            [egui::pos2(left_x, y), egui::pos2(rect.right(), y)],
            grid_stroke,
        );
    }

    if history.len() < 2 {
        return;
    }

    let to_screen = |t: f64, v: f32| -> egui::Pos2 {
        let x = right_x - ((now - t) / INTERVAL) as f32 * (STEP_PX * 0.4);
        let y = bottom - v.clamp(0.0, 1.0) * inner_h;
        egui::pos2(x, y)
    };

    let mut pts: Vec<egui::Pos2> = history
        .iter()
        .map(|s| to_screen(s.time, s.value))
        .collect();
    pts.push(to_screen(now, live));

    if pts.len() < 2 {
        return;
    }

    let thickness = 2.5_f32;
    let mut mesh = egui::Mesh::default();
    for (i, p) in pts.iter().enumerate() {
        let v = ((bottom - p.y) / inner_h).clamp(0.0, 1.0);
        let color = gradient_color(v);
        mesh.colored_vertex(*p, color);
        mesh.colored_vertex(egui::pos2(p.x, p.y + thickness), color);
        if i + 1 < pts.len() {
            let n = i + 1;
            mesh.add_triangle(i as u32 * 2, i as u32 * 2 + 1, n as u32 * 2);
            mesh.add_triangle(n as u32 * 2, i as u32 * 2 + 1, n as u32 * 2 + 1);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn draw_cpu_stats_table(ui: &mut egui::Ui, app: &TmezApp) {
    let total_processes = app.system.processes().len();
    let uptime_secs = sysinfo::System::uptime();
    let uptime_str = format_uptime(uptime_secs);
    let physical = app.system.physical_core_count().unwrap_or(0);
    let logical = app.system.cpus().len();
    let base_speed = if !app.system.cpus().is_empty() {
        app.system.cpus()[0].frequency() as f32 / 1000.0
    } else {
        0.0
    };

    let virtualization = if app.cpu_static.virtualization_enabled {
        "Включена"
    } else {
        "Отключена"
    };

    let l2 = if app.cpu_static.l2_cache_kb > 0 {
        format_cache(app.cpu_static.l2_cache_kb)
    } else {
        "—".to_string()
    };
    let l3 = if app.cpu_static.l3_cache_kb > 0 {
        format_cache(app.cpu_static.l3_cache_kb)
    } else {
        "—".to_string()
    };

    egui::Grid::new("cpu_stats_grid")
        .num_columns(4)
        .spacing([24.0, 4.0])
        .striped(false)
        .show(ui, |ui| {
            let label_color = egui::Color32::from_gray(150);
            let value_color = egui::Color32::from_gray(210);

            label(ui, "Загрузка", label_color);
            label(ui, &format!("{:.1}%", app.cpu_smoothed * 100.0), value_color);
            label(ui, "Базовая частота", label_color);
            label(ui, &format!("{:.2} ГГц", base_speed), value_color);
            ui.end_row();

            label(ui, "Текущая частота", label_color);
            label(ui, &format!("{:.2} ГГц", app.cpu_freq_ghz), value_color);
            label(ui, "Сокеты", label_color);
            label(ui, "1", value_color);
            ui.end_row();

            label(ui, "Процессы", label_color);
            label(ui, &total_processes.to_string(), value_color);
            label(ui, "Виртуализация", label_color);
            label(ui, virtualization, value_color);
            ui.end_row();

            label(ui, "Потоки", label_color);
            label(ui, "—", value_color);
            label(ui, "Кэш L1", label_color);
            label(ui, "—", value_color);
            ui.end_row();

            label(ui, "Время работы", label_color);
            label(ui, &uptime_str, value_color);
            label(ui, "Кэш L2", label_color);
            label(ui, &l2, value_color);
            ui.end_row();

            label(ui, "Логических процессоров", label_color);
            label(ui, &logical.to_string(), value_color);
            label(ui, "Кэш L3", label_color);
            label(ui, &l3, value_color);
            ui.end_row();

            label(ui, "Физических ядер", label_color);
            label(ui, &physical.to_string(), value_color);
            label(ui, "Дескрипторы", label_color);
            label(ui, "—", value_color);
            ui.end_row();
        });
}

// ============ Детальная панель памяти ============
fn draw_memory_detail(app: &TmezApp, ui: &mut egui::Ui, now: f64) {
    let mem_color = MEM_COLOR;

    egui::Frame::NONE
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0_f32, CARD_EDGE))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height());

            let mem_type = if app.mem_static.memory_type.is_empty() {
                String::new()
            } else {
                format!(" {}", app.mem_static.memory_type)
            };
            let title = format!("{:.1} ГБ{}", app.mem_total_gb, mem_type);
            ui.label(
                egui::RichText::new(title)
                    .size(26.0)
                    .color(egui::Color32::from_gray(230)),
            );
            ui.add_space(8.0);

            draw_memory_bar(ui, app, mem_color);
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new("Использование памяти")
                    .size(11.0)
                    .color(egui::Color32::from_gray(160)),
            );

            ui.add_space(10.0);
            let graph_h = 200.0;
            draw_mem_graph(ui, app, mem_color, graph_h, now);
            ui.add_space(12.0);
            draw_memory_stats_table(ui, app);
        });
}

fn draw_memory_bar(ui: &mut egui::Ui, app: &TmezApp, color: egui::Color32) {
    let h = 28.0;
    let total_w = ui.available_width();
    let value_w = 90.0;
    let bar_w = (total_w - value_w - 8.0).max(100.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        let (bar_rect, _) = ui.allocate_exact_size(
            egui::vec2(bar_w, h),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(bar_rect);

        painter.rect_filled(bar_rect, 3.0, egui::Color32::from_rgb(20, 20, 28));

        let frac = app.mem_smoothed.clamp(0.0, 1.0);
        if frac > 0.0 {
            let fill_w = bar_rect.width() * frac;
            let fill_rect = egui::Rect::from_min_size(
                bar_rect.min,
                egui::vec2(fill_w, bar_rect.height()),
            );
            painter.rect_filled(fill_rect, 3.0, color);
        }

        let (val_rect, _) = ui.allocate_exact_size(
            egui::vec2(value_w, h),
            egui::Sense::hover(),
        );
        ui.painter().text(
            val_rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            format!("{:.1}%", app.mem_smoothed * 100.0),
            egui::FontId::proportional(24.0),
            egui::Color32::from_gray(230),
        );
    });
}

fn draw_mem_graph(ui: &mut egui::Ui, app: &TmezApp, color: egui::Color32, h: f32, now: f64) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), h),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(6, 6, 10));
    painter.rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 30, 40)),
        egui::StrokeKind::Inside,
    );

    let right_x = rect.right() - 8.0;
    let left_x = rect.left() + 4.0;
    let top = rect.top() + 6.0;
    let bottom = rect.bottom() - 6.0;
    let inner_h = bottom - top;

    let grid_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(20, 20, 30));
    for frac in [0.25_f32, 0.5, 0.75, 1.0] {
        let y = bottom - frac * inner_h;
        painter.line_segment(
            [egui::pos2(left_x, y), egui::pos2(rect.right(), y)],
            grid_stroke,
        );
    }

    if app.mem_history.len() < 2 {
        return;
    }

    let to_screen = |t: f64, v: f32| -> egui::Pos2 {
        let x = right_x - ((now - t) / INTERVAL) as f32 * (STEP_PX * 0.6);
        let y = bottom - v.clamp(0.0, 1.0) * inner_h;
        egui::pos2(x, y)
    };

    let mut pts: Vec<egui::Pos2> = app
        .mem_history
        .iter()
        .map(|s| to_screen(s.time, s.value))
        .collect();
    pts.push(to_screen(now, app.mem_smoothed));

    if pts.len() < 2 {
        return;
    }

    let fill_color = egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        60,
    );
    let mut mesh = egui::Mesh::default();
    for p in &pts {
        mesh.colored_vertex(*p, fill_color);
        mesh.colored_vertex(egui::pos2(p.x, bottom), fill_color);
    }
    for i in 0..pts.len() - 1 {
        let i0 = (i * 2) as u32;
        let i1 = i0 + 1;
        let i2 = i0 + 2;
        let i3 = i0 + 3;
        mesh.add_triangle(i0, i1, i2);
        mesh.add_triangle(i1, i2, i3);
    }
    painter.add(egui::Shape::mesh(mesh));

    painter.add(egui::Shape::line(
        pts,
        egui::Stroke::new(1.5_f32, color),
    ));
}

fn draw_memory_stats_table(ui: &mut egui::Ui, app: &TmezApp) {
    let available = app.mem_total_gb - app.mem_used_gb;
    let swap_avail = app.swap_total_gb - app.swap_used_gb;

    let speed_str = if app.mem_static.speed_mts > 0 {
        format!("{} MT/s", app.mem_static.speed_mts)
    } else {
        "—".to_string()
    };

    let form_str = if app.mem_static.form_factor.is_empty() {
        "—".to_string()
    } else {
        app.mem_static.form_factor.clone()
    };

    let type_str = if app.mem_static.memory_type.is_empty() {
        "—".to_string()
    } else {
        app.mem_static.memory_type.clone()
    };

    let slots_str = if app.mem_static.slots_total > 0 {
        format!("{} из {}", app.mem_static.slots_used, app.mem_static.slots_total)
    } else {
        "—".to_string()
    };

    egui::Grid::new("memory_stats_grid")
        .num_columns(4)
        .spacing([24.0, 4.0])
        .striped(false)
        .show(ui, |ui| {
            let label_color = egui::Color32::from_gray(150);
            let value_color = egui::Color32::from_gray(210);

            label(ui, "Использовано", label_color);
            label(ui, &format!("{:.2} ГБ", app.mem_used_gb), value_color);
            label(ui, "Доступно", label_color);
            label(ui, &format!("{:.2} ГБ", available), value_color);
            ui.end_row();

            label(ui, "Файл подкачки (исп.)", label_color);
            label(ui, &format!("{:.2} ГБ", app.swap_used_gb), value_color);
            label(ui, "Файл подкачки (дост.)", label_color);
            label(ui, &format!("{:.2} ГБ", swap_avail), value_color);
            ui.end_row();

            label(ui, "Кэш", label_color);
            label(ui, "—", value_color);
            label(ui, "Скорость", label_color);
            label(ui, &speed_str, value_color);
            ui.end_row();

            label(ui, "Форм-фактор", label_color);
            label(ui, &form_str, value_color);
            label(ui, "Тип", label_color);
            label(ui, &type_str, value_color);
            ui.end_row();

            label(ui, "Занято слотов", label_color);
            label(ui, &slots_str, value_color);
            label(ui, "Всего", label_color);
            label(ui, &format!("{:.1} ГБ", app.mem_total_gb), value_color);
            ui.end_row();
        });
}

// ============ Детальная панель диска (заглушка) ============
fn draw_disk_detail(app: &TmezApp, ui: &mut egui::Ui, idx: usize, now: f64) {
    let list: Vec<_> = app.disks.iter().collect();
    let Some(d) = list.get(idx) else {
        draw_placeholder(ui, "Диск");
        return;
    };

    let mount = d.mount_point().to_string_lossy().to_string();
    let fs = d.file_system().to_string_lossy().to_string();
    let kind = match d.kind() {
        sysinfo::DiskKind::HDD => "HDD",
        sysinfo::DiskKind::SSD => "SSD",
        _ => "—",
    };

    // Читаем СГЛАЖЕННЫЕ значения для отображения
    let activity = app.per_disk_activity_smoothed.get(idx).copied().unwrap_or(0.0);
    let rate_live = app.per_disk_rate_smoothed.get(idx).copied().unwrap_or(0.0);

    // Сырые — только для таблицы
    let read_bps = app.per_disk_read_bps.get(idx).copied().unwrap_or(0.0);
    let write_bps = app.per_disk_write_bps.get(idx).copied().unwrap_or(0.0);

    egui::Frame::NONE
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0_f32, CARD_EDGE))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height());

            // === 1. Заголовок с моделью ===
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&mount)
                        .size(26.0)
                        .color(egui::Color32::from_gray(230)),
                );

                let letter = mount.chars().next();
                if let Some(c) = letter {
                    if let Some(model) = app.disk_models.get(&c) {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(model)
                                    .size(14.0)
                                    .color(egui::Color32::from_gray(180)),
                            );
                        });
                    }
                }
            });
            ui.add_space(8.0);

            // === 2. Полоса активности ===
            draw_disk_activity_bar(ui, activity, DISK_COLOR);
            ui.add_space(6.0);

            // === 3. Подпись ===
            ui.label(
                egui::RichText::new(format!("Логический том · {} · {}", fs, kind))
                    .size(11.0)
                    .color(egui::Color32::from_gray(160)),
            );

            ui.add_space(10.0);

            // === 4. График % Активности ===
            ui.label(
                egui::RichText::new("% Активности")
                    .size(11.0)
                    .color(egui::Color32::from_gray(180)),
            );
            ui.add_space(4.0);
            let history = app
                .per_disk_history
                .get(idx)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            draw_disk_graph(ui, history, activity, DISK_COLOR, 140.0, now);

            ui.add_space(10.0);

            // === 5. График скорости передачи ===
            ui.label(
                egui::RichText::new("Скорость передачи")
                    .size(11.0)
                    .color(egui::Color32::from_gray(180)),
            );
            ui.add_space(4.0);
            let rate_history = app
                .per_disk_rate_history
                .get(idx)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            draw_disk_graph(ui, rate_history, rate_live, DISK_COLOR, 140.0, now);

            ui.add_space(12.0);

            // === 6. Нижняя таблица ===
            draw_disk_stats_table(ui, app, idx, d, read_bps, write_bps, activity);
        });
}

/// Полоса активности диска — цельная, зелёная.
fn draw_disk_activity_bar(ui: &mut egui::Ui, activity: f32, color: egui::Color32) {
    let h = 28.0;
    let total_w = ui.available_width();
    let value_w = 90.0;
    let bar_w = (total_w - value_w - 8.0).max(100.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        let (bar_rect, _) = ui.allocate_exact_size(
            egui::vec2(bar_w, h),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(bar_rect);

        painter.rect_filled(bar_rect, 3.0, egui::Color32::from_rgb(20, 20, 28));

        let frac = activity.clamp(0.0, 1.0);
        if frac > 0.0 {
            let fill_w = bar_rect.width() * frac;
            let fill_rect = egui::Rect::from_min_size(
                bar_rect.min,
                egui::vec2(fill_w, bar_rect.height()),
            );
            painter.rect_filled(fill_rect, 3.0, color);
        }

        let (val_rect, _) = ui.allocate_exact_size(
            egui::vec2(value_w, h),
            egui::Sense::hover(),
        );
        ui.painter().text(
            val_rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            format!("{:.1}%", activity * 100.0),
            egui::FontId::proportional(24.0),
            egui::Color32::from_gray(230),
        );
    });
}

/// График диска (используется и для активности, и для скорости).
fn draw_disk_graph(
    ui: &mut egui::Ui,
    history: &[crate::widgets::Sample],
    live: f32,
    color: egui::Color32,
    h: f32,
    now: f64,
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), h),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);

    painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(6, 6, 10));
    painter.rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 30, 40)),
        egui::StrokeKind::Inside,
    );

    let right_x = rect.right() - 8.0;
    let left_x = rect.left() + 4.0;
    let top = rect.top() + 6.0;
    let bottom = rect.bottom() - 6.0;
    let inner_h = bottom - top;

    // Горизонтальная сетка
    let grid_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(20, 20, 30));
    for frac in [0.25_f32, 0.5, 0.75, 1.0] {
        let y = bottom - frac * inner_h;
        painter.line_segment(
            [egui::pos2(left_x, y), egui::pos2(rect.right(), y)],
            grid_stroke,
        );
    }

    if history.len() < 2 {
        return;
    }

    let to_screen = |t: f64, v: f32| -> egui::Pos2 {
        let x = right_x - ((now - t) / INTERVAL) as f32 * (STEP_PX * 0.6);
        let y = bottom - v.clamp(0.0, 1.0) * inner_h;
        egui::pos2(x, y)
    };

    let mut pts: Vec<egui::Pos2> = history
        .iter()
        .map(|s| to_screen(s.time, s.value))
        .collect();
    pts.push(to_screen(now, live));

    if pts.len() < 2 {
        return;
    }

    // Градиент по высоте (зелёный → красный), как в tmog
    let thickness = 3.0_f32;
    let mut mesh = egui::Mesh::default();
    for (i, p) in pts.iter().enumerate() {
        let v = ((bottom - p.y) / inner_h).clamp(0.0, 1.0);
        let grad_color = gradient_color(v);
        // Смешиваем цвет датчика с градиентом — но проще использовать градиент
        let _ = color;
        mesh.colored_vertex(*p, grad_color);
        mesh.colored_vertex(egui::pos2(p.x, p.y + thickness), grad_color);
        if i + 1 < pts.len() {
            let n = i + 1;
            mesh.add_triangle(i as u32 * 2, i as u32 * 2 + 1, n as u32 * 2);
            mesh.add_triangle(n as u32 * 2, i as u32 * 2 + 1, n as u32 * 2 + 1);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Нижняя таблица характеристик диска.
fn draw_disk_stats_table(
    ui: &mut egui::Ui,
    app: &TmezApp,
    idx: usize,
    d: &sysinfo::Disk,
    read_bps: f64,
    write_bps: f64,
    activity: f32,
) {
    let usage = d.usage();
    let total_read = usage.read_bytes;
    let total_write = usage.written_bytes;
    let capacity = d.total_space();
    let fs = d.file_system().to_string_lossy().to_string();

    let kind = match d.kind() {
        sysinfo::DiskKind::HDD => "HDD",
        sysinfo::DiskKind::SSD => "SSD",
        _ => "—",
    };

    // System disk = тот, чей mount_point начинается с "C:" или "/" на Linux
    let is_system = d.mount_point().to_string_lossy().starts_with("C:");

    egui::Grid::new(format!("disk_stats_grid_{}", idx))
        .num_columns(4)
        .spacing([24.0, 4.0])
        .striped(false)
        .show(ui, |ui| {
            let lc = egui::Color32::from_gray(150);
            let vc = egui::Color32::from_gray(210);

            // Строка 1
            label(ui, "Скорость чтения", lc);
            label(ui, &fmt_bps(read_bps), vc);
            label(ui, "Скорость записи", lc);
            label(ui, &fmt_bps(write_bps), vc);
            ui.end_row();

            // Строка 2
            label(ui, "Всего прочитано", lc);
            label(ui, &fmt_bytes(total_read), vc);
            label(ui, "Всего записано", lc);
            label(ui, &fmt_bytes(total_write), vc);
            ui.end_row();

            // Строка 3
            label(ui, "Активность", lc);
            label(ui, &format!("{:.1}%", activity * 100.0), vc);
            label(ui, "Ср. время отклика", lc);
            label(ui, "—", vc);
            ui.end_row();

            // Строка 4
            label(ui, "Объём", lc);
            label(ui, &fmt_bytes(capacity), vc);
            label(ui, "Отформатировано", lc);
            label(ui, &fmt_bytes(capacity), vc);
            ui.end_row();

            // Строка 5
            label(ui, "Системный диск", lc);
            label(ui, if is_system { "Да" } else { "Нет" }, vc);
            label(ui, "Тип", lc);
            label(ui, &format!("{} · {}", kind, fs), vc);
            ui.end_row();
        });

    // Тихий warning про app, чтобы не удалять параметр
    let _ = app;
}

/// Форматирует байты в КБ/МБ/ГБ/ТБ.
fn fmt_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} ТБ", b / TB)
    } else if b >= GB {
        format!("{:.1} ГБ", b / GB)
    } else if b >= MB {
        format!("{:.1} МБ", b / MB)
    } else if b >= KB {
        format!("{:.1} КБ", b / KB)
    } else {
        format!("{} Б", bytes)
    }
}

// ============ Заглушка ============
fn draw_placeholder(ui: &mut egui::Ui, title: &str) {
    egui::Frame::NONE
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0_f32, CARD_EDGE))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height());

            ui.label(
                egui::RichText::new(title)
                    .size(28.0)
                    .color(egui::Color32::from_gray(220)),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("🚧  Детальная панель — следующий этап")
                    .size(14.0)
                    .color(egui::Color32::from_gray(140)),
            );
        });
}

// ============ Хелперы ============
fn sensor_icon(s: &Sensor) -> &'static str {
    match s {
        Sensor::Cpu => "🔲",
        Sensor::Memory => "▦",
        Sensor::Disk(_) => "◉",
        Sensor::Network => "⇄",
        Sensor::Nvidia => "▣",
    }
}

fn sensor_title(app: &TmezApp, s: &Sensor) -> String {
    match s {
        Sensor::Cpu => "ЦП".to_string(),
        Sensor::Memory => "Память".to_string(),
        Sensor::Disk(i) => {
            let list: Vec<_> = app.disks.iter().collect();
            if let Some(d) = list.get(*i) {
                d.mount_point().to_string_lossy().to_string()
            } else {
                format!("Диск {}", i)
            }
        }
        Sensor::Network => "Сеть".to_string(),
        Sensor::Nvidia => "NVIDIA".to_string(),
    }
}

fn sensor_subtitle(app: &TmezApp, s: &Sensor) -> String {
    match s {
        Sensor::Cpu => format!(
            "{:.1}% · {} лог. процессоров",
            app.cpu_smoothed * 100.0,
            app.system.cpus().len()
        ),
        Sensor::Memory => format!(
            "{:.1} ГБ / {:.1} ГБ · {:.0}%",
            app.mem_used_gb,
            app.mem_total_gb,
            app.mem_smoothed * 100.0
        ),
        Sensor::Disk(i) => {
            let list: Vec<_> = app.disks.iter().collect();
            if let Some(d) = list.get(*i) {
                let kind = match d.kind() {
                    sysinfo::DiskKind::HDD => "HDD",
                    sysinfo::DiskKind::SSD => "SSD",
                    _ => "—",
                };
                let used_pct = if d.total_space() > 0 {
                    (1.0 - d.available_space() as f64 / d.total_space() as f64) * 100.0
                } else {
                    0.0
                };
                format!("{} · {:.0}% занято", kind, used_pct)
            } else {
                "—".to_string()
            }
        }
        Sensor::Network => {
            let total = app.net_rx_bps + app.net_tx_bps;
            format!("Все адаптеры · {}", fmt_bps(total))
        }
        Sensor::Nvidia => format!("GPU adapter · {:.0}%", app.gpu_nvidia_load * 100.0),
    }
}

fn sensor_history<'a>(app: &'a TmezApp, s: &Sensor) -> &'a [crate::widgets::Sample] {
    match s {
        Sensor::Cpu => &app.cpu_history,
        Sensor::Memory => &app.mem_history,
        Sensor::Disk(i) => app
            .per_disk_history
            .get(*i)
            .map(|v| v.as_slice())
            .unwrap_or(&[]),
        Sensor::Network => &app.net_history,
        Sensor::Nvidia => &app.nvidia_history,
    }
}

fn label(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.label(egui::RichText::new(text).size(11.0).color(color));
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let sec = secs % 60;
    if days > 0 {
        format!("{}д {}ч {}м {}с", days, hours, mins, sec)
    } else if hours > 0 {
        format!("{}ч {}м {}с", hours, mins, sec)
    } else {
        format!("{}м {}с", mins, sec)
    }
}

fn format_cache(kb: u64) -> String {
    if kb >= 1024 {
        format!("{:.1} MB", kb as f64 / 1024.0)
    } else {
        format!("{} KB", kb)
    }
}

fn fmt_bps(bps: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bps >= GB {
        format!("{:.1} ГБ/с", bps / GB)
    } else if bps >= MB {
        format!("{:.1} МБ/с", bps / MB)
    } else if bps >= KB {
        format!("{:.0} КБ/с", bps / KB)
    } else {
        format!("{:.0} Б/с", bps)
    }
}