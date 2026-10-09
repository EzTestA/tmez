use eframe::egui;
use crate::theme::{STEP_PX, MAX_POINTS};

/// Точка истории: значение и «возраст» (сколько INTERVAL прошло с момента записи).
#[derive(Clone, Copy)]
pub struct Sample {
    /// Значение 0.0..1.0
    pub value: f32,
    /// Возраст в единицах INTERVAL.
    /// 0.0 — только что записана (стоит на right_x),
    /// 1.0 — один INTERVAL назад (сдвинута на STEP_PX влево).
    pub age: f32,
}

/// Градиент по высоте: 0.0 = зелёный, 0.5 = жёлтый, 1.0 = красный.
pub fn gradient_color(v: f32) -> egui::Color32 {
    let v = v.clamp(0.0, 1.0);

    let stops: [(f32, f32, f32, f32); 3] = [
        (0.0, 0.15, 0.95, 0.15),
        (0.5, 0.95, 0.90, 0.10),
        (1.0, 0.95, 0.15, 0.10),
    ];

    let mut lo = stops[0];
    let mut hi = stops[stops.len() - 1];
    for w in stops.windows(2) {
        if v >= w[0].0 && v <= w[1].0 {
            lo = w[0];
            hi = w[1];
            break;
        }
    }

    let span = (hi.0 - lo.0).max(1e-6);
    let t = ((v - lo.0) / span).clamp(0.0, 1.0);

    let r = lo.1 + (hi.1 - lo.1) * t;
    let g = lo.2 + (hi.2 - lo.2) * t;
    let b = lo.3 + (hi.3 - lo.3) * t;

    egui::Color32::from_rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

/// Сдвигает возраст всех точек истории на `age_delta` (в единицах INTERVAL).
/// Вызывать один раз за кадр, до добавления новых точек.
pub fn advance_history(history: &mut [Sample], age_delta: f32) {
    for s in history.iter_mut() {
        s.age += age_delta;
    }
}

/// Добавляет точку в историю с age = 0.0 и подрезает до MAX_POINTS.
pub fn push_sample(history: &mut Vec<Sample>, value: f32) {
    if history.len() >= MAX_POINTS {
        history.remove(0);
    }
    history.push(Sample { value, age: 0.0 });
}

/// Рисует график с историей, «живой» точкой и градиентной линией.
/// Больше не зависит от `now` — только от `age` точек.
pub fn draw_graph(
    ui: &mut egui::Ui,
    history: &[Sample],
    live_value: f32,
    height: f32,
) {
    let desired_size = egui::vec2(ui.available_width(), height);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(6, 6, 10));

    let right_x = rect.right() - 10.0;
    let left_x = rect.left() + 4.0;
    let top = rect.top() + 8.0;
    let bottom = rect.bottom() - 8.0;
    let h = bottom - top;

    // --- Сетка (статичный фон) ---
    let grid_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(24, 24, 34));

    let mut k: i32 = 0;
    loop {
        let x = right_x - (k as f32) * STEP_PX;
        if x < left_x {
            break;
        }
        painter.line_segment(
            [egui::pos2(x, top), egui::pos2(x, bottom)],
            grid_stroke,
        );
        k += 1;
    }

    for frac in [0.25_f32, 0.5, 0.75, 1.0] {
        let y = bottom - frac * h;
        painter.line_segment(
            [egui::pos2(left_x, y), egui::pos2(rect.right(), y)],
            grid_stroke,
        );
    }

    // --- Точки ---
    let to_screen = |age: f32, v: f32| -> egui::Pos2 {
        let x = right_x - age * STEP_PX;
        let y = bottom - v.clamp(0.0, 1.0) * h;
        egui::pos2(x, y)
    };

    let mut pts: Vec<egui::Pos2> = history
        .iter()
        .map(|s| to_screen(s.age, s.value))
        .filter(|p| p.x >= left_x - STEP_PX)
        .collect();

    // Живая точка: age = 0 → ровно на right_x.
    pts.push(to_screen(0.0, live_value));

    if pts.len() < 2 {
        return;
    }

    // --- Линия с градиентом ---
    let thickness = 4.0_f32;
    let mut mesh = egui::Mesh::default();

    for (i, p) in pts.iter().enumerate() {
        let v = ((bottom - p.y) / h).clamp(0.0, 1.0);
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