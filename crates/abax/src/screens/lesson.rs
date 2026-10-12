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
    /// Weak prerequisites to mention on this visit; decided once, on the
    /// first frame, and only if the note was never shown for this lesson.
    builds_on: Option<Vec<SkillId>>,
    /// A second tap is needed to reset a skill that has real practice.
    confirm_reset: bool,
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
            builds_on: None,
            confirm_reset: false,
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
        if self.builds_on.is_none() {
            let weak = app.save.scheduler.weak_prerequisites(self.skill);
            let show = !weak.is_empty() && app.save.note_builds_on(self.skill);
            if show {
                app.mark_dirty();
            }
            self.builds_on = Some(if show { weak } else { Vec::new() });
        }
        let title = format!("Lesson {}", self.skill.code());
        let st = app.save.scheduler.state(self.skill);
        let narrow = ui.available_width() < 480.0;
        let mut know = false;
        let mut reset = false;
        let back = crate::ui::header(ui, &pal, Some("Lessons"), &title, |ui| {
            if st.marked_known || self.confirm_reset {
                let label = if self.confirm_reset {
                    "Tap again to reset"
                } else if narrow {
                    "Reset skill"
                } else {
                    "Reset this skill"
                };
                reset = ui
                    .button(RichText::new(label).color(pal.ink_soft))
                    .on_hover_text("Forget the estimate for this skill and start it fresh")
                    .clicked();
            } else {
                let label = if narrow {
                    "I know this"
                } else {
                    "I know this already"
                };
                know = ui
                    .button(label)
                    .on_hover_text(
                        "Skip ahead: the skill counts as known, and a few drills will check it",
                    )
                    .clicked();
            }
        });
        if back {
            app.view.interactive = true;
            app.screen = crate::Screen::Lessons;
            return;
        }
        if know {
            app.save.scheduler.mark_known(self.skill);
            app.mark_dirty();
            app.view.interactive = true;
            app.screen = match app.save.scheduler.next_skill_to_learn() {
                Some(next) if next != self.skill => crate::Screen::Lesson(LessonState::new(next)),
                _ => crate::Screen::Lessons,
            };
            return;
        }
        if reset {
            // Nothing earned is lost when the skill was only marked: reset at
            // once. Real practice takes a second tap.
            if st.n == 0 || self.confirm_reset {
                app.save.scheduler.reset_skill(self.skill);
                app.mark_dirty();
                self.confirm_reset = false;
            } else {
                self.confirm_reset = true;
            }
        }
        ui.label(
            RichText::new(l.title)
                .size(crate::ui::size::CARD_TITLE + 4.0)
                .strong()
                .color(pal.ink),
        );
        if st.marked_known && !self.confirm_reset {
            ui.label(
                RichText::new("You marked this as known. Drills will check it now and then.")
                    .color(pal.ink_soft)
                    .small(),
            );
        }
        let notes = self.builds_on.clone().unwrap_or_default();
        if !notes.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(RichText::new("Builds on:").color(pal.ink_soft).small());
                for (i, p) in notes.iter().enumerate() {
                    let r = ui.add(
                        egui::Button::new(
                            RichText::new(builds_on_label(*p)).color(pal.accent).small(),
                        )
                        .frame(false),
                    );
                    if r.clicked() {
                        app.screen = crate::Screen::Lesson(LessonState::new(*p));
                    }
                    if i + 1 < notes.len() {
                        ui.label(RichText::new("·").color(pal.ink_soft).small());
                    }
                }
            });
            if !matches!(app.screen, crate::Screen::Placeholder) {
                return;
            }
        }
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

/// The short name of a lesson for the "builds on" note: code plus the part
/// of the title before any colon ("S3 Small friends").
pub fn builds_on_label(skill: SkillId) -> String {
    let title = lesson(skill).title;
    let short = title.split(':').next().unwrap_or(title).trim();
    format!("{} {}", skill.code(), short)
}

/// One row of the lesson list, as data. Every row opens its lesson: there
/// is no locked state.
#[derive(Clone, Debug, PartialEq)]
pub struct LessonRow {
    pub skill: SkillId,
    pub status: String,
    /// The suggested next lesson (one at most).
    pub suggested: bool,
    pub mastered: bool,
}

