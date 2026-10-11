//! A lesson: plain explanation, then demonstrations with highlighted beads
//! and the finger to use, then guided practice.

use abax_core::curriculum::{lesson, Demo, SkillId};
use abax_core::moves::{add_number, decimal_digits, steps_to_show, sub_number};
use abax_core::{Finger, Rule, Soroban, Step, StepKind};
use egui::{Align, Layout, RichText, Ui};

use crate::screens::drill::DrillMode;
use crate::App;

pub struct LessonState {
    pub skill: SkillId,
    demo_idx: usize,
    steps: Vec<Step>,
    step_idx: usize,
    start_value: u128,
    text: String,
    needs_setup: bool,
    next_at: f64,
    autoplay: bool,
    procedure: Option<abax_core::arith::Procedure>,
}

impl LessonState {
    pub fn new(skill: SkillId) -> Self {
        Self {
            skill,
            demo_idx: 0,
            steps: Vec::new(),
            step_idx: 0,
            start_value: 0,
            text: String::new(),
            needs_setup: true,
            next_at: 0.0,
            autoplay: true,
            procedure: None,
        }
    }

    fn setup_demo(&mut self, app: &mut App, now: f64) {
        self.needs_setup = false;
        self.step_idx = 0;
        self.procedure = None;
        app.view.hint.clear();
        app.view.show_digits = true;
        app.view.interactive = false;
        app.view.fade = 0.0;
        let demos = lesson(self.skill).demos;
        // A lesson shows a small soroban so the beads are large.
        let wide = app.last_width > 700.0;
        let rods = match demos.get(self.demo_idx) {
            Some(Demo::Mul(a, b)) => {
                abax_core::arith::rods_for_multiply(a.to_string().len(), b.to_string().len())
            }
            Some(Demo::Div(a, b)) => {
                abax_core::arith::rods_for_divide(a.to_string().len(), b.to_string().len())
            }
            _ => 5,
        }
        .max(if wide { 9 } else { 5 });
        app.view.set_rod_count(rods);
        let Some(demo) = demos.get(self.demo_idx) else {
            self.steps.clear();
            return;
        };
        let mut s = Soroban::new(rods);
        match *demo {
            Demo::Set(n) => {
                self.start_value = 0;
                self.steps = steps_to_show(&s, n);
                self.text = format!("Set {n}.");
            }
            Demo::Add(a, b) => {
                s.set_value(a);
                self.start_value = a;
                self.steps = add_number(&mut s, b).unwrap_or_default();
                self.text = format!("{a} + {b}");
            }
            Demo::Sub(a, b) => {
                s.set_value(a);
                self.start_value = a;
                self.steps = sub_number(&mut s, b).unwrap_or_default();
                self.text = format!("{a} − {b}");
            }
            Demo::Mul(a, b) => {
                self.start_value = 0;
                self.procedure = abax_core::arith::multiply(a, b, rods).ok();
                self.steps = Vec::new();
                self.text = format!("{a} × {b}");
            }
            Demo::Div(a, b) => {
                self.start_value = 0;
                self.procedure = abax_core::arith::divide(a, b, rods).ok();
                self.steps = Vec::new();
                self.text = format!("{a} ÷ {b}");
            }
        }
        let mut start = Soroban::new(rods);
        start.set_value(self.start_value);
        app.view.animate_to(&start, now, 0.3);
        self.next_at = now + 1.2;
        let _ = decimal_digits(0);
    }

    fn total_steps(&self) -> usize {
        match &self.procedure {
            Some(p) => p.steps.len(),
            None => self.steps.len(),
        }
    }

    /// Advance one demonstrated move.
    fn play_next(&mut self, app: &mut App, now: f64) {
        if let Some(p) = &self.procedure {
            if let Some(ps) = p.steps.get(self.step_idx) {
                for st in &ps.beads {
                    app.view.apply_step(st, now, 0.4);
                }
                app.view.hint = ps.beads.clone();
                app.view.active_rod = ps.focus.first().copied();
                self.step_idx += 1;
                self.next_at = now + 2.6;
            }
            return;
        }
        if let Some(st) = self.steps.get(self.step_idx).copied() {
            app.view.hint = vec![st];
            app.view.apply_step(&st, now, 0.4);
            self.step_idx += 1;
            self.next_at = now + 1.6;
        }
    }

