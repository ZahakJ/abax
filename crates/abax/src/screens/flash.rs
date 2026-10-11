//! Flash anzan: numbers flash one after another; the learner types the sum.

use abax_core::curriculum::{Level, SkillId, Timing};
use abax_core::generator::{generate, Item};
use abax_core::stats::{Attempt, DrillKind};
use egui::{RichText, Ui};

use crate::keypad::{keyboard_actions, keypad_centered, DigitBuffer, KeyAction};
use crate::App;

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Ready,
    Showing {
        index: usize,
        until: f64,
        blank_until: f64,
    },
    Answer {
        started: f64,
    },
    Feedback {
        correct: bool,
        until: f64,
    },
    Done,
}

pub struct FlashState {
    pub skill: SkillId,
    pub digits: u8,
    pub count: u8,
    pub interval_ms: u16,
    numbers: Vec<u128>,
    answer: u128,
    phase: Phase,
    buffer: DigitBuffer,
    results: Vec<(bool, f64)>,
    rounds: usize,
    round: usize,
    stepped_msg: Option<String>,
}

impl FlashState {
    pub fn new(app: &mut App, skill: SkillId) -> Self {
        let s = &app.save.settings;
        let level = app.save.scheduler.current_level(skill);
        let (digits, count, interval_ms) = if skill == SkillId::FlashOne {
            (1, level.operands, level.flash_ms)
        } else if skill == SkillId::FlashMulti {
            (level.digits.max(2), level.operands, level.flash_ms)
        } else {
            (
                s.flash_digits.max(1),
                s.flash_count.max(2),
                s.flash_ms.max(200),
            )
        };
        Self {
            skill,
            digits,
            count,
            interval_ms,
            numbers: Vec::new(),
            answer: 0,
            phase: Phase::Ready,
            buffer: DigitBuffer::default(),
            results: Vec::new(),
            rounds: 5,
            round: 0,
            stepped_msg: None,
        }
    }

    fn level(&self) -> Level {
        Level {
            digits: self.digits,
            operands: self.count,
            timing: Timing::Untimed,
            flash_ms: self.interval_ms,
        }
    }

    /// Start the first round now (screenshots of the flashing phase).
    pub fn begin(&mut self, app: &mut App, now: f64) {
        self.round = 0;
        self.results.clear();
        self.start_round(app, now);
    }

    fn start_round(&mut self, app: &mut App, now: f64) {
        let level = self.level();
        let item = {
            let rng = &mut app.rng;
            let mut dice = |n: u64| rng.below(n.max(1) as u32) as u64;
            generate(SkillId::FlashMulti, level, 13, &mut dice)
        };
        if let Item::Flash {
            numbers, answer, ..
        } = item
        {
            self.numbers = numbers;
            self.answer = answer;
        }
        self.buffer.clear();
        let iv = self.interval_ms as f64 / 1000.0;
        self.phase = Phase::Showing {
            index: 0,
            until: now + 0.8 + iv,
            blank_until: now + 0.8,
        };
    }

