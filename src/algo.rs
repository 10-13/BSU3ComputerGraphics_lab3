// src/algorithms.rs

use crate::logger::Logger;
use egui::Pos2;

// Тип для представления пикселя со значением интенсивности (для сглаживания)
pub type AntialiasedPixel = (i32, i32, f32);

// Enum для выбора алгоритма в интерфейсе
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Algorithm {
    StepByStep,
    DDA,
    BresenhamLine,
    BresenhamCircle,
    CastlePitway,
    WuLine,
    StepByStepAA,
    DdaAA,
    BresenhamAA,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Пошаговый алгоритм
pub fn step_by_step<L: Logger>(p1: Pos2, p2: Pos2, logger: &mut L) -> Vec<Pos2> {
    let mut pixels = Vec::new();
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    if dx.abs() > dy.abs() {
        let k = dy / dx;
        let b = p1.y - k * p1.x;
        
        let (start_x, end_x) = if p1.x < p2.x { (p1.x, p2.x) } else { (p2.x, p1.x) };

        for x in (start_x.round() as i32)..=(end_x.round() as i32) {
            let y = (k * x as f32 + b).round() as i32;
            if logger.can_write() {
                logger.log(format!("x = {}, y = {:.2} * {} + {:.2} = {:.2} -> округляем до {}", x, k, x, b, (k * x as f32 + b), y));
            }
            pixels.push(Pos2::new(x as f32, y as f32));
        }
    } else {
        let k = dx / dy;
        let b = p1.x - k * p1.y;
        
        let (start_y, end_y) = if p1.y < p2.y { (p1.y, p2.y) } else { (p2.y, p1.y) };

        for y in (start_y.round() as i32)..=(end_y.round() as i32) {
            let x = (k * y as f32 + b).round() as i32;
            if logger.can_write() {
                logger.log(format!("y = {}, x = {:.2} * {} + {:.2} = {:.2} -> округляем до {}", y, k, y, b, (k * y as f32 + b), x));
            }
            pixels.push(Pos2::new(x as f32, y as f32));
        }
    }
    pixels
}

/// Алгоритм ЦДА
pub fn dda<L: Logger>(p1: Pos2, p2: Pos2, logger: &mut L) -> Vec<Pos2> {
    let mut pixels = Vec::new();
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    let steps = if dx.abs() > dy.abs() { dx.abs() } else { dy.abs() };
    let x_inc = dx / steps;
    let y_inc = dy / steps;

    let mut x = p1.x;
    let mut y = p1.y;

    for i in 0..=steps.round() as u32 {
        let ix = x.round() as i32;
        let iy = y.round() as i32;
        if logger.can_write() {
            logger.log(format!("Шаг {}: x = {:.2}, y = {:.2} -> Пиксель ({}, {})", i, x, y, ix, iy));
        }
        pixels.push(Pos2::new(ix as f32, iy as f32));
        x += x_inc;
        y += y_inc;
    }
    pixels
}

/// Алгоритм Брезенхема для отрезка
pub fn bresenham_line<L: Logger>(p1: Pos2, p2: Pos2, logger: &mut L) -> Vec<Pos2> {
    let mut pixels = Vec::new();
    let mut x1 = p1.x.round() as i32;
    let mut y1 = p1.y.round() as i32;
    let x2 = p2.x.round() as i32;
    let y2 = p2.y.round() as i32;

    let dx = (x2 - x1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let dy = -(y2 - y1).abs();
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        pixels.push(Pos2::new(x1 as f32, y1 as f32));
        if logger.can_write() {
            logger.log(format!("Пиксель: ({}, {}), Ошибка: {}", x1, y1, err));
        }
        if x1 == x2 && y1 == y2 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
    }
    pixels
}

/// Алгоритм Брезенхема для окружности
pub fn bresenham_circle<L: Logger>(center: Pos2, radius: f32, logger: &mut L) -> Vec<Pos2> {
    let mut pixels = Vec::new();
    let cx = center.x.round() as i32;
    let cy = center.y.round() as i32;
    let r = radius.round() as i32;

    let mut x = 0;
    let mut y = r;
    let mut d = 3 - 2 * r;

    while y >= x {
        if logger.can_write() {
            logger.log(format!("x={}, y={}, d={}", x, y, d));
        }
        // Отрисовка для всех 8 октантов
        pixels.push(Pos2::new((cx+x) as f32, (cy+y) as f32));
        pixels.push(Pos2::new((cx-x) as f32, (cy+y) as f32));
        pixels.push(Pos2::new((cx+x) as f32, (cy-y) as f32));
        pixels.push(Pos2::new((cx-x) as f32, (cy-y) as f32));
        pixels.push(Pos2::new((cx+y) as f32, (cy+x) as f32));
        pixels.push(Pos2::new((cx-y) as f32, (cy+x) as f32));
        pixels.push(Pos2::new((cx+y) as f32, (cy-x) as f32));
        pixels.push(Pos2::new((cx-y) as f32, (cy-x) as f32));

        x += 1;
        if d > 0 {
            y -= 1;
            d = d + 4 * (x - y) + 10;
        } else {
            d = d + 4 * x + 6;
        }
    }
    pixels
}

/// Алгоритм Кастла-Питвея (Де Кастельжо)
pub fn castle_pitway<L: Logger>(points: &[Pos2], logger: &mut L) -> Vec<Pos2> {
    let mut curve_pixels = Vec::new();
    if points.len() < 2 { return curve_pixels; }

    let steps = 1000; // Количество шагов для построения кривой
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let mut temp_points = points.to_vec();

        if logger.can_write() && i % 100 == 0 { // Логируем каждый 100-й шаг
            logger.log(format!("t = {:.2}", t));
        }

        while temp_points.len() > 1 {
            temp_points = temp_points
                .windows(2)
                .map(|p| p[0].lerp(p[1], t))
                .collect();
        }
        if let Some(p) = temp_points.first() {
            let pixel = Pos2::new(p.x.round(), p.y.round());
            if !curve_pixels.contains(&pixel) {
                curve_pixels.push(pixel);
            }
        }
    }
    curve_pixels
}

