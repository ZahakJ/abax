#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let bench = match std::env::var("ABAX_BENCH").ok().as_deref() {
        Some("start") => Some(abax::Bench::Start),
        Some(s) if s.starts_with("frames") => Some(abax::Bench::Frames(
            s.trim_start_matches("frames=").parse().unwrap_or(300),
        )),
        _ => None,
    };
    let icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../../../packaging/icon-256.png")).ok();
    let size: [f32; 2] = std::env::var("ABAX_WINDOW")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some([w.parse().ok()?, h.parse().ok()?])
        })
        .unwrap_or([960.0, 680.0]);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Abax")
        .with_app_id("abax")
        .with_inner_size(size)
        .with_min_inner_size([360.0, 480.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }
    if std::env::var("ABAX_SHOT").is_ok() {
        // A fixed-size window is floated by tiling compositors.
        viewport = viewport
            .with_resizable(false)
            .with_min_inner_size(size)
            .with_max_inner_size(size)
            .with_decorations(false);
    }
    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    if bench.is_some() {
        // Present without vsync so the measured frame period reflects the
        // real work, not the display refresh. AutoNoVsync is supported on
        // every surface (Immediate is not, and wgpu panics on unsupported
        // modes).
        options.wgpu_options.surface.present_mode = eframe::wgpu::PresentMode::AutoNoVsync;
    }
    eframe::run_native(
        "Abax",
        options,
        Box::new(move |cc| Ok(Box::new(abax::App::new(cc, bench)))),
    )
}

#[cfg(target_os = "android")]
fn main() {}