/// The lesson list for a scheduler state, in teaching order.
pub fn lesson_rows(sched: &abax_core::scheduler::Scheduler, now: u64) -> Vec<LessonRow> {
    let next = sched.next_skill_to_learn();
    SkillId::ALL
        .into_iter()
        .map(|skill| {
            let st = sched.state(skill);
            let mastered = sched.is_mastered(skill);
            let suggested = next == Some(skill);
            let status = if mastered {
                "mastered".to_string()
            } else if suggested {
                "next".to_string()
            } else if st.n == 0 && st.marked_known {
                "known".to_string()
            } else if st.n == 0 {
                "new".to_string()
            } else {
                format!(
                    "{}% · step {}/{}",
                    (sched.p_eff(skill, now) * 100.0).round() as i32,
                    st.level + 1,
                    skill.ladder().len()
                )
            };
            LessonRow {
                skill,
                status,
                suggested,
                mastered,
            }
        })
        .collect()
}

pub fn list(app: &mut App, ui: &mut Ui) {
    let pal = app.palette;
    if crate::ui::header(ui, &pal, None, "Lessons", |_| {}) {
        app.go_home();
        return;
    }
    ui.label(RichText::new("Each lesson explains one idea, shows it on the beads, then lets you try it with hints. Open any of them, in any order; “next” marks the suggested path. Already know one? Open it and tap “I know this already”.").color(pal.ink_soft));
    ui.add_space(8.0);
    let now = crate::storage::now_unix();
    let rows = lesson_rows(&app.save.scheduler, now);
    egui::ScrollArea::vertical().show(ui, |ui| {
        crate::ui::card(ui, &pal, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let n = rows.len();
            for (i, row) in rows.iter().enumerate() {
                if lesson_row(ui, &pal, row).clicked() {
                    app.screen = crate::Screen::Lesson(LessonState::new(row.skill));
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
/// button, always enabled.
pub fn lesson_row(ui: &mut Ui, pal: &crate::theme::Palette, row: &LessonRow) -> egui::Response {
    let skill = row.skill;
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 40.0), egui::Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, skill.name()));
    let p = ui.painter();
    if resp.hovered() {
        p.rect_filled(
            rect.expand2(egui::vec2(6.0, 0.0)),
            6.0,
            crate::theme::mix(pal.paper2, pal.accent, 0.08),
        );
    }
    let ink = pal.ink;
    let narrow = w < 420.0;
    let code_w = 40.0;
    p.text(
        egui::pos2(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        skill.code(),
        egui::FontId::monospace(14.0),
        pal.accent,
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
        &row.status,
        egui::FontId::proportional(13.0),
        if row.mastered {
            pal.ok
        } else if row.suggested {
            pal.accent
        } else {
            pal.ink_soft
        },
    );
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use abax_core::scheduler::Scheduler;

    #[test]
    fn no_lesson_is_ever_locked() {
        // A fresh profile, one with gaps, and one taken out of order.
        let fresh = Scheduler::default();
        let mut gaps = Scheduler::default();
        gaps.mark_known(SkillId::Setting);
        gaps.record(SkillId::Divide, true, 10.0, 1_000);
        let mut jumped = Scheduler::default();
        jumped.mark_known(SkillId::FlashMulti);
        for sched in [fresh, gaps, jumped] {
            let rows = lesson_rows(&sched, 2_000);
            assert_eq!(rows.len(), SkillId::ALL.len());
            for r in &rows {
                assert!(!r.status.contains("lock"), "{:?}: {}", r.skill, r.status);
            }
            assert!(rows.iter().filter(|r| r.suggested).count() <= 1);
        }
    }

    #[test]
    fn statuses_show_next_known_and_progress() {
        let mut s = Scheduler::default();
        let rows = lesson_rows(&s, 0);
        assert_eq!(rows[0].status, "next");
        assert!(rows[0].suggested);
        assert!(rows[1..].iter().all(|r| r.status == "new"));
        s.mark_known(SkillId::Setting);
        s.record(SkillId::Divide, true, 10.0, 1_000);
        let rows = lesson_rows(&s, 1_000);
        assert_eq!(rows[0].status, "known");
        assert_eq!(rows[1].status, "next");
        let div = rows.iter().find(|r| r.skill == SkillId::Divide).unwrap();
        assert!(div.status.contains("step 1/"), "{}", div.status);
    }

    #[test]
    fn every_row_is_an_enabled_button() {
        let ctx = egui::Context::default();
        let rows = lesson_rows(&Scheduler::default(), 0);
        let mut seen = 0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            for row in &rows {
                let r = lesson_row(ui, &crate::theme::LIGHT, row);
                assert!(r.sense.senses_click(), "{:?}", row.skill);
                assert!(r.enabled(), "{:?}", row.skill);
                seen += 1;
            }
        });
        out.textures_delta.clear();
        assert_eq!(seen, SkillId::ALL.len());
    }

    #[test]
    fn builds_on_labels_are_short() {
        assert_eq!(builds_on_label(SkillId::AddFive), "S3 Small friends");
        assert_eq!(builds_on_label(SkillId::MultiAdd), "S8 Longer numbers");
    }
}
