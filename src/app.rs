use eframe::egui;
use sysinfo::System;
use nvml_wrapper::Nvml;

use crate::theme::{INTERVAL, MAX_POINTS, TAU, ACCENT};
use crate::tabs::{self, Tab};
use crate::tabs::summary as summary_tab;
use crate::tabs::processes as processes_tab;
use crate::tabs::performance as performance_tab;
use crate::tabs::performance::Sensor;
use crate::widgets::Sample;
use crate::processes::{collect_top, ProcessRow};
use crate::system_info::{
    query_cpu_static_info, query_memory_static_info, query_disk_models_by_letter,
    CpuStaticInfo, MemoryStaticInfo,
};

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

    pub last_disk_refresh: f64,
    pub last_net_refresh: f64,

    pub top_processes: Vec<ProcessRow>,

    // Сеть
    pub net_rx_bps: f64,
    pub net_tx_bps: f64,
    pub net_rx_total: u64,
    pub net_tx_total: u64,
    pub prev_net_rx: u64,
    pub prev_net_tx: u64,
    pub networks: sysinfo::Networks,

    // Диски
    pub disk_read_bps: f64,
    pub disk_write_bps: f64,
    pub disk_activity: f32,
    pub prev_disk_read: u64,
    pub prev_disk_write: u64,
    pub disks: sysinfo::Disks,

    // GPU (расширение NVML)
    pub gpu_temp_c: f32,

    pub performance_sensor: Sensor,

    pub mem_history: Vec<Sample>,
    pub disk_history: Vec<Sample>,
    pub net_history: Vec<Sample>,
    pub nvidia_history: Vec<Sample>,

    pub cpu_static: CpuStaticInfo,

    pub per_core_usage: Vec<f32>,
    pub per_core_history: Vec<Vec<Sample>>,
    pub per_core_smoothed: Vec<f32>,

    pub mem_static: MemoryStaticInfo,

    // Swap
    pub swap_used_gb: f64,
    pub swap_total_gb: f64,

    pub per_disk_read_bps: Vec<f64>,
    pub per_disk_write_bps: Vec<f64>,
    pub per_disk_activity: Vec<f32>,
    pub per_disk_history: Vec<Vec<Sample>>,
    pub per_disk_prev_read: Vec<u64>,
    pub per_disk_prev_write: Vec<u64>,

    pub per_disk_rate_history: Vec<Vec<Sample>>,

    pub disk_models: std::collections::HashMap<char, String>,

    pub per_disk_activity_smoothed: Vec<f32>,
    pub per_disk_rate_smoothed: Vec<f32>,

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

        let nvml = Nvml::init().ok();

        // --- Per-core подготовка (ДО Self) ---
        let num_cores = system.cpus().len();
        let per_core_usage: Vec<f32> = vec![0.0; num_cores];
        let per_core_smoothed: Vec<f32> = vec![0.0; num_cores];
        let per_core_history: Vec<Vec<Sample>> = (0..num_cores)
            .map(|_| Vec::with_capacity(MAX_POINTS))
            .collect();

        // --- Диски подготовка (ДО Self) ---
        let mut disks = sysinfo::Disks::new_with_refreshed_list();
        disks.refresh(true);
        let num_disks = disks.len();

        let per_disk_read_bps = vec![0.0_f64; num_disks];
        let per_disk_write_bps = vec![0.0_f64; num_disks];
        let per_disk_activity = vec![0.0_f32; num_disks];
        let per_disk_prev_read = vec![0u64; num_disks];
        let per_disk_prev_write = vec![0u64; num_disks];
        let per_disk_history: Vec<Vec<Sample>> = (0..num_disks)
            .map(|_| Vec::with_capacity(MAX_POINTS))
            .collect();
        let per_disk_rate_history: Vec<Vec<Sample>> = (0..num_disks)
            .map(|_| Vec::with_capacity(MAX_POINTS))
            .collect();

        let per_disk_activity_smoothed = vec![0.0_f32; num_disks];
        let per_disk_rate_smoothed = vec![0.0_f32; num_disks];

        let disk_models = query_disk_models_by_letter();

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

            last_disk_refresh: 0.0,
            last_net_refresh: 0.0,

            top_processes: Vec::new(),

            networks: sysinfo::Networks::new_with_refreshed_list(),
            net_rx_bps: 0.0,
            net_tx_bps: 0.0,
            net_rx_total: 0,
            net_tx_total: 0,
            prev_net_rx: 0,
            prev_net_tx: 0,

            disks,
            disk_read_bps: 0.0,
            disk_write_bps: 0.0,
            disk_activity: 0.0,
            prev_disk_read: 0,
            prev_disk_write: 0,

            gpu_temp_c: 0.0,

            performance_sensor: Sensor::Cpu,

            mem_history: Vec::with_capacity(MAX_POINTS),
            disk_history: Vec::with_capacity(MAX_POINTS),
            net_history: Vec::with_capacity(MAX_POINTS),
            nvidia_history: Vec::with_capacity(MAX_POINTS),

            cpu_static: query_cpu_static_info(),

            per_core_usage,
            per_core_history,
            per_core_smoothed,

            mem_static: query_memory_static_info(),

            swap_used_gb: 0.0,
            swap_total_gb: 0.0,

            per_disk_read_bps,
            per_disk_write_bps,
            per_disk_activity,
            per_disk_history,
            per_disk_prev_read,
            per_disk_prev_write,

            per_disk_rate_history,

            disk_models,

            per_disk_activity_smoothed,
            per_disk_rate_smoothed,

            current_tab: Tab::Summary,
        }
    }

    pub fn read_cpu(&mut self) -> f32 {
        self.system.refresh_cpu_usage();
        self.system.global_cpu_usage() / 100.0
    }

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

        let swap_total = self.system.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
        let swap_used = self.system.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
        self.swap_total_gb = swap_total;
        self.swap_used_gb = swap_used;

        (pct, used, total)
    }

    pub fn read_gpu_nvidia(&self) -> f32 {
        let Some(nvml) = &self.nvml else { return 0.0; };
        let Ok(device) = nvml.device_by_index(0) else { return 0.0; };
        match device.utilization_rates() {
            Ok(rates) => rates.gpu as f32 / 100.0,
            Err(_) => 0.0,
        }
    }

    pub fn update_networks(&mut self) {
        let now = self.last_add;
        if self.last_net_refresh > 0.0 && now - self.last_net_refresh < 1.0 {
            return;
        }

        let dt_real = if self.last_net_refresh > 0.0 {
            (now - self.last_net_refresh).max(0.001)
        } else {
            1.0
        };
        self.last_net_refresh = now;

        self.networks.refresh(true);

        let rx: u64 = self.networks.iter().map(|(_, data)| data.total_received()).sum();
        let tx: u64 = self.networks.iter().map(|(_, data)| data.total_transmitted()).sum();

        if self.prev_net_rx > 0 {
            self.net_rx_bps = rx.saturating_sub(self.prev_net_rx) as f64 / dt_real;
            self.net_tx_bps = tx.saturating_sub(self.prev_net_tx) as f64 / dt_real;
        }

        self.prev_net_rx = rx;
        self.prev_net_tx = tx;
        self.net_rx_total = rx;
        self.net_tx_total = tx;
    }

    pub fn update_disks(&mut self) {
        let now = self.last_add;
        if self.last_disk_refresh > 0.0 && now - self.last_disk_refresh < 1.0 {
            return;
        }

        let dt_real = if self.last_disk_refresh > 0.0 {
            (now - self.last_disk_refresh).max(0.001)
        } else {
            1.0
        };
        self.last_disk_refresh = now;

        self.disks.refresh(true);

        let n = self.disks.len();

        if self.per_disk_read_bps.len() != n {
            self.per_disk_read_bps = vec![0.0; n];
            self.per_disk_write_bps = vec![0.0; n];
            self.per_disk_activity = vec![0.0; n];
            self.per_disk_activity_smoothed = vec![0.0; n];
            self.per_disk_rate_smoothed = vec![0.0; n];
            self.per_disk_prev_read = vec![0u64; n];
            self.per_disk_prev_write = vec![0u64; n];
            self.per_disk_history = (0..n).map(|_| Vec::with_capacity(MAX_POINTS)).collect();
            self.per_disk_rate_history = (0..n).map(|_| Vec::with_capacity(MAX_POINTS)).collect();
        }

        let mut total_read: u64 = 0;
        let mut total_write: u64 = 0;

        for (i, d) in self.disks.iter().enumerate() {
            let usage = d.usage();
            let read = usage.read_bytes;
            let write = usage.written_bytes;

            total_read += read;
            total_write += write;

            if self.per_disk_prev_read[i] > 0 {
                self.per_disk_read_bps[i] = read as f64 / dt_real;
                self.per_disk_write_bps[i] = write as f64 / dt_real;

                let max_bps = match d.kind() {
                    sysinfo::DiskKind::HDD => 150.0 * 1024.0 * 1024.0,
                    sysinfo::DiskKind::SSD => 500.0 * 1024.0 * 1024.0,
                    _ => 200.0 * 1024.0 * 1024.0,
                };
                let total_bps = self.per_disk_read_bps[i] + self.per_disk_write_bps[i];
                self.per_disk_activity[i] = (total_bps / max_bps).min(1.0) as f32;
            }

            self.per_disk_prev_read[i] = read;
            self.per_disk_prev_write[i] = write;
        }

        if self.prev_disk_read > 0 {
            self.disk_read_bps = total_read as f64 / dt_real;
            self.disk_write_bps = total_write as f64 / dt_real;

            let max_bps = 300.0 * 1024.0 * 1024.0;
            let total_bps = self.disk_read_bps + self.disk_write_bps;
            self.disk_activity = (total_bps / max_bps).min(1.0) as f32;
        }

        self.prev_disk_read = total_read;
        self.prev_disk_write = total_write;
    }

    pub fn read_gpu_temp(&self) -> f32 {
        let Some(nvml) = &self.nvml else { return 0.0; };
        let Ok(device) = nvml.device_by_index(0) else { return 0.0; };
        match device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
            Ok(t) => t as f32,
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

            self.per_core_usage = self
                .system
                .cpus()
                .iter()
                .map(|c| c.cpu_usage() / 100.0)
                .collect();

            self.per_core_smoothed = self.per_core_usage.clone();

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

        // Per-core
        if self.per_core_smoothed.len() != self.per_core_usage.len() {
            self.per_core_smoothed = self.per_core_usage.clone();
        }
        for (i, &raw) in self.per_core_usage.iter().enumerate() {
            self.per_core_smoothed[i] += (raw - self.per_core_smoothed[i]) * alpha;
        }

        // Активность дисков
        if self.per_disk_activity_smoothed.len() != self.per_disk_activity.len() {
            self.per_disk_activity_smoothed = self.per_disk_activity.clone();
        }
        for (i, &raw) in self.per_disk_activity.iter().enumerate() {
            self.per_disk_activity_smoothed[i] += (raw - self.per_disk_activity_smoothed[i]) * alpha;
        }

        // Скорость передачи по дискам
        if self.per_disk_rate_smoothed.len() != self.per_disk_activity.len() {
            self.per_disk_rate_smoothed = vec![0.0; self.per_disk_activity.len()];
        }
        {
            let disks_snapshot: Vec<_> = self.disks.iter().collect();
            for i in 0..self.per_disk_activity.len() {
                let max_bps = if i < disks_snapshot.len() {
                    match disks_snapshot[i].kind() {
                        sysinfo::DiskKind::HDD => 150.0 * 1024.0 * 1024.0,
                        sysinfo::DiskKind::SSD => 500.0 * 1024.0 * 1024.0,
                        _ => 200.0 * 1024.0 * 1024.0,
                    }
                } else {
                    200.0 * 1024.0 * 1024.0
                };
                let total_bps = self.per_disk_read_bps[i] + self.per_disk_write_bps[i];
                let raw_rate = (total_bps / max_bps).min(1.0) as f32;
                self.per_disk_rate_smoothed[i] += (raw_rate - self.per_disk_rate_smoothed[i]) * alpha;
            }
        }

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

            self.update_networks();
            self.update_disks();
            self.gpu_temp_c = self.read_gpu_temp();

            self.top_processes = collect_top(&mut self.system, 50);

            self.per_core_usage = self
                .system
                .cpus()
                .iter()
                .map(|c| c.cpu_usage() / 100.0)
                .collect();

            if self.per_core_history.len() != self.per_core_usage.len() {
                self.per_core_history = (0..self.per_core_usage.len())
                    .map(|_| Vec::with_capacity(MAX_POINTS))
                    .collect();
            }

            for (i, &usage) in self.per_core_usage.iter().enumerate() {
                let hist = &mut self.per_core_history[i];
                if hist.len() >= MAX_POINTS {
                    hist.remove(0);
                }
                hist.push(Sample {
                    value: usage,
                    time: self.last_add,
                });
            }

            // Общие истории
            if self.cpu_history.len() >= MAX_POINTS {
                self.cpu_history.remove(0);
            }
            self.cpu_history.push(Sample {
                value: self.cpu_smoothed,
                time: self.last_add,
            });

            if self.mem_history.len() >= MAX_POINTS {
                self.mem_history.remove(0);
            }
            self.mem_history.push(Sample {
                value: self.mem_smoothed,
                time: self.last_add,
            });

            if self.disk_history.len() >= MAX_POINTS {
                self.disk_history.remove(0);
            }
            self.disk_history.push(Sample {
                value: self.disk_activity,
                time: self.last_add,
            });

            // Per-disk
            if self.per_disk_history.len() != self.per_disk_activity.len() {
                self.per_disk_history = (0..self.per_disk_activity.len())
                    .map(|_| Vec::with_capacity(MAX_POINTS))
                    .collect();
            }
            if self.per_disk_rate_history.len() != self.per_disk_activity.len() {
                self.per_disk_rate_history = (0..self.per_disk_activity.len())
                    .map(|_| Vec::with_capacity(MAX_POINTS))
                    .collect();
            }

            let disks_snapshot: Vec<_> = self.disks.iter().collect();
            for i in 0..self.per_disk_activity.len() {
                // Активность
                let hist = &mut self.per_disk_history[i];
                if hist.len() >= MAX_POINTS {
                    hist.remove(0);
                }
                hist.push(Sample {
                    value: self.per_disk_activity[i],
                    time: self.last_add,
                });

                // Скорость — нормируем по типу диска
                let max_bps = if i < disks_snapshot.len() {
                    match disks_snapshot[i].kind() {
                        sysinfo::DiskKind::HDD => 150.0 * 1024.0 * 1024.0,
                        sysinfo::DiskKind::SSD => 500.0 * 1024.0 * 1024.0,
                        _ => 200.0 * 1024.0 * 1024.0,
                    }
                } else {
                    200.0 * 1024.0 * 1024.0
                };
                let total_bps = self.per_disk_read_bps[i] + self.per_disk_write_bps[i];
                let rate_norm = (total_bps / max_bps).min(1.0) as f32;

                let rate_hist = &mut self.per_disk_rate_history[i];
                if rate_hist.len() >= MAX_POINTS {
                    rate_hist.remove(0);
                }
                rate_hist.push(Sample {
                    value: rate_norm,
                    time: self.last_add,
                });
            }

            let net_total = self.net_rx_bps + self.net_tx_bps;
            let net_norm = (net_total / (10.0 * 1024.0 * 1024.0)).min(1.0) as f32;
            if self.net_history.len() >= MAX_POINTS {
                self.net_history.remove(0);
            }
            self.net_history.push(Sample {
                value: net_norm,
                time: self.last_add,
            });

            if self.nvidia_history.len() >= MAX_POINTS {
                self.nvidia_history.remove(0);
            }
            self.nvidia_history.push(Sample {
                value: self.gpu_nvidia_load,
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
                Tab::Performance => performance_tab::draw(self, ui, now),
                Tab::SystemInfo => tabs::draw_placeholder(ui, "Система"),
                Tab::StartupApps => tabs::draw_placeholder(ui, "Автозагрузка"),
                Tab::Users => tabs::draw_placeholder(ui, "Пользователи"),
                Tab::Services => tabs::draw_placeholder(ui, "Службы"),
                Tab::Settings => tabs::draw_placeholder(ui, "Настройки"),
            });
    }
}