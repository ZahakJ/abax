//! Abax: a soroban trainer. This crate is the eframe application; the logic
//! lives in `abax-core`.

pub mod audio;
pub mod charts;
pub mod keypad;
pub mod render;
pub mod screens;
pub mod soroban_view;
pub mod storage;
pub mod theme;
pub mod ui;

use std::collections::VecDeque;
use std::time::Instant;

use abax_core::curriculum::SkillId;
use abax_core::scheduler::{DrillSpec, Rng, SessionPlan};
use abax_core::store::{SaveFile, Theme};
use egui::Ui;

use screens::drill::{DrillMode, DrillState};
use screens::flash::FlashState;
use screens::lesson::LessonState;
use screens::picker::PickerState;
use screens::stats::StatsState;
use soroban_view::{InputMode, SorobanView};
use theme::{Material, Palette};

#[allow(clippy::large_enum_variant)]
pub enum Screen {
    Home,
    Practice,
    Lessons,
    Lesson(LessonState),
    Drill(DrillState),
    Flash(FlashState),
    Picker(PickerState),
    Stats(StatsState),
    Settings,
    /// Transient value while a screen is being drawn.
    Placeholder,
}

/// Benchmark modes driven by the `ABAX_BENCH` environment variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bench {
    /// Log the time to the first frame, then exit.
    Start,
    /// Animate 27 rods for N frames, log frame-time statistics, then exit.
    Frames(u32),
}

/// Screenshot automation driven by `ABAX_SHOT=<screen>:<path>` (desktop).
/// A path ending in `/` is a directory: the app then records a sequence of
/// [`SEQUENCE_FRAMES`] consecutive frames (`frame-000.png` ...) while the
/// practice soroban runs a scripted set of moves (the landing-page hero).
#[derive(Clone, Debug)]
pub struct Shot {
    pub screen: String,
    pub path: String,
    frames: u32,
    /// Sequence frames written so far.
    written: u32,
}

/// Frames in a screenshot sequence.
pub const SEQUENCE_FRAMES: u32 = 48;
/// The frame the sequence starts on.
const SEQUENCE_START: u32 = 10;

impl Shot {
    pub fn from_env() -> Option<Self> {
        let v = std::env::var("ABAX_SHOT").ok()?;
        let (screen, path) = v.split_once(':')?;
        Some(Self {
            screen: screen.to_string(),
            path: path.to_string(),
            frames: 0,
            written: 0,
        })
    }

    pub fn is_sequence(&self) -> bool {
        self.path.ends_with('/')
    }

    /// The scripted value for a sequence frame, if the soroban changes then.
    pub fn sequence_value(frame: u32) -> Option<u128> {
        match frame {
            10 => Some(4_071_583),
            26 => Some(2_958),
            40 => Some(7_310_246),
            _ => None,
        }
    }
}

