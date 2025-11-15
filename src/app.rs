// src/app.rs

use crate::algo::{self, Algorithm, AntialiasedPixel};
use crate::logger::{Logger, NoOpLogger, StringLogger};
use egui::{
    color_picker, Align2, Color32, Painter, Pos2, Rect, Rounding, Sense, Stroke, Vec2,
};

// Структура для хранения состояния открытого текстового окна
struct TextWindow {
    title: String,
    content: String,
    is_open: bool,
}

// Структура для хранения параметров алгоритмов
struct AppParameters {
    p1: Pos2,
    p2: Pos2,
    circle_center: Pos2,
    circle_radius: f32,
    castle_points: Vec<Pos2>,
}

impl Default for AppParameters {
    fn default() -> Self {
        Self {
            p1: Pos2::new(-50.0, -10.0),
            p2: Pos2::new(50.0, 20.0),
            circle_center: Pos2::new(0.0, 0.0),
            circle_radius: 60.0,
            castle_points: vec![
                Pos2::new(-80.0, -50.0),
                Pos2::new(-30.0, 80.0),
                Pos2::new(30.0, -80.0),
                Pos2::new(80.0, 50.0),
            ],
        }
    }
}

// Структура для хранения результата работы алгоритма
enum RenderResult {
    None,
    Pixels(Vec<Pos2>),
    Antialiased(Vec<AntialiasedPixel>),
}

// Основная структура приложения
pub struct GraphicsLabApp {
    selected_algorithm: Algorithm,
    params: AppParameters,
    log_enabled: bool,

    // Состояние холста
    pan: Vec2,
    zoom: f32,

    // Результаты и окна
    last_run_algorithm: Option<Algorithm>,
    render_result: RenderResult,
    text_windows: Vec<TextWindow>,
}

impl Default for GraphicsLabApp {
    fn default() -> Self {
        Self {
            selected_algorithm: Algorithm::BresenhamLine,
            params: AppParameters::default(),
            log_enabled: false,
            pan: Vec2::ZERO,
            zoom: 2.0,
            last_run_algorithm: None,
            render_result: RenderResult::None,
            text_windows: Vec::new(),
        }
    }
}

// Реализация логики приложения
impl eframe::App for GraphicsLabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Панель управления (без изменений) ---
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Управление");
            ui.separator();

            // ... (весь код панели управления остается здесь) ...
            egui::ComboBox::from_label("Алгоритм")
                .selected_text(format!("{}", self.selected_algorithm))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false); // Предотвращаем перенос строк в комбобоксе
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::StepByStep, "StepByStep");
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::DDA, "DDA");
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::BresenhamLine, "BresenhamLine");
                    ui.separator();
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::StepByStepAA, "StepByStep (AA)");
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::DdaAA, "DDA (AA)");
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::BresenhamAA, "Bresenham (Gupta-Sproull)");
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::WuLine, "WuLine");
                    ui.separator();
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::BresenhamCircle, "BresenhamCircle");
                    ui.selectable_value(&mut self.selected_algorithm, Algorithm::CastlePitway, "CastlePitway");
                });

            ui.separator();
            self.show_parameters_ui(ui);
            ui.separator();

            ui.checkbox(&mut self.log_enabled, "Сохранять вычисления");
            if ui.button("Запуск").clicked() {
                self.run_algorithm();
            }
            if ui.button("Справка").clicked() {
                self.show_help();
            }
        });

        // --- Отрисовка текстовых окон (без изменений) ---
        self.draw_text_windows(ctx);

        // --- Переменные для обмена данными между панелями ---
        // Мы вычислим их в CentralPanel, а используем в TopBottomPanel.
        let mut canvas_info = None;

        // --- Центральная панель (холст) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

            self.handle_canvas_input(&response);

            let to_screen = self.get_transform(&response.rect);
            let from_screen = to_screen.inverse();

            self.draw_grid(&painter, &response.rect, from_screen);
            self.draw_results(&painter, to_screen);

            // Сохраняем информацию для строки состояния
            canvas_info = Some((response.rect.clone(), from_screen, response.hover_pos()));
        });

        // --- Нижняя панель (строка состояния) ---
        // Теперь она находится на верхнем уровне, как и положено.
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            if let Some((rect, from_screen, hover_pos)) = canvas_info {
                self.draw_status_bar_content(ui, &rect, from_screen, hover_pos);
            }
        });
    }
}

// Вспомогательные функции
impl GraphicsLabApp {