/// Алгоритм сглаживания Ву
pub fn wu_line<L: Logger>(p1: Pos2, p2: Pos2, logger: &mut L) -> Vec<AntialiasedPixel> {
    let mut pixels = Vec::new();
    let mut x0 = p1.x;
    let mut y0 = p1.y;
    let mut x1 = p2.x;
    let mut y1 = p2.y;

    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    if steep {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let gradient = if dx == 0.0 { 1.0 } else { dy / dx };

    let mut y = y0;
    for x in (x0.round() as i32)..=(x1.round() as i32) {
        let y_floor = y.floor();
        let fractional_part = y - y_floor;

        let intensity1 = 1.0 - fractional_part;
        let intensity2 = fractional_part;

        if steep {
            pixels.push((y.floor() as i32, x, intensity1));
            pixels.push((y.floor() as i32 + 1, x, intensity2));
        } else {
            pixels.push((x, y.floor() as i32, intensity1));
            pixels.push((x, y.floor() as i32 + 1, intensity2));
        }

        if logger.can_write() {
            logger.log(format!(
                "x: {}, y: {:.2}, пиксели: ({}, {}, {:.2}), ({}, {}, {:.2})",
                x, y,
                if steep {y.floor() as i32} else {x}, if steep {x} else {y.floor() as i32}, intensity1,
                if steep {y.floor() as i32 + 1} else {x}, if steep {x} else {y.floor() as i32 + 1}, intensity2
            ));
        }
        y += gradient;
    }
    pixels
}

pub fn step_by_step_aa<L: Logger>(p1: Pos2, p2: Pos2, logger: &mut L) -> Vec<AntialiasedPixel> {
    let mut pixels = Vec::new();
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    if dx.abs() > dy.abs() {
        let k = dy / dx;
        let b = p1.y - k * p1.x;
        let (start_x, end_x) = if p1.x < p2.x { (p1.x, p2.x) } else { (p2.x, p1.x) };

        for x_int in (start_x.round() as i32)..=(end_x.round() as i32) {
            let x = x_int as f32;
            let y_ideal = k * x + b;
            let y_floor = y_ideal.floor();
            let fractional = y_ideal - y_floor;

            let y1 = y_floor as i32;
            let y2 = y1 + 1;
            let intensity1 = 1.0 - fractional;
            let intensity2 = fractional;

            pixels.push((x_int, y1, intensity1));
            pixels.push((x_int, y2, intensity2));

            if logger.can_write() {
                logger.log(format!("x: {}, y_ideal: {:.2} -> ({}, {:.2}), ({}, {:.2})", x_int, y_ideal, y1, intensity1, y2, intensity2));
            }
        }
    } else {
        // Аналогичная логика для dy > dx
        let k = dx / dy;
        let b = p1.x - k * p1.y;
        let (start_y, end_y) = if p1.y < p2.y { (p1.y, p2.y) } else { (p2.y, p1.y) };

        for y_int in (start_y.round() as i32)..=(end_y.round() as i32) {
            let y = y_int as f32;
            let x_ideal = k * y + b;
            let x_floor = x_ideal.floor();
            let fractional = x_ideal - x_floor;

            let x1 = x_floor as i32;
            let x2 = x1 + 1;
            let intensity1 = 1.0 - fractional;
            let intensity2 = fractional;

            pixels.push((x1, y_int, intensity1));
            pixels.push((x2, y_int, intensity2));

            if logger.can_write() {
                logger.log(format!("y: {}, x_ideal: {:.2} -> ({}, {:.2}), ({}, {:.2})", y_int, x_ideal, x1, intensity1, x2, intensity2));
            }
        }
    }
    pixels
}

/// Алгоритм ЦДА со сглаживанием
pub fn dda_aa<L: Logger>(p1: Pos2, p2: Pos2, logger: &mut L) -> Vec<AntialiasedPixel> {
    let mut pixels = Vec::new();
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    let steps = if dx.abs() > dy.abs() { dx.abs() } else { dy.abs() };
    let x_inc = dx / steps;
    let y_inc = dy / steps;

    let mut x_ideal = p1.x;
    let mut y_ideal = p1.y;

    let is_steep = dy.abs() > dx.abs();

    for _ in 0..=steps.round() as u32 {
        if is_steep {
            let x_floor = x_ideal.floor();
            let fractional = x_ideal - x_floor;
            let x1 = x_floor as i32;
            let x2 = x1 + 1;
            pixels.push((x1, y_ideal.round() as i32, 1.0 - fractional));
            pixels.push((x2, y_ideal.round() as i32, fractional));
        } else {
            let y_floor = y_ideal.floor();
            let fractional = y_ideal - y_floor;
            let y1 = y_floor as i32;
            let y2 = y1 + 1;
            pixels.push((x_ideal.round() as i32, y1, 1.0 - fractional));
            pixels.push((x_ideal.round() as i32, y2, fractional));
        }

        if logger.can_write() {
            logger.log(format!("x: {:.2}, y: {:.2}", x_ideal, y_ideal));
        }

        x_ideal += x_inc;
        y_ideal += y_inc;
    }
    pixels
}

pub fn bresenham_aa<L: Logger>(p1: Pos2, p2: Pos2, logger: &mut L) -> Vec<AntialiasedPixel> {
    let mut pixels = Vec::new();
    let x1 = p1.x.round() as i32;
    let y1 = p1.y.round() as i32;
    let x2 = p2.x.round() as i32;
    let y2 = p2.y.round() as i32;

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();

    // Определяем направление шага
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };

    if dx >= dy {
        // --- Пологая линия (доминантная ось X) ---
        let mut d = 0; // Накопитель ошибки (числитель дроби d/dx)
        let mut y = y1;
        let mut x = x1;

        // Итерируемся ровно dx раз
        for _ in 0..=dx {
            // Интенсивность = насколько далеко мы ушли от центра пикселя y
            // d / dx  -> число от 0.0 до 1.0
            let intensity = d as f32 / dx as f32;

            // Основной пиксель (чем меньше отклонение d, тем он ярче)
            pixels.push((x, y, 1.0 - intensity));
            // Соседний пиксель (получает остаток интенсивности)
            pixels.push((x, y + sy, intensity));

            if logger.can_write() {
                logger.log(format!("x: {}, y: {}, d: {}/{} -> int: {:.2}", x, y, d, dx, intensity));
            }

            d += dy;
            // Если накопитель превысил порог (1.0 или dx в целых числах)
            if d >= dx {
                d -= dx; // Сбрасываем ошибку (эквивалент fract() в float)
                y += sy; // Переходим на следующую строку
            }
            x += sx;
        }
    } else {
        // --- Крутая линия (доминантная ось Y) ---
        // Логика та же, только меняем роли x и y, dx и dy
        let mut d = 0;
        let mut x = x1;
        let mut y = y1;

        for _ in 0..=dy {
            let intensity = d as f32 / dy as f32;

            pixels.push((x, y, 1.0 - intensity));
            pixels.push((x + sx, y, intensity));

            if logger.can_write() {
                logger.log(format!("y: {}, x: {}, d: {}/{} -> int: {:.2}", y, x, d, dy, intensity));
            }

            d += dx;
            if d >= dy {
                d -= dy;
                x += sx;
            }
            y += sy;
        }
    }

    pixels
}