    pub fn show(&mut self, app: &mut App, ui: &mut Ui) {
        let now = ui.input(|i| i.time);
        let pal = app.palette;
        let summary = format!(
            "{} × {} digit{} · {:.1} s",
            self.count,
            self.digits,
            if self.digits == 1 { "" } else { "s" },
            self.interval_ms as f32 / 1000.0
        );
        if crate::ui::header(ui, &pal, None, "Flash anzan", |ui| {
            ui.label(RichText::new(summary).color(pal.ink_soft));
        }) {
            app.go_home();
            return;
        }

        match self.phase.clone() {
            Phase::Ready => {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Ready?")
                            .size(crate::ui::size::PROMPT)
                            .strong()
                            .color(pal.ink),
                    );
                    ui.label(
                        RichText::new("Add the numbers as they flash, then type the total.")
                            .color(pal.ink_soft),
                    );
                    ui.add_space(12.0);
                    // The three settings in a centred card.
                    let w = ui.available_width().min(420.0);
                    let pad = ((ui.available_width() - w) / 2.0).max(0.0);
                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        ui.vertical(|ui| {
                            ui.set_width(w);
                            crate::ui::card(ui, &pal, |ui| {
                                ui.spacing_mut().slider_width = (w - 230.0).clamp(80.0, 200.0);
                                egui::Grid::new("flash-settings")
                                    .num_columns(2)
                                    .spacing([12.0, 10.0])
                                    .show(ui, |ui| {
                                        ui.label("Digits");
                                        ui.add(egui::Slider::new(&mut self.digits, 1..=4));
                                        ui.end_row();
                                        ui.label("Numbers");
                                        ui.add(egui::Slider::new(&mut self.count, 2..=20));
                                        ui.end_row();
                                        ui.label("Seconds each");
                                        let mut secs = self.interval_ms as f32 / 1000.0;
                                        if ui
                                            .add(
                                                egui::Slider::new(&mut secs, 0.2..=3.0)
                                                    .step_by(0.1),
                                            )
                                            .changed()
                                        {
                                            self.interval_ms = (secs * 1000.0).round() as u16;
                                        }
                                        ui.end_row();
                                    });
                            });
                        });
                    });
                    ui.add_space(10.0);
                    if crate::ui::primary_button(ui, &pal, "Start", egui::vec2(140.0, 40.0))
                        .clicked()
                    {
                        if self.skill == SkillId::Anzan {
                            let s = &mut app.save.settings;
                            s.flash_digits = self.digits;
                            s.flash_count = self.count;
                            s.flash_ms = self.interval_ms;
                            app.mark_dirty();
                        }
                        self.round = 0;
                        self.results.clear();
                        self.stepped_msg = None;
                        self.start_round(app, now);
                    }
                });
            }
            Phase::Showing {
                index,
                until,
                blank_until,
            } => {
                ui.ctx().request_repaint();
                // The flashed number: large tabular numerals in ink, no
                // box; a small counter underneath.
                let avail = ui.available_size();
                let (rect, _) = ui
                    .allocate_exact_size(egui::vec2(avail.x, avail.y * 0.7), egui::Sense::hover());
                let p = ui.painter();
                let size = (rect.height() * 0.5).clamp(64.0, 220.0);
                let n = self.numbers.get(index).copied().unwrap_or(0);
                // Fit the widest number of this round in the width.
                let digits = n.to_string().len().max(self.digits as usize) as f32;
                let size = size.min(rect.width() / (digits * 0.62).max(1.0));
                if now >= blank_until {
                    p.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        n.to_string(),
                        egui::FontId::monospace(size),
                        pal.ink,
                    );
                }
                p.text(
                    egui::pos2(rect.center().x, rect.bottom() - 14.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{} / {}", index + 1, self.numbers.len()),
                    egui::FontId::monospace(14.0),
                    pal.ink_soft,
                );
                if now >= until {
                    let iv = self.interval_ms as f64 / 1000.0;
                    let gap = (iv * 0.12).clamp(0.04, 0.2);
                    if index + 1 < self.numbers.len() {
                        self.phase = Phase::Showing {
                            index: index + 1,
                            until: now + gap + iv,
                            blank_until: now + gap,
                        };
                    } else {
                        self.phase = Phase::Answer { started: now };
                    }
                }
            }
            Phase::Answer { started } => {
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("Total?").size(22.0));
                    let shown = if self.buffer.is_empty() {
                        "…".to_string()
                    } else {
                        self.buffer.text.clone()
                    };
                    ui.label(RichText::new(shown).size(40.0).monospace());
                });
                let mut actions = keyboard_actions(ui);
                let key = egui::vec2(
                    ((ui.available_width() - 40.0) / 3.0).clamp(48.0, 96.0),
                    46.0,
                );
                if let Some(a) = keypad_centered(ui, &pal, app.save.settings.left_handed, key) {
                    actions.push(a);
                }
                for a in actions {
                    match a {
                        KeyAction::Enter => {
                            if let Some(v) = self.buffer.value() {
                                let correct = v == self.answer;
                                let secs = now - started;
                                self.results.push((correct, secs));
                                let ts = crate::storage::now_unix();
                                let skill = if self.skill == SkillId::Anzan {
                                    if self.digits == 1 {
                                        SkillId::FlashOne
                                    } else {
                                        SkillId::FlashMulti
                                    }
                                } else {
                                    self.skill
                                };
                                let outcome = app.save.scheduler.record(
                                    skill,
                                    correct,
                                    secs + self.count as f64 * self.interval_ms as f64 / 1000.0,
                                    ts,
                                );
                                let level_idx = skill
                                    .ladder()
                                    .iter()
                                    .position(|l| *l == self.level())
                                    .unwrap_or(0)
                                    as u8;
                                app.save.history.push(Attempt {
                                    ts,
                                    skill,
                                    level: level_idx,
                                    kind: DrillKind::Flash,
                                    correct,
                                    seconds: secs as f32,
                                });
                                app.mark_dirty();
                                if outcome.stepped_up {
                                    self.stepped_msg = Some("Moved up the speed ladder.".into());
                                } else if outcome.stepped_down {
                                    self.stepped_msg =
                                        Some("Eased back a step; build up again.".into());
                                }
                                self.phase = Phase::Feedback {
                                    correct,
                                    until: now + 1.6,
                                };
                            }
                        }
                        other => self.buffer.apply(other),
                    }
                }
            }
            Phase::Feedback { correct, until } => {
                ui.ctx().request_repaint();
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    if correct {
                        ui.label(RichText::new("Correct").color(pal.ok).size(30.0));
                    } else {
                        ui.label(
                            RichText::new(format!("The total was {}", self.answer))
                                .color(pal.accent)
                                .size(26.0),
                        );
                    }
                    let parts: Vec<String> = self.numbers.iter().map(|n| n.to_string()).collect();
                    ui.label(
                        RichText::new(parts.join(" + "))
                            .color(pal.ink_soft)
                            .monospace(),
                    );
                });
                if now >= until {
                    self.round += 1;
                    if self.round >= self.rounds {
                        self.phase = Phase::Done;
                    } else {
                        self.start_round(app, now);
                    }
                }
            }
            Phase::Done => {
                let total = self.results.len();
                let correct = self.results.iter().filter(|r| r.0).count();
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Round done")
                            .size(crate::ui::size::PROMPT)
                            .strong()
                            .color(pal.ink),
                    );
                    ui.label(RichText::new(format!("{correct} of {total} correct")).size(24.0));
                    if let Some(m) = &self.stepped_msg {
                        ui.label(RichText::new(m).color(pal.ok));
                    }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if crate::ui::primary_button(ui, &pal, "Again", egui::vec2(0.0, 0.0))
                            .clicked()
                        {
                            // Refresh the ladder step in case it moved.
                            let level = app.save.scheduler.current_level(self.skill);
                            if self.skill != SkillId::Anzan {
                                self.count = level.operands;
                                self.interval_ms = level.flash_ms;
                                self.digits = if self.skill == SkillId::FlashOne {
                                    1
                                } else {
                                    level.digits.max(2)
                                };
                            }
                            self.phase = Phase::Ready;
                        }
                        if ui.button("Home").clicked() {
                            app.go_home();
                        }
                    });
                });
            }
        }
    }
}
