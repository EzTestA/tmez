pub mod summary;
pub mod processes;

/// Список вкладок приложения.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Summary,
    Performance,
    Processes,
    SystemInfo,
    StartupApps,
    Users,
    Services,
    Settings,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Summary => "📊  Обзор",
            Tab::Performance => "📈  Производительность",
            Tab::Processes => "📋  Процессы",
            Tab::SystemInfo => "ℹ  Система",
            Tab::StartupApps => "🚀  Автозагрузка",
            Tab::Users => "👤  Пользователи",
            Tab::Services => "⚙  Службы",
            Tab::Settings => "🔧  Настройки",
        }
    }

    pub fn all() -> &'static [Tab] {
        &[
            Tab::Summary,
            Tab::Performance,
            Tab::Processes,
            Tab::SystemInfo,
            Tab::StartupApps,
            Tab::Users,
            Tab::Services,
            Tab::Settings,
        ]
    }
}

/// Общая функция-заглушка для нереализованных разделов.
pub fn draw_placeholder(ui: &mut egui::Ui, title: &str) {
    ui.heading(title);
    ui.add_space(20.0);
    ui.label(
        egui::RichText::new("🚧  Раздел в разработке")
            .size(16.0)
            .color(egui::Color32::from_gray(140)),
    );
}