pub struct App {
    pub save: SaveFile,
    pub screen: Screen,
    pub view: SorobanView,
    /// The small decorative soroban on Home (the identity mark).
    pub home_view: SorobanView,
    pub rng: Rng,
    pub audio: audio::Audio,
    pub palette: Palette,
    /// The instrument's finish for the current theme mode.
    pub material: Material,
    pub confirm_reset: bool,
    pub pending_drills: Vec<DrillSpec>,
    /// Width of the last frame in points, for layout decisions made outside
    /// the frame (e.g. how many rods a drill shows).
    pub last_width: f32,
    dirty: bool,
    last_save: Instant,
    style_key: Option<(bool, bool, abax_core::store::Finish)>,
    started: Instant,
    first_frame: bool,
    frame_times: VecDeque<f32>,
    last_frame: Option<Instant>,
    bench: Option<Bench>,
    bench_frames: u32,
    /// UI-pass time per frame (top of `ui` to end of the central panel), ms.
    bench_samples: Vec<f32>,
    /// Frame period per frame (interval between successive `ui` calls), ms.
    bench_periods: Vec<f32>,
    /// eframe's CPU time for the previous frame (ui + tessellation + paint
    /// submission, without waiting for present), ms.
    bench_cpu: Vec<f32>,
    shot: Option<Shot>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, bench: Option<Bench>) -> Self {
        // Taken first so the start benchmark includes everything below
        // (profile load and the soroban pipeline creation).
        let started = Instant::now();
        let mut save = storage::load();
        if cfg!(target_os = "android") && save.history.is_empty() && save.settings.rod_count == 13 {
            // Phones: fewer rods so beads are large enough to tap.
            save.settings.rod_count = 9;
        }
        let mut view = SorobanView::new(save.settings.rod_count.clamp(1, 40));
        view.show_digits = save.settings.show_numbers;
        let mut home_view = SorobanView::new(7);
        home_view.interactive = false;
        home_view.show_digits = false;
        if let Some(rs) = cc.wgpu_render_state.as_ref() {
            let t = Instant::now();
            render::install(rs, &cc.egui_ctx);
            log::info!(
                "soroban pipeline created in {:.1} ms",
                t.elapsed().as_secs_f32() * 1000.0
            );
        } else {
            log::warn!("no wgpu render state: drawing the soroban with the classic painter");
        }
        let seed = storage::now_unix() ^ 0xA5A5_5A5A_1234_5678;
        let mut app = Self {
            save,
            screen: Screen::Home,
            view,
            home_view,
            rng: Rng::new(seed),
            audio: audio::Audio::new(),
            palette: theme::LIGHT,
            material: theme::material(Default::default(), false),
            confirm_reset: false,
            pending_drills: Vec::new(),
            last_width: 960.0,
            dirty: false,
            last_save: Instant::now(),
            style_key: None,
            started,
            first_frame: false,
            frame_times: VecDeque::with_capacity(128),
            last_frame: None,
            bench,
            bench_frames: 0,
            bench_samples: Vec::new(),
            bench_periods: Vec::new(),
            bench_cpu: Vec::new(),
            shot: Shot::from_env(),
        };
        if std::env::var("ABAX_DEMO_DATA").is_ok() {
            app.seed_demo_data();
        }
        app.apply_settings();
        app.refresh_style(&cc.egui_ctx);
        if let Some(Bench::Frames(_)) = bench {
            app.view.set_rod_count(27);
            app.screen = Screen::Practice;
        }
        app
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn save_now(&mut self) {
        self.save.updated = storage::now_unix();
        match storage::save(&self.save) {
            Ok(()) => self.dirty = false,
            Err(e) => log::warn!("could not save profile: {e}"),
        }
        self.last_save = Instant::now();
    }

    /// Push settings into the live widgets.
    pub fn apply_settings(&mut self) {
        let s = &self.save.settings;
        self.view.set_rod_count(s.rod_count.clamp(1, 40));
        self.view.input = match s.input_mode {
            abax_core::store::InputMode::Tap => InputMode::Tap,
            abax_core::store::InputMode::Drag => InputMode::Drag,
            abax_core::store::InputMode::Both => InputMode::Both,
        };
        self.view.motion.reduce = s.reduce_motion;
        self.home_view.motion.reduce = s.reduce_motion;
        self.view.show_digits = s.show_numbers;
        self.audio.enabled = s.sound;
        self.audio.volume = s.volume;
        self.style_key = None;
    }

    fn refresh_style(&mut self, ctx: &egui::Context) {
        let dark = match self.save.settings.theme {
            Theme::Light => false,
            Theme::Dark => true,
            Theme::System => ctx
                .system_theme()
                .map(|t| t == egui::Theme::Dark)
                .unwrap_or(false),
        };
        let key = (
            dark,
            self.save.settings.large_text,
            self.save.settings.finish,
        );
        if self.style_key != Some(key) {
            self.palette = if dark { theme::DARK } else { theme::LIGHT };
            self.material = theme::material(self.save.settings.finish, dark);
            theme::apply_style(ctx, &self.palette, self.save.settings.large_text);
            self.style_key = Some(key);
        }
    }

    pub fn go_home(&mut self) {
        self.view
            .set_rod_count(self.save.settings.rod_count.clamp(1, 40));
        self.view.interactive = true;
        self.view.hint.clear();
        self.view.fade = 0.0;
        self.view.show_digits = self.save.settings.show_numbers;
        self.view.active_rod = None;
        self.pending_drills.clear();
        self.screen = Screen::Home;
    }

    pub fn start_drill(&mut self, spec: DrillSpec, mode: DrillMode) {
        let state = DrillState::new(self, spec, mode);
        self.screen = Screen::Drill(state);
    }

    pub fn start_flash(&mut self, skill: SkillId) {
        let state = FlashState::new(self, skill);
        self.screen = Screen::Flash(state);
    }

    /// Run a planned session: warm-up, focus, cool-down in turn.
    pub fn start_session(&mut self, plan: SessionPlan) {
        self.pending_drills.clear();
        let mut specs = Vec::new();
        if let Some(w) = plan.warmup {
            specs.push(w);
        }
        specs.push(plan.focus);
        if let Some(c) = plan.cooldown {
            specs.push(c);
        }
        let first = specs.remove(0);
        self.pending_drills = specs;
        self.start_pending_or(first);
    }

    fn start_pending_or(&mut self, spec: DrillSpec) {
        if matches!(spec.skill, SkillId::FlashOne | SkillId::FlashMulti) {
            self.start_flash(spec.skill);
        } else if spec.skill == SkillId::Anzan {
            self.start_drill(spec, DrillMode::Anzan { fade_seconds: 2.5 });
        } else {
            self.start_drill(spec, DrillMode::Normal);
        }
    }

    pub fn start_next_pending(&mut self) -> bool {
        if self.pending_drills.is_empty() {
            return false;
        }
        let next = self.pending_drills.remove(0);
        self.start_pending_or(next);
        true
    }

    /// Synthetic history for screenshots and manual testing: a learner who
    /// practised for three weeks.
    pub fn seed_demo_data(&mut self) {
        use abax_core::stats::{Attempt, DrillKind};
        let now = storage::now_unix();
        let mut rng = Rng::new(7);
        let skills = [
            SkillId::Setting,
            SkillId::AddSimple,
            SkillId::SubSimple,
            SkillId::AddFive,
            SkillId::SubFive,
            SkillId::AddTen,
            SkillId::SubTen,
            SkillId::Combined,
        ];
        for day in (0..21).rev() {
            if day % 5 == 3 {
                continue; // a rest day
            }
            let day_start = now - day as u64 * 86400 - 3600 * 9;
            let mut t = day_start;
            // Skills enter one by one over the three weeks.
            let active = ((21 - day) / 3 + 1).min(skills.len());
            for _ in 0..(18 + rng.below(14)) {
                let skill = skills[(rng.below(active as u32) as usize).min(active - 1)];
                let mastery = (21 - day) as f64 / 21.0;
                let p = 0.6 + 0.35 * mastery;
                let correct = rng.next_f64() < p;
                let secs = (6.0 - 3.5 * mastery + rng.next_f64() * 2.0) as f32;
                self.save.scheduler.record(skill, correct, secs as f64, t);
                self.save.history.push(Attempt {
                    ts: t,
                    skill,
                    level: 0,
                    kind: DrillKind::AddSub,
                    correct,
                    seconds: secs,
                });
                t += secs as u64 + 2;
            }
        }
        // The learner skipped the anzan lesson: "I know this already".
        self.save.scheduler.mark_known(SkillId::Anzan);
    }

    fn run_shot(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.shot else {
            return;
        };
        shot.frames += 1;
        let frames = shot.frames;
        let screen = shot.screen.clone();
        let sequence = shot.is_sequence();
        ctx.request_repaint();
        if frames == 2 {
            self.navigate_for_shot(&screen);
            if sequence {
                if let Some(dir) = self.shot.as_ref().map(|s| s.path.clone()) {
                    let _ = std::fs::create_dir_all(dir);
                }
                // The scripted moves start from an empty soroban.
                let empty = abax_core::Soroban::new(self.view.rod_count());
                self.view.snap_to(&empty);
            }
        } else if sequence && frames >= SEQUENCE_START {
            let now = ctx.input(|i| i.time);
            if let Some(v) = Shot::sequence_value(frames) {
                let mut target = self.view.model.clone();
                target.set_value(v);
                let d = self.view.motion.demo_move;
                self.view.animate_to(&target, now, d);
            }
            if frames < SEQUENCE_START + SEQUENCE_FRAMES {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            if frames > SEQUENCE_START + SEQUENCE_FRAMES + 200 {
                log::warn!("screenshot sequence incomplete; exiting");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else if !sequence && frames == 40 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        } else if !sequence && frames > 200 {
            log::warn!("screenshot never arrived; exiting");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn navigate_for_shot(&mut self, screen: &str) {
        let now = 0.0;
        match screen {
            "practice" => {
                self.screen = Screen::Practice;
                self.view.set_value(4_071_583, now);
            }
            "lesson" => {
                self.screen = Screen::Lesson(LessonState::new(SkillId::AddFive));
            }
            "lesson-gap" => {
                // A lesson opened ahead of the suggested path: S11 builds on
                // S10, which the demo learner has not met.
                self.screen = Screen::Lesson(LessonState::new(SkillId::Divide));
            }
            "glass" => {
                // The anzan fade part-way: beads turning to glass.
                self.screen = Screen::Practice;
                self.view.set_value(4_071_583, now);
                self.view.fade = 0.55;
            }
            "drill" => {
                let skill = SkillId::AddTen;
                let spec = DrillSpec {
                    skill,
                    level: skill.ladder()[1],
                    items: 10,
                    interleave: Vec::new(),
                    target_success: 0.8,
                };
                self.start_drill(spec, DrillMode::Normal);
            }
            "stats" => self.screen = Screen::Stats(StatsState::default()),
            "lessons" => self.screen = Screen::Lessons,
            "flash" => self.start_flash(SkillId::Anzan),
            "flash-run" => {
                let mut st = FlashState::new(self, SkillId::FlashMulti);
                // Long enough that frame 40 lands on a shown number.
                st.interval_ms = 3000;
                st.begin(self, -0.8);
                self.screen = Screen::Flash(st);
            }
            "settings" => self.screen = Screen::Settings,
            _ => self.screen = Screen::Home,
        }
    }

    #[cfg(target_os = "android")]
    fn save_screenshot(&self, _image: &egui::ColorImage, _path: &str) {}

    #[cfg(not(target_os = "android"))]
    fn save_screenshot(&self, image: &egui::ColorImage, path: &str) {
        let file = match std::fs::File::create(path) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("cannot write {path}: {e}");
                return;
            }
        };
        let w = std::io::BufWriter::new(file);
        let mut enc = png::Encoder::new(w, image.width() as u32, image.height() as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        if let Ok(mut writer) = enc.write_header() {
            let _ = writer.write_image_data(image.as_raw());
        }
        println!("ABAX_SHOT wrote {path}");
    }

    /// (average, p95) frame time in milliseconds over the recent window.
    pub fn frame_stats(&self) -> (f32, f32) {
        if self.frame_times.is_empty() {
            return (0.0, 0.0);
        }
        let mut v: Vec<f32> = self.frame_times.iter().copied().collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg = v.iter().sum::<f32>() / v.len() as f32;
        let p95 = v[((v.len() as f32 * 0.95) as usize).min(v.len() - 1)];
        (avg, p95)
    }

    /// Records the interval since the previous `ui` call and returns it (ms).
    fn record_frame(&mut self) -> Option<f32> {
        let now = Instant::now();
        let mut out = None;
        if let Some(prev) = self.last_frame {
            let dt = now.duration_since(prev).as_secs_f32() * 1000.0;
            if self.frame_times.len() >= 120 {
                self.frame_times.pop_front();
            }
            self.frame_times.push_back(dt);
            out = Some(dt);
        }
        self.last_frame = Some(now);
        out
    }

    fn run_bench(
        &mut self,
        ctx: &egui::Context,
        ui_pass_ms: f32,
        period_ms: Option<f32>,
        cpu_ms: Option<f32>,
    ) {
        let Some(b) = self.bench else {
            return;
        };
        match b {
            Bench::Start => {
                if !self.first_frame {
                    self.first_frame = true;
                    let ms = self.started.elapsed().as_secs_f32() * 1000.0;
                    println!("ABAX_BENCH start-to-first-frame: {ms:.1} ms (App::new to first ui)");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Bench::Frames(n) => {
                // Move every rod every frame so all 135 beads animate.
                let now = ctx.input(|i| i.time);
                let rods = self.view.rod_count();
                for i in 0..rods {
                    let d = self.rng.below(10) as u8;
                    self.view
                        .set_rod(i, abax_core::Rod::from_digit(d), now, 0.09, 0.0);
                }
                self.bench_frames += 1;
                if self.bench_frames > 10 {
                    self.bench_samples.push(ui_pass_ms);
                    if let Some(p) = period_ms {
                        self.bench_periods.push(p);
                    }
                    if let Some(c) = cpu_ms {
                        self.bench_cpu.push(c);
                    }
                }
                ctx.request_repaint();
                if self.bench_frames == n {
                    let (ui_avg, ui_p95) = avg_p95(&self.bench_samples);
                    let (fp_avg, fp_p95) = avg_p95(&self.bench_periods);
                    let (cpu_avg, cpu_p95) = avg_p95(&self.bench_cpu);
                    println!(
                        "ABAX_BENCH frames={} rods={} ui-pass avg/p95={ui_avg:.2}/{ui_p95:.2} ms cpu-frame avg/p95={cpu_avg:.2}/{cpu_p95:.2} ms frame period avg/p95={fp_avg:.2}/{fp_p95:.2} ms",
                        self.bench_samples.len(),
                        rods
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn show_screen(&mut self, ui: &mut Ui) {
        // Take the screen out so screens can borrow `self` mutably. A screen
        // navigates by assigning `self.screen`; we detect that by checking
        // whether the placeholder was replaced.
        let mut screen = std::mem::replace(&mut self.screen, Screen::Placeholder);
        match &mut screen {
            Screen::Home => screens::home::show(self, ui),
            Screen::Practice => screens::practice::show(self, ui),
            Screen::Lessons => screens::lesson::list(self, ui),
            Screen::Lesson(st) => st.show(self, ui),
            Screen::Drill(st) => st.show(self, ui),
            Screen::Flash(st) => st.show(self, ui),
            Screen::Picker(st) => st.show(self, ui),
            Screen::Stats(st) => st.show(self, ui),
            Screen::Settings => screens::settings::show(self, ui),
            Screen::Placeholder => {}
        }
        if matches!(self.screen, Screen::Placeholder) {
            self.screen = screen;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ui_start = Instant::now();
        let ctx = ui.ctx().clone();
        self.refresh_style(&ctx);
        let period_ms = self.record_frame();
        self.last_width = ctx.content_rect().width();
        self.view.material = self.material;
        self.home_view.material = self.material;
        let fill = self.palette.paper;
        // On Android the NativeActivity surface still extends under the status
        // bar: the theme's windowOptOutEdgeToEdgeEnforcement (values-v35) was
        // tested on an API 35 emulator and did not move the content, so the
        // Back/Clear row overlapped the clock. Reserve room for a typical
        // 24-30 dp status bar here instead.
        let top = if cfg!(target_os = "android") { 34 } else { 10 };
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(fill).inner_margin(egui::Margin {
                left: 12,
                right: 12,
                top,
                bottom: 10,
            }))
            .show(ui, |ui| {
                self.show_screen(ui);
            });
        let ui_pass_ms = ui_start.elapsed().as_secs_f32() * 1000.0;
        let cpu_ms = frame.info().cpu_usage.map(|s| s * 1000.0);
        self.run_bench(&ctx, ui_pass_ms, period_ms, cpu_ms);
        self.run_shot(&ctx);
        if self.dirty && self.last_save.elapsed().as_secs_f32() > 5.0 {
            self.save_now();
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        if self.dirty {
            self.save_now();
        }
    }

    fn on_exit(&mut self) {
        if self.dirty {
            self.save_now();
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.palette.paper.to_normalized_gamma_f32()
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        let Some(shot) = &self.shot else {
            return;
        };
        let sequence = shot.is_sequence();
        let path = shot.path.clone();
        for ev in &raw_input.events {
            if let egui::Event::Screenshot { image, .. } = ev {
                if sequence {
                    // Screenshots can arrive a frame late; each one simply
                    // goes to the next file.
                    let n = self.shot.as_ref().map_or(0, |s| s.written);
                    if n < SEQUENCE_FRAMES {
                        self.save_screenshot(image, &format!("{path}frame-{n:03}.png"));
                        if let Some(s) = &mut self.shot {
                            s.written += 1;
                        }
                    }
                    if n + 1 >= SEQUENCE_FRAMES {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                } else {
                    self.save_screenshot(image, &path);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use eframe::NativeOptions;
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    if let Some(p) = app.internal_data_path() {
        storage::set_data_dir(p);
    }
    let mut options = NativeOptions::default();
    options.android_app = Some(app);
    eframe::run_native(
        "Abax",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, None)))),
    )
    .unwrap();
}

/// (average, p95) of a sample set, in the samples' unit.
fn avg_p95(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }
    let mut v = samples.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg = v.iter().sum::<f32>() / v.len() as f32;
    let p95 = v[((v.len() as f32 * 0.95) as usize).min(v.len() - 1)];
    (avg, p95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shot_sequence_script() {
        let shot = Shot {
            screen: "practice".into(),
            path: "out/".into(),
            frames: 0,
            written: 0,
        };
        assert!(shot.is_sequence());
        let changes: Vec<u32> = (0..200)
            .filter(|f| Shot::sequence_value(*f).is_some())
            .collect();
        assert_eq!(changes, vec![10, 26, 40]);
        assert!(changes
            .iter()
            .all(|f| *f >= SEQUENCE_START && *f < SEQUENCE_START + SEQUENCE_FRAMES));
    }
}
