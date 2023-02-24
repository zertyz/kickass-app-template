#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[path = "../../src/frontend/egui/mod.rs"]
mod app_egui;
use app_egui::Egui;


fn main() -> eframe::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    return Egui::run_egui_native_app();
    #[cfg(target_arch = "wasm32")]
    return Egui::run_egui_web_app();
}