    pub fn show(&mut self, app: &mut App, ui: &mut Ui) {
        let now = ui.input(|i| i.time);
        let pal = app.palette;
        if self.needs_setup {
            self.setup_demo(app, now);
        }
        let l = lesson(self.skill);
        let title = format!("Lesson {}", self.skill.code());
        if crate::ui::header(ui, &pal, Some("Lessons"), &title, |_| {}) {
            app.view.interactive = true;
            app.screen = crate::Screen::Lessons;
            return;
        }
        ui.label(
            RichText::new(l.title)
                .size(crate::ui::size::CARD_TITLE + 4.0)
                .strong()
                .color(pal.ink),
        );
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() * 0.38)
            .show(ui, |ui| {
                for p in l.paragraphs {
                    ui.label(RichText::new(*p).color(pal.ink));
                    ui.add_space(4.0);
                }
            });
        crate::ui::rule(ui, &pal, 4.0);

        let has_demos = !l.demos.is_empty();
        if has_demos {
            let n = l.demos.len();
            if ui.available_width() < 640.0 {
                // Narrow: label on its own line, controls below, left to right.
                ui.label(
                    RichText::new(format!("Demo {}/{}: {}", self.demo_idx + 1, n, self.text))
                        .strong(),
                );
                ui.horizontal_wrapped(|ui| {
                    if self.demo_idx > 0 && ui.button("Previous").clicked() {
                        self.demo_idx -= 1;
                        self.needs_setup = true;
                    }
                    if self.demo_idx + 1 < n && ui.button("Next demo").clicked() {
                        self.demo_idx += 1;
                        self.needs_setup = true;
                    }
                    if ui.button("Replay").clicked() {
                        self.needs_setup = true;
                    }
                    ui.checkbox(&mut self.autoplay, "Auto");
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Demonstration {} of {}: {}",
                            self.demo_idx + 1,
                            n,
                            self.text
                        ))
                        .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Replay").clicked() {
                            self.needs_setup = true;
                        }
                        if self.demo_idx + 1 < n && ui.button("Next demo").clicked() {
                            self.demo_idx += 1;
                            self.needs_setup = true;
                        }
                        if self.demo_idx > 0 && ui.button("Previous").clicked() {
                            self.demo_idx -= 1;
                            self.needs_setup = true;
                        }
                        ui.checkbox(&mut self.autoplay, "Auto");
                    });
                });
            }
            // Explanation of the current/last move.
            let desc = if let Some(p) = &self.procedure {
                self.step_idx
                    .checked_sub(1)
                    .and_then(|i| p.steps.get(i))
                    .map(|s| s.text.clone())
                    .unwrap_or_else(|| "Watch the rods.".to_string())
            } else {
                self.step_idx
                    .checked_sub(1)
                    .and_then(|i| self.steps.get(i))
                    .map(describe_step)
                    .unwrap_or_else(|| "Watch the highlighted beads.".to_string())
            };
            ui.label(RichText::new(desc).color(pal.accent));
            if self.autoplay && self.step_idx < self.total_steps() && now >= self.next_at {
                self.play_next(app, now);
            }
            if self.step_idx >= self.total_steps() && now >= self.next_at && !self.steps.is_empty()
            {
                app.view.hint.clear();
            }
            ui.ctx().request_repaint_after_secs(0.1);
        }

        let avail = ui.available_size();
        let h = app
            .view
            .natural_height(avail.x)
            .min((avail.y - 54.0).max(120.0));
        let _ = app.view.show(ui, &pal, egui::vec2(avail.x, h));

        ui.horizontal(|ui| {
            if has_demos && self.step_idx < self.total_steps() && ui.button("Next move").clicked() {
                self.play_next(app, now);
                self.autoplay = false;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = if self.skill.is_anzan()
                    || matches!(self.skill, SkillId::Multiply | SkillId::Divide)
                {
                    "Start drilling"
                } else {
                    "Try it with hints"
                };
                if crate::ui::primary_button(ui, &pal, label, egui::vec2(0.0, 0.0)).clicked() {
                    app.view.interactive = true;
                    let level = app.save.scheduler.current_level(self.skill);
                    let spec = abax_core::scheduler::DrillSpec {
                        skill: self.skill,
                        level,
                        items: 8,
                        interleave: Vec::new(),
                        target_success: 0.8,
                    };
                    let mode = if label == "Try it with hints" {
                        DrillMode::Guided
                    } else {
                        DrillMode::Normal
                    };
                    if self.skill == SkillId::FlashOne || self.skill == SkillId::FlashMulti {
                        app.start_flash(self.skill);
                    } else {
                        app.start_drill(spec, mode);
                    }
                }
            });
        });
    }
}