    fn show_parameters_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Параметры:");
        match self.selected_algorithm {
            Algorithm::StepByStep | Algorithm::DDA | Algorithm::BresenhamLine | Algorithm::WuLine |
            Algorithm::StepByStepAA | Algorithm::DdaAA | Algorithm::BresenhamAA => {
                ui.horizontal(|ui| {
                    ui.label("P1:");
                    ui.add(egui::DragValue::new(&mut self.params.p1.x).speed(1.0).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut self.params.p1.y).speed(1.0).prefix("y:"));
                });
                ui.horizontal(|ui| {
                    ui.label("P2:");
                    ui.add(egui::DragValue::new(&mut self.params.p2.x).speed(1.0).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut self.params.p2.y).speed(1.0).prefix("y:"));
                });
            }
            Algorithm::BresenhamCircle => {
                ui.horizontal(|ui| {
                    ui.label("Центр:");
                    ui.add(egui::DragValue::new(&mut self.params.circle_center.x).speed(1.0).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut self.params.circle_center.y).speed(1.0).prefix("y:"));
                });
                ui.add(egui::DragValue::new(&mut self.params.circle_radius).speed(1.0).prefix("Радиус:"));
            }
            Algorithm::CastlePitway => {
                ui.label("Опорные точки:");
                for (i, p) in self.params.castle_points.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("P{}:", i));
                        ui.add(egui::DragValue::new(&mut p.x).speed(1.0).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut p.y).speed(1.0).prefix("y:"));
                    });
                }
            }
        }
    }

    fn run_algorithm(&mut self) {
        self.last_run_algorithm = Some(self.selected_algorithm);

        if self.log_enabled {
            let mut logger = StringLogger::new();
            self.render_result = self.execute_and_get_result(&mut logger);

            let log_content = logger.results(); // logger будет "потреблен" здесь
            self.text_windows.push(TextWindow {
                title: format!("Логи: {}", self.selected_algorithm),
                content: log_content,
                is_open: true,
            });
        } else {
            let mut logger = NoOpLogger;
            self.render_result = self.execute_and_get_result(&mut logger);
        }
    }

    // Наша новая generic-функция-хелпер
    fn execute_and_get_result<L: Logger>(&self, logger: &mut L) -> RenderResult {
        match self.selected_algorithm {
            Algorithm::StepByStep => {
                RenderResult::Pixels(algo::step_by_step(self.params.p1, self.params.p2, logger))
            }
            Algorithm::DDA => {
                RenderResult::Pixels(algo::dda(self.params.p1, self.params.p2, logger))
            }
            Algorithm::BresenhamLine => {
                RenderResult::Pixels(algo::bresenham_line(self.params.p1, self.params.p2, logger))
            }
            Algorithm::BresenhamCircle => RenderResult::Pixels(algo::bresenham_circle(
                self.params.circle_center,
                self.params.circle_radius,
                logger,
            )),
            Algorithm::StepByStepAA => RenderResult::Antialiased(algo::step_by_step_aa(
                self.params.p1, self.params.p2, logger
            )),
            Algorithm::DdaAA => RenderResult::Antialiased(algo::dda_aa(
                self.params.p1, self.params.p2, logger
            )),
            Algorithm::BresenhamAA => RenderResult::Antialiased(
                algo::bresenham_aa(self.params.p1, self.params.p2, logger)
            ),
            Algorithm::CastlePitway => RenderResult::Pixels(algo::castle_pitway(
                &self.params.castle_points,
                logger,
            )),
            Algorithm::WuLine => {
                RenderResult::Antialiased(algo::wu_line(self.params.p1, self.params.p2, logger))
            }
        }
    }

    fn show_help(&mut self) {
        let help_content = match self.selected_algorithm {
            Algorithm::StepByStep => "Временная сложность: O(N), где N - длина отрезка по доминантной оси. Использует операции с плавающей точкой.",
            Algorithm::DDA => "Временная сложность: O(N), где N - длина отрезка по доминантной оси. Использует операции с плавающей точкой, но более эффективен, чем пошаговый.",
            Algorithm::BresenhamLine => "Временная сложность: O(N), где N - длина отрезка по доминантной оси. Использует только целочисленную арифметику, очень быстрый.",
            Algorithm::BresenhamCircle => "Временная сложность: O(R), где R - радиус. Вычисляет одну восьмую часть окружности, используя только целочисленную арифметику.",
            Algorithm::CastlePitway => "Временная сложность: O(S * P^2), где S - количество шагов, P - количество опорных точек. Сложность для генерации одной точки - O(P^2).",
            Algorithm::WuLine => "Временная сложность: O(N), где N - длина отрезка по доминантной оси. Эталонный алгоритм сглаживания. Использует вычисления с плавающей точкой для определения интенсивности пикселей.",
            Algorithm::StepByStepAA => "Временная сложность: O(N). Аналогичен обычному StepByStep, но вычисляет интенсивность для двух пикселей на каждом шаге вместо округления.",
            Algorithm::DdaAA => "Временная сложность: O(N). Аналогичен обычному DDA, но использует идеальные координаты для вычисления интенсивности двух пикселей на каждом шаге.",
            Algorithm::BresenhamAA => "Временная сложность: O(N). Модификация алгоритма Брезенхема. Сохраняет целочисленный итеративный процесс, но использует параметр ошибки для вычисления интенсивности пикселей (требует деления на каждом шаге).",
        };

        self.text_windows.push(TextWindow {
            title: format!("Справка: {}", self.selected_algorithm),
            content: help_content.to_string(),
            is_open: true,
        });
    }

    fn draw_text_windows(&mut self, ctx: &egui::Context) {
        self.text_windows.retain_mut(|win| {
            let mut is_open = win.is_open;
            egui::Window::new(&win.title)
                .open(&mut is_open)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.label(&win.content);
                });
            win.is_open = is_open;
            win.is_open
        });
    }

    fn get_transform(&self, rect: &Rect) -> egui::emath::RectTransform {
        let center = rect.center();
        egui::emath::RectTransform::from_to(
            Rect::from_center_size(Pos2::ZERO, rect.size() / self.zoom),
            Rect::from_center_size(center + self.pan, rect.size()),
        )
    }

    fn handle_canvas_input(&mut self, response: &egui::Response) {
        if response.dragged_by(egui::PointerButton::Primary) {
            self.pan += response.drag_delta();
        }
        if response.hovered() {
            let scroll = response.ctx.input(|i| i.scroll_delta);
            if scroll.y != 0.0 {
                let old_zoom = self.zoom;
                self.zoom = (self.zoom * (1.0 + scroll.y * 0.01)).max(0.1);
                if let Some(hover_pos) = response.hover_pos() {
                    let from_screen = self.get_transform(&response.rect).inverse();
                    let logic_pos_before = from_screen * hover_pos;
                    let to_screen_after = self.get_transform(&response.rect);
                    let screen_pos_after = to_screen_after * logic_pos_before;
                    self.pan += hover_pos - screen_pos_after;
                }
            }
        }
    }

    fn calculate_grid_step(&self) -> f32 {
        // Цель: иметь линии сетки примерно каждые 80 пикселей
        let target_step_pixels = 80.0;
        let logical_step = target_step_pixels / self.zoom;

        // Находим ближайшую степень 10
        let power_of_10 = 10.0_f32.powf(logical_step.log10().floor());

        // Нормализуем шаг к диапазону [1, 10)
        let normalized_step = logical_step / power_of_10;

        // Выбираем "красивый" шаг из 1, 2, 5
        let beautiful_step = if normalized_step < 1.5 {
            1.0
        } else if normalized_step < 3.5 {
            2.0
        } else if normalized_step < 7.5 {
            5.0
        } else {
            10.0
        };

        beautiful_step * power_of_10
    }

    fn draw_grid(&self, painter: &Painter, rect: &Rect, from_screen: egui::emath::RectTransform) {
        let to_screen = self.get_transform(rect);

        // 1. Фон
        painter.rect_filled(rect.clone(), Rounding::none(), Color32::WHITE);

        // Получаем видимые границы в логических координатах
        let top_left = from_screen * rect.min;
        let bottom_right = from_screen * rect.max;

        // 2. Сетка
        let grid_step = self.calculate_grid_step();
        let grid_color = Color32::from_rgb(230, 230, 250); // Очень светло-синий
        let grid_stroke = Stroke::new(1.0, grid_color);

        // Вертикальные линии (со смещением 0.5)
        let x_start = ((top_left.x - 0.5) / grid_step).floor() * grid_step + 0.5;
        let mut x = x_start;
        while x < bottom_right.x {
            let line_start = to_screen * Pos2::new(x, top_left.y);
            let line_end = to_screen * Pos2::new(x, bottom_right.y);
            painter.line_segment([line_start, line_end], grid_stroke);
            x += grid_step;
        }

        // Горизонтальные линии (со смещением 0.5)
        let y_start = ((top_left.y - 0.5) / grid_step).floor() * grid_step + 0.5;
        let mut y = y_start;
        while y < bottom_right.y {
            let line_start = to_screen * Pos2::new(top_left.x, y);
            let line_end = to_screen * Pos2::new(bottom_right.x, y);
            painter.line_segment([line_start, line_end], grid_stroke);
            y += grid_step;
        }

        // 3. Оси координат
        let axes_color = Color32::from_rgb(200, 200, 255);
        let axes_stroke = Stroke::new(1.5, axes_color);

        // Ось Y (линия x = 0)
        let y_axis_start = to_screen * Pos2::new(0.0, top_left.y);
        let y_axis_end = to_screen * Pos2::new(0.0, bottom_right.y);
        painter.line_segment([y_axis_start, y_axis_end], axes_stroke);

        // Ось X (линия y = 0)
        let x_axis_start = to_screen * Pos2::new(top_left.x, 0.0);
        let x_axis_end = to_screen * Pos2::new(bottom_right.x, 0.0);
        painter.line_segment([x_axis_start, x_axis_end], axes_stroke);

        // --- НОВЫЙ БЛОК ---
        // 4. Подписи осей
        let text_color = Color32::DARK_GRAY;
        let font_id = egui::FontId::proportional(16.0);

        // Подпись оси Y
        // Позиция: x=0, y - вверху экрана. Смещаем немного вправо и вниз для читаемости.
        let y_label_pos = to_screen * Pos2::new(0.0, top_left.y);
        painter.text(
            y_label_pos + Vec2::new(10.0, 10.0), // Отступ в экранных координатах
            Align2::LEFT_TOP, // Якорь - левый верхний угол текста
            "Y",
            font_id.clone(),
            text_color,
        );

        // Подпись оси X
        // Позиция: y=0, x - справа на экране. Смещаем немного влево и вверх для читаемости.
        let x_label_pos = to_screen * Pos2::new(bottom_right.x, 0.0);
        painter.text(
            x_label_pos + Vec2::new(-10.0, -10.0), // Отступ в экранных координатах
            Align2::RIGHT_BOTTOM, // Якорь - правый нижний угол текста
            "X",
            font_id,
            text_color,
        );
    }

    // В impl GraphicsLabApp в src/app.rs

    fn draw_results(&self, painter: &Painter, to_screen: egui::emath::RectTransform) {
        let pixel_size = Vec2::splat(1.0);

        // Отрисовка растеризованных пикселей
        match &self.render_result {
            RenderResult::Pixels(pixels) => {
                for &p in pixels {
                    let screen_pos = to_screen * p;
                    painter.rect_filled(Rect::from_center_size(screen_pos, pixel_size * self.zoom), Rounding::none(), Color32::BLACK); // БЫЛО WHITE
                }
            }
            RenderResult::Antialiased(pixels) => {
                for &(x, y, intensity) in pixels {
                    let p = Pos2::new(x as f32, y as f32);
                    let screen_pos = to_screen * p;
                    // Используем from_black_alpha для градиента от прозрачного до черного
                    let color = Color32::from_black_alpha((intensity * 255.0) as u8); // БЫЛО from_white_alpha
                    painter.rect_filled(Rect::from_center_size(screen_pos, pixel_size * self.zoom), Rounding::none(), color);
                }
            }
            RenderResult::None => {}
        }

        // Отрисовка "идеальных" линий и маркеров
        if let Some(algo) = self.last_run_algorithm {
            match algo {
                Algorithm::StepByStep | Algorithm::DDA | Algorithm::BresenhamLine | Algorithm::WuLine | Algorithm::BresenhamAA | Algorithm::DdaAA | Algorithm::StepByStepAA => {
                    let p1 = to_screen * self.params.p1;
                    let p2 = to_screen * self.params.p2;
                    // Линию делаем темно-серой, чтобы она отличалась от черных пикселей
                    painter.line_segment([p1, p2], Stroke::new(1.0, Color32::RED)); // БЫЛ синий
                    painter.circle_filled(p1, 4.0, Color32::RED);
                    painter.circle_filled(p2, 4.0, Color32::RED);
                }
                Algorithm::BresenhamCircle => {
                    let center = to_screen * self.params.circle_center;
                    let radius = self.params.circle_radius * self.zoom;
                    painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::RED)); // БЫЛ синий
                }
                Algorithm::CastlePitway => {
                    for p in &self.params.castle_points {
                        painter.circle_filled(to_screen * *p, 4.0, Color32::RED);
                    }
                }
            }
        }
    }

    fn draw_status_bar_content(&self, ui: &mut egui::Ui, rect: &Rect, from_screen: egui::emath::RectTransform, hover_pos: Option<Pos2>) {
        ui.horizontal(|ui| {
            // Координаты мыши
            if let Some(pos) = hover_pos {
                let logic_pos = from_screen * pos;
                ui.label(format!("Курсор: ({:.1}, {:.1})", logic_pos.x, logic_pos.y));
            } else {
                ui.label("Курсор: (N/A)");
            }

            ui.separator();

            // Последний метод
            if let Some(algo) = self.last_run_algorithm {
                ui.label(format!("Метод: {}", algo));
            }

            ui.separator();

            // Видимый диапазон
            let top_left = from_screen * rect.min;
            let bottom_right = from_screen * rect.max;
            ui.label(format!("X: [{:.1}:{:.1}], Y: [{:.1}:{:.1}]", top_left.x, bottom_right.x, top_left.y, bottom_right.y));
        });
    }
}