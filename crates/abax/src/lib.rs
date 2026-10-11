pub struct App {
    count: u32,
}

impl Default for App {
    fn default() -> Self {
        Self { count: 0 }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Abax spike");
            ui.label(abax_core::hello());
            if ui.button(format!("Tapped {} times", self.count)).clicked() {
                self.count += 1;
            }
        });
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use eframe::NativeOptions;
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let mut options = NativeOptions::default();
    options.android_app = Some(app);
    eframe::run_native("Abax", options, Box::new(|_cc| Ok(Box::new(App::default())))).unwrap();
}