/// Plain-words description of one bead move.
pub fn describe_step(st: &Step) -> String {
    let finger = match st.finger() {
        Finger::Thumb => "thumb",
        Finger::Index => "index finger",
    };
    let action = match st.kind {
        StepKind::EarthUp(n) => {
            format!("push {} earth bead{} up", n, if n == 1 { "" } else { "s" })
        }
        StepKind::EarthDown(n) => format!(
            "push {} earth bead{} down",
            n,
            if n == 1 { "" } else { "s" }
        ),
        StepKind::HeavenDown => "push the heaven bead down".to_string(),
        StepKind::HeavenUp => "push the heaven bead up".to_string(),
    };
    let why = match st.rule {
        Rule::Direct => String::new(),
        Rule::FiveComplement { friend } => {
            format!(" (small friend: {} and {} make 5)", friend, 5 - friend)
        }
        Rule::TenComplement { friend } => {
            format!(" (big friend: {} and {} make 10)", friend, 10 - friend)
        }
        Rule::Carry => " (the carry)".to_string(),
        Rule::Borrow => " (the borrow)".to_string(),
    };
    let mut s = format!("{}: {}{}", finger, action, why);
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

pub fn list(app: &mut App, ui: &mut Ui) {
    let pal = app.palette;
    if crate::ui::header(ui, &pal, None, "Lessons", |_| {}) {
        app.go_home();
        return;
    }
    ui.label(RichText::new("Each lesson explains one idea, shows it on the beads, then lets you try it with hints. Lessons unlock as the ones before them settle in.").color(pal.ink_soft));
    ui.add_space(8.0);
    let now = crate::storage::now_unix();
    egui::ScrollArea::vertical().show(ui, |ui| {
        crate::ui::card(ui, &pal, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let n = SkillId::ALL.len();
            for (i, skill) in SkillId::ALL.into_iter().enumerate() {
                let unlocked = app.save.scheduler.is_unlocked(skill);
                let mastered = app.save.scheduler.is_mastered(skill);
                let p = app.save.scheduler.p_eff(skill, now);
                let st = app.save.scheduler.state(skill);
                let status = if !unlocked {
                    "locked".to_string()
                } else if mastered {
                    "mastered".to_string()
                } else if st.n == 0 {
                    "new".to_string()
                } else {
                    format!(
                        "{}% · step {}/{}",
                        (p * 100.0).round() as i32,
                        st.level + 1,
                        skill.ladder().len()
                    )
                };
                if lesson_row(ui, &pal, skill, &status, unlocked, mastered).clicked() {
                    app.screen = crate::Screen::Lesson(LessonState::new(skill));
                }
                if i + 1 < n {
                    let w = ui.available_width();
                    let (r, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
                    ui.painter().hline(
                        r.x_range(),
                        r.center().y,
                        egui::Stroke::new(1.0, crate::ui::hairline(&pal)),
                    );
                }
            }
        });
        ui.add_space(12.0);
    });
}

/// One row of the lesson list: code, name, status; the whole row is the
/// button.
fn lesson_row(
    ui: &mut Ui,
    pal: &crate::theme::Palette,
    skill: SkillId,
    status: &str,
    unlocked: bool,
    mastered: bool,
) -> egui::Response {
    let w = ui.available_width();
    let sense = if unlocked {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 40.0), sense);
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, unlocked, skill.name())
    });
    let p = ui.painter();
    if unlocked && resp.hovered() {
        p.rect_filled(
            rect.expand2(egui::vec2(6.0, 0.0)),
            6.0,
            crate::theme::mix(pal.paper2, pal.accent, 0.08),
        );
    }
    let ink = if unlocked {
        pal.ink
    } else {
        crate::theme::with_alpha(pal.ink_soft, 0.7)
    };
    let narrow = w < 420.0;
    let code_w = 40.0;
    p.text(
        egui::pos2(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        skill.code(),
        egui::FontId::monospace(14.0),
        if unlocked { pal.accent } else { ink },
    );
    p.text(
        egui::pos2(rect.left() + code_w, rect.center().y),
        egui::Align2::LEFT_CENTER,
        if narrow {
            skill.short_name()
        } else {
            skill.name()
        },
        egui::FontId::proportional(16.0),
        ink,
    );
    p.text(
        egui::pos2(rect.right(), rect.center().y),
        egui::Align2::RIGHT_CENTER,
        status,
        egui::FontId::proportional(13.0),
        if mastered { pal.ok } else { pal.ink_soft },
    );
    resp
}
