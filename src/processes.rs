use sysinfo::{System, ProcessesToUpdate};

/// Одна строка в таблице процессов.
#[derive(Clone)]
pub struct ProcessRow {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,        // проценты (может быть > 100)
    pub memory_mb: f32,  // мегабайты
}

/// Обновляет список процессов и возвращает их отсортированными по CPU (убывание).
/// Возвращает не больше `limit` штук.
pub fn collect_top(system: &mut System, limit: usize) -> Vec<ProcessRow> {
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut rows: Vec<ProcessRow> = system
        .processes()
        .iter()
        .map(|(pid, process)| ProcessRow {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            cpu: process.cpu_usage(),
            memory_mb: process.memory() as f32 / 1024.0 / 1024.0,
        })
        .collect();

    // Сортировка по CPU, от большего к меньшему
    rows.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));

    rows.truncate(limit);
    rows
}