use eframe::egui;
use sysinfo::System;
use nvml_wrapper::Nvml;

use crate::theme::{INTERVAL, MAX_POINTS, TAU, ACCENT};
use crate::tabs::{self, Tab};
use crate::tabs::summary as summary_tab;
use crate::tabs::processes as processes_tab;
use crate::widgets::Sample;
use crate::processes::{collect_top, ProcessRow};

pub struct TmezApp {
    pub system: System,

    // CPU
    pub cpu_raw: f32,
    pub cpu_smoothed: f32,
    pub cpu_history: Vec<Sample>,
    pub cpu_freq_ghz: f32,
    pub cpu_brand: String,

    // Память
    pub mem_raw: f32,
    pub mem_smoothed: f32,
    pub mem_used_gb: f64,
    pub mem_total_gb: f64,

    // GPU (NVIDIA)
    pub nvml: Option<Nvml>,
    pub gpu_nvidia_load: f32,

    // Тайминги
    pub last_add: f64,
    pub last_render: f64,
    pub started: bool,

    pub top_processes: Vec<ProcessRow>,

    pub current_tab: Tab,
}

impl TmezApp {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let total = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used = system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let mem_pct = if total > 0.0 { (used / total) as f32 } else { 0.0 };

        let cpu_brand = system
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "CPU".to_string());

        // Пытаемся инициализировать NVML. Если NVIDIA нет или драйвер
        // не отвечает — получим None, и фича просто отключится.
        let nvml = Nvml::init().ok();

        Self {
            system,

            cpu_raw: 0.0,
            cpu_smoothed: 0.0,
            cpu_history: Vec::with_capacity(MAX_POINTS),
            cpu_freq_ghz: 0.0,
            cpu_brand,

            mem_raw: mem_pct,
            mem_smoothed: mem_pct,
            mem_used_gb: used,
            mem_total_gb: total,

            nvml,
            gpu_nvidia_load: 0.0,

            last_add: 0.0,
            last_render: 0.0,
            started: false,

            top_processes: Vec::new(),

            current_tab: Tab::Summary,
        }
    }

    pub fn read_cpu(&mut self) -> f32 {
        self.system.refresh_cpu_usage();
        self.system.global_cpu_usage() / 100.0
    }

    /// Текущая частота CPU (среднее по всем ядрам) в ГГц.
    pub fn read_cpu_freq(&self) -> f32 {
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return 0.0;
        }
        let sum_mhz: u64 = cpus.iter().map(|c| c.frequency()).sum();
        (sum_mhz as f32 / cpus.len() as f32) / 1000.0
    }

    pub fn read_mem(&mut self) -> (f32, f64, f64) {
        self.system.refresh_memory();
        let total = self.system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used = self.system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let pct = if total > 0.0 { (used / total) as f32 } else { 0.0 };
        (pct, used, total)
    }

    /// Загрузка дискретной NVIDIA GPU, 0.0..1.0.
    /// Если NVML не инициализирован — возвращает 0.0.
    pub fn read_gpu_nvidia(&self) -> f32 {
        let Some(nvml) = &self.nvml else { return 0.0; };
        let Ok(device) = nvml.device_by_index(0) else { return 0.0; };
        match device.utilization_rates() {
            Ok(rates) => rates.gpu as f32 / 100.0,
            Err(_) => 0.0,
        }
    }

    fn draw_nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("TMEZ")
                    .size(22.0)
                    .strong()
                    .color(ACCENT),
            );
        });
        ui.add_space(14.0);

        for &tab in Tab::all() {
            let selected = self.current_tab == tab;
            let height = 34.0;
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), height),
                egui::Sense::click(),
            );

            if response.clicked() {
                self.current_tab = tab;
            }

            let bg = if selected {
                egui::Color32::from_rgb(40, 90, 160)
            } else if response.hovered() {
                egui::Color32::from_rgb(28, 28, 40)
            } else {
                egui::Color32::TRANSPARENT
            };

            ui.painter().rect_filled(rect, 4.0, bg);

            if selected {
                let accent = egui::Rect::from_min_size(
                    rect.left_top() + egui::vec2(0.0, 4.0),
                    egui::vec2(3.0, rect.height() - 8.0),
                );
                ui.painter().rect_filled(accent, 1.0, ACCENT);
            }

            let text_color = if selected {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(200, 200, 210)
            };

            ui.painter().text(
                rect.left_center() + egui::vec2(14.0, 0.0),
                egui::Align2::LEFT_CENTER,
                tab.label(),
                egui::FontId::proportional(14.0),
                text_color,
            );
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("v0.1.0 · Free")
                        .size(11.0)
                        .color(egui::Color32::from_gray(120)),
                );
            });
        });
    }
}

impl eframe::App for TmezApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);

        if !self.started {
            self.started = true;
            self.last_add = now;
            self.last_render = now;

            self.cpu_raw = self.read_cpu();
            self.cpu_freq_ghz = self.read_cpu_freq();
            self.gpu_nvidia_load = self.read_gpu_nvidia();

            let (pct, used, total) = self.read_mem();
            self.mem_raw = pct;
            self.mem_used_gb = used;
            self.mem_total_gb = total;
            self.cpu_smoothed = self.cpu_raw;
            self.mem_smoothed = self.mem_raw;

            self.top_processes = collect_top(&mut self.system, 50);

            self.cpu_history.push(Sample {
                value: self.cpu_smoothed,
                time: self.last_add,
            });
        }

        // Сглаживание
        let dt = (now - self.last_render).clamp(0.0, 0.1);
        self.last_render = now;
        let alpha = (1.0 - (-dt / TAU).exp()) as f32;
        self.cpu_smoothed += (self.cpu_raw - self.cpu_smoothed) * alpha;
        self.mem_smoothed += (self.mem_raw - self.mem_smoothed) * alpha;

        // Новые сэмплы раз в INTERVAL
        while now - self.last_add >= INTERVAL {
            self.last_add += INTERVAL;

            self.cpu_raw = self.read_cpu();
            self.cpu_freq_ghz = self.read_cpu_freq();
            self.gpu_nvidia_load = self.read_gpu_nvidia();

            let (pct, used, total) = self.read_mem();
            self.mem_raw = pct;
            self.mem_used_gb = used;
            self.mem_total_gb = total;

            if self.cpu_history.len() >= MAX_POINTS {
                self.cpu_history.remove(0);
            }
            self.cpu_history.push(Sample {
                value: self.cpu_smoothed,
                time: self.last_add,
            });
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(16));

        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(220.0)
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(12, 12, 16))
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                self.draw_nav(ui);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(10, 10, 14))
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(ctx, |ui| match self.current_tab {
                Tab::Summary => summary_tab::draw(self, ui, now),
                Tab::Processes => processes_tab::draw(ui),
                Tab::Performance => tabs::draw_placeholder(ui, "Производительность"),
                Tab::SystemInfo => tabs::draw_placeholder(ui, "Система"),
                Tab::StartupApps => tabs::draw_placeholder(ui, "Автозагрузка"),
                Tab::Users => tabs::draw_placeholder(ui, "Пользователи"),
                Tab::Services => tabs::draw_placeholder(ui, "Службы"),
                Tab::Settings => tabs::draw_placeholder(ui, "Настройки"),
            });
    }
}