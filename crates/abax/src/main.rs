#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions::default();
    eframe::run_native("Abax", options, Box::new(|_cc| Ok(Box::new(abax::App::default()))))
}

#[cfg(target_os = "android")]
fn main() {}
