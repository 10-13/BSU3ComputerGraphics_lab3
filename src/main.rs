mod app;
mod algo;
mod logger;

use app::GraphicsLabApp;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Лабораторная работа №3: Растровые алгоритмы",
        native_options,
        Box::new(|_cc| Box::new(GraphicsLabApp::default())),
    );
}