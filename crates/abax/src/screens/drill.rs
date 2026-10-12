//! The drill runner: one engine for set/read/chain/multiply/divide items,
//! guided lesson practice, anzan (fading beads) and timed sprints.

use abax_core::arith::Procedure;
use abax_core::curriculum::{Level, SkillId};
use abax_core::generator::{generate, Item, Op};
use abax_core::moves::{add_number_at, steps_to_show, sub_number_at};
use abax_core::scheduler::DrillSpec;
use abax_core::stats::{Attempt, DrillKind};
use abax_core::{Soroban, Step};
use egui::{RichText, Ui};

use crate::keypad::{keyboard_actions, keypad, DigitBuffer, KeyAction};
use crate::soroban_view::SorobanView;
use crate::App;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrillMode {
    Normal,
    /// Lesson practice: hints come fast, nothing is recorded as a drill.
    Guided,
    /// Beads fade out over this many seconds after the item starts.
    Anzan {
        fade_seconds: f32,
    },
    /// As many items as possible in this many seconds.
    Sprint {
        seconds: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Working,
    /// Short confirmation before the next item.
    Correct {
        until: f64,
    },
    /// Wrong: the correct moves replay on the soroban.
    Replay {
        steps: Vec<Step>,
        next: usize,
        next_at: f64,
        answer: u128,
    },
    Done,
}

struct Queued {
    skill: SkillId,
    level: Level,
    item: Item,
}

pub struct DrillState {
    pub spec: DrillSpec,
    pub mode: DrillMode,
    pub kind: DrillKind,
    queue: Vec<Queued>,
    idx: usize,
    phase: Phase,
    item_started: f64,
    last_move: f64,
    buffer: DigitBuffer,
    progress: usize,
    hint_given: bool,
    results: Vec<(SkillId, bool, f64)>,
    sprint_end: Option<f64>,
    walkthrough: Option<(Procedure, usize)>,
    stable_since: Option<f64>,
    pub summary_lines: Vec<String>,
    needs_setup: bool,
}

/// Rods an item needs, with one spare rod for carries.
fn rods_needed(item: &Item) -> usize {
    let digits = |n: u128| n.to_string().len();
    match item {
        Item::Set { target } | Item::Read { value: target } => digits(*target) + 1,
        Item::Chain { start, ops, .. } => {
            let mut v = *start;
            let mut max = v;
            for (op, n) in ops {
                v = match op {
                    Op::Add => v + n,
                    Op::Sub => v.saturating_sub(*n),
                };
                max = max.max(v).max(*n);
            }
            digits(max) + 1
        }
        Item::Multiply { a, b, .. } => abax_core::arith::rods_for_multiply(digits(*a), digits(*b)),
        Item::Divide {
            dividend, divisor, ..
        } => abax_core::arith::rods_for_divide(digits(*dividend), digits(*divisor)),
        Item::Flash { answer, .. } => digits(*answer) + 1,
    }
}

fn drill_kind_for(skill: SkillId, mode: DrillMode) -> DrillKind {
    match mode {
        DrillMode::Guided => DrillKind::Lesson,
        DrillMode::Anzan { .. } => DrillKind::Anzan,
        DrillMode::Sprint { .. } => DrillKind::Sprint,
        DrillMode::Normal => match skill {
            SkillId::Setting => DrillKind::Set,
            SkillId::Multiply => DrillKind::Multiply,
            SkillId::Divide => DrillKind::Divide,
            SkillId::MultiMixed | SkillId::Combined => DrillKind::Mixed,
            SkillId::FlashOne | SkillId::FlashMulti => DrillKind::Flash,
            _ => DrillKind::AddSub,
        },
    }
}

/// The soroban a wrong-answer replay starts from: the chain's start value,
/// or an empty soroban for every other item.
fn replay_start(item: &Item, rod_count: usize) -> Soroban {
    let mut s = Soroban::new(rod_count);
    if let Item::Chain { start, .. } = item {
        s.set_value(*start);
    }
    s
}

/// The moves a wrong-answer replay shows, applied to [`replay_start`].
/// A chain replays only its operations: the start value is already on the
/// beads, so it must not be added again.
fn replay_steps_for(item: &Item, rod_count: usize) -> Vec<Step> {
    match item {
        Item::Read { .. } | Item::Multiply { .. } | Item::Divide { .. } => Vec::new(),
        Item::Set { target } => steps_to_show(&Soroban::new(rod_count), *target),
        Item::Chain { ops, .. } => {
            let mut s = replay_start(item, rod_count);
            let unit = s.unit_rod();
            let mut out = Vec::new();
            for (op, n) in ops {
                let steps = match op {
                    Op::Add => add_number_at(&mut s, unit, *n),
                    Op::Sub => sub_number_at(&mut s, unit, *n),
                };
                out.extend(steps.unwrap_or_default());
            }
            out
        }
        Item::Flash { .. } => item.solution_steps(rod_count),
    }
}

impl DrillState {
    pub fn new(app: &mut App, spec: DrillSpec, mode: DrillMode) -> Self {
        let rod_count = app.save.settings.rod_count;
        let n = match mode {
            DrillMode::Sprint { .. } => 60,
            DrillMode::Guided => 3,
            _ => spec.items.max(1) as usize,
        };
        let mut queue = Vec::with_capacity(n);
        let interleave = spec.interleave.clone();
        for i in 0..n {
            // About 30 % interleaved items from other skills the learner has met.
            let use_other = !interleave.is_empty() && mode == DrillMode::Normal && i % 10 >= 7;
            let (skill, level) = if use_other {
                let s = interleave[app.rng.below(interleave.len() as u32) as usize];
                (s, app.save.scheduler.current_level(s))
            } else {
                (spec.skill, spec.level)
            };
            let item = {
                let rng = &mut app.rng;
                let mut dice = |n: u64| rng.below(n.max(1) as u32) as u64;
                generate(skill, level, rod_count, &mut dice)
            };
            queue.push(Queued { skill, level, item });
        }
        // Size the soroban to the problem: on a phone a 13-rod soroban has
        // beads too small to tap, and a drill never needs more than a few
        // rods beyond the numbers involved.
        let needed = queue
            .iter()
            .map(|q| rods_needed(&q.item))
            .max()
            .unwrap_or(5)
            .max(5);
        let wide = app.last_width > 700.0;
        let rods = if wide {
            needed.max(rod_count.min(9)).min(rod_count.max(needed))
        } else {
            needed
        };
        app.view.set_rod_count(rods);
        let kind = drill_kind_for(spec.skill, mode);
        Self {
            spec,
            mode,
            kind,
            queue,
            idx: 0,
            phase: Phase::Working,
            item_started: 0.0,
            last_move: 0.0,
            buffer: DigitBuffer::default(),
            progress: 0,
            hint_given: false,
            results: Vec::new(),
            sprint_end: None,
            walkthrough: None,
            stable_since: None,
            summary_lines: Vec::new(),
            needs_setup: true,
        }
    }

    fn current(&self) -> Option<&Queued> {
        self.queue.get(self.idx)
    }

    /// Does the learner type the answer (true) or work it on the beads?
    fn typed_answer(&self) -> bool {
        match self.current().map(|q| &q.item) {
            Some(Item::Read { .. })
            | Some(Item::Multiply { .. })
            | Some(Item::Divide { .. })
            | Some(Item::Flash { .. }) => true,
            Some(Item::Chain { .. }) => matches!(self.mode, DrillMode::Anzan { .. }),
            _ => false,
        }
    }

    fn setup_item(&mut self, view: &mut SorobanView, now: f64) {
        self.needs_setup = false;
        self.buffer.clear();
        self.progress = 0;
        self.hint_given = false;
        self.walkthrough = None;
        self.stable_since = None;
        view.hint.clear();
        view.fade = 0.0;
        view.active_rod = None;
        self.item_started = now;
        self.last_move = now;
        let Some(q) = self.queue.get(self.idx) else {
            return;
        };
        match &q.item {
            Item::Read { value } => {
                view.show_digits = false;
                view.set_value(*value, now);
                view.interactive = false;
            }
            Item::Set { .. } => {
                view.show_digits = false;
                view.clear(now);
                view.interactive = true;
            }
            Item::Chain { start, .. } => {
                view.show_digits = true;
                view.interactive = true;
                view.clear(now);
                view.set_value(*start, now);
            }
            Item::Multiply { .. } | Item::Divide { .. } | Item::Flash { .. } => {
                view.show_digits = true;
                view.interactive = true;
                view.clear(now);
            }
        }
        if matches!(self.mode, DrillMode::Anzan { .. }) {
            view.show_digits = false;
        }
        if let (DrillMode::Sprint { seconds }, None) = (self.mode, self.sprint_end) {
            self.sprint_end = Some(now + seconds as f64);
        }
    }

    /// Expected running values of a chain: v[0] = start, v[i] after i ops.
    fn chain_values(item: &Item) -> Vec<u128> {
        let Item::Chain { start, ops, .. } = item else {
            return Vec::new();
        };
        let mut v = vec![*start];
        let mut cur = *start as i128;
        for (op, n) in ops {
            cur = match op {
                Op::Add => cur + *n as i128,
                Op::Sub => cur - *n as i128,
            };
            v.push(cur.max(0) as u128);
        }
        v
    }

    fn hint_steps(&self, model: &Soroban) -> Vec<Step> {
        let Some(q) = self.current() else {
            return Vec::new();
        };
        match &q.item {
            Item::Set { target } => steps_to_show(model, *target),
            Item::Chain { ops, .. } => {
                let values = Self::chain_values(&q.item);
                let p = self.progress.min(ops.len());
                let expected = values[p];
                if model.value() != expected {
                    return steps_to_show(model, expected);
                }
                if p >= ops.len() {
                    return Vec::new();
                }
                let (op, n) = ops[p];
                let mut c = model.clone();
                let unit = c.unit_rod();
                match op {
                    Op::Add => add_number_at(&mut c, unit, n).unwrap_or_default(),
                    Op::Sub => sub_number_at(&mut c, unit, n).unwrap_or_default(),
                }
            }
            _ => Vec::new(),
        }
    }

    fn submit(&mut self, app: &mut App, given: Option<u128>, now: f64) {
        let Some(q) = self.queue.get(self.idx) else {
            return;
        };
        let answer = q.item.answer();
        let correct = given == Some(answer);
        let seconds = (now - self.item_started).max(0.05);
        let skill = q.skill;
        let level_idx = skill
            .ladder()
            .iter()
            .position(|l| *l == q.level)
            .unwrap_or(0) as u8;
        self.results.push((skill, correct, seconds));
        let ts = crate::storage::now_unix();
        if self.mode != DrillMode::Guided {
            let outcome = app.save.scheduler.record(skill, correct, seconds, ts);
            app.save.history.push(Attempt {
                ts,
                skill,
                level: level_idx,
                kind: self.kind,
                correct,
                seconds: seconds as f32,
            });
            app.mark_dirty();
            if outcome.newly_mastered {
                self.summary_lines
                    .push(format!("{} is now mastered.", skill.name()));
            } else if outcome.stepped_up {
                self.summary_lines
                    .push(format!("{}: moved up a step.", skill.short_name()));
            } else if outcome.stepped_down {
                self.summary_lines
                    .push(format!("{}: eased back a step.", skill.short_name()));
            }
        }
        if correct {
            self.phase = Phase::Correct { until: now + 0.7 };
        } else {
            // Replay the right moves from the item's start state.
            let rod_count = app.view.rod_count();
            let start = replay_start(&q.item, rod_count);
            let steps = replay_steps_for(&q.item, rod_count);
            app.view.hint.clear();
            app.view.fade = 0.0;
            app.view.show_digits = true;
            app.view.interactive = false;
            app.view.animate_to(&start, now, 0.25);
            self.phase = Phase::Replay {
                steps,
                next: 0,
                next_at: now + 0.9,
                answer,
            };
        }
    }

    fn advance(&mut self, now: f64) {
        self.idx += 1;
        self.needs_setup = true;
        let out_of_items = self.idx >= self.queue.len();
        let sprint_over = self.sprint_end.is_some_and(|e| now >= e);
        if out_of_items || sprint_over {
            self.phase = Phase::Done;
        } else {
            self.phase = Phase::Working;
        }
    }

    pub fn show(&mut self, app: &mut App, ui: &mut Ui) {
        let now = ui.input(|i| i.time);
        let pal = app.palette;
        if self.needs_setup && self.phase == Phase::Working {
            let mut view = std::mem::replace(&mut app.view, SorobanView::new(1));
            self.setup_item(&mut view, now);
            app.view = view;
        }
        // A sprint ends when the clock runs out, even mid-item; the
        // unfinished item is not recorded.
        if self.phase == Phase::Working && self.sprint_end.is_some_and(|e| now >= e) {
            self.phase = Phase::Done;
        }
        if self.phase == Phase::Done {
            self.show_summary(app, ui);
            return;
        }
        let Some(q) = self.queue.get(self.idx) else {
            self.phase = Phase::Done;
            return;
        };
        let item = q.item.clone();
        let skill = q.skill;
        let typed = self.typed_answer();

        // Header: skill and progress.
        let title = if ui.available_width() < 480.0 {
            skill.short_name()
        } else {
            skill.name()
        };
        let progress = match self.sprint_end {
            Some(end) => {
                ui.ctx().request_repaint();
                format!("{:.0} s", (end - now).max(0.0))
            }
            None => format!("{} / {}", self.idx + 1, self.queue.len()),
        };
        let crumb = match self.mode {
            DrillMode::Anzan { .. } => Some("Anzan"),
            DrillMode::Sprint { .. } => Some("Sprint"),
            DrillMode::Guided => Some("Practice"),
            DrillMode::Normal => None,
        };
        if crate::ui::header(ui, &pal, crumb, title, |ui| {
            ui.label(RichText::new(progress).monospace().color(pal.ink_soft));
        }) {
            app.go_home();
            return;
        }

        // Prompt.
        let prompt_text = match &item {
            Item::Chain { start, ops, .. } if !typed => {
                // Highlight the operand the learner is on.
                let mut job = egui::text::LayoutJob::default();
                let big = egui::FontId::proportional(30.0);
                let p = self.progress;
                let col = |i: usize| {
                    if i == p {
                        pal.accent
                    } else if i < p {
                        pal.ink_soft
                    } else {
                        pal.ink
                    }
                };
                job.append(
                    &start.to_string(),
                    0.0,
                    egui::TextFormat {
                        font_id: big.clone(),
                        color: col(0),
                        ..Default::default()
                    },
                );
                for (i, (op, n)) in ops.iter().enumerate() {
                    let sym = match op {
                        Op::Add => " + ",
                        Op::Sub => " − ",
                    };
                    job.append(
                        sym,
                        0.0,
                        egui::TextFormat {
                            font_id: big.clone(),
                            color: pal.ink_soft,
                            ..Default::default()
                        },
                    );
                    job.append(
                        &n.to_string(),
                        0.0,
                        egui::TextFormat {
                            font_id: big.clone(),
                            color: col(i + 1),
                            ..Default::default()
                        },
                    );
                }
                ui.vertical_centered(|ui| ui.label(job));
                None
            }
            Item::Set { target } => Some(format!("Set {target}")),
            Item::Read { .. } => Some("What number is shown?".to_string()),
            _ => Some(item.prompt()),
        };
        if let Some(t) = prompt_text {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t).size(30.0).strong());
            });
        }

        // Phase-specific status line.
        match &self.phase {
            Phase::Correct { .. } => {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Correct").color(pal.ok).size(18.0))
                });
            }
            Phase::Replay { answer, .. } => {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(format!("The answer is {answer}. Watch the moves."))
                            .color(pal.accent)
                            .size(18.0),
                    );
                });
            }
            _ => {
                let hint_text = self.status_text(&item, typed);
                ui.vertical_centered(|ui| ui.label(RichText::new(hint_text).color(pal.ink_soft)));
            }
        }

        // Anzan fade.
        if let DrillMode::Anzan { fade_seconds } = self.mode {
            if self.phase == Phase::Working {
                let t = (now - self.item_started) as f32;
                app.view.fade = ((t - 0.8) / fade_seconds.max(0.1)).clamp(0.0, 1.0);
                if app.view.fade < 1.0 {
                    ui.ctx().request_repaint();
                }
            }
        }

        // Soroban.
        let avail = ui.available_size();
        let keypad_h = if app.save.settings.show_keypad || typed {
            4.0 * 48.0 + 40.0
        } else {
            60.0
        };
        let soroban_h = app
            .view
            .natural_height(avail.x)
            .min((avail.y - keypad_h).max(140.0));
        let events = app.view.show(ui, &pal, egui::vec2(avail.x, soroban_h));
        for ev in &events {
            app.audio.click(ev.heaven);
            self.last_move = now;
            app.view.hint.clear();
            self.hint_given = false;
        }

        // Progress tracking for chains / sets.
        if self.phase == Phase::Working && !typed {
            let value = app.view.model.value();
            match &item {
                Item::Chain { .. } => {
                    let values = Self::chain_values(&item);
                    while self.progress + 1 < values.len() && value == values[self.progress + 1] {
                        self.progress += 1;
                    }
                    if value == item.answer() && self.progress + 1 == values.len() {
                        let since = *self.stable_since.get_or_insert(now);
                        if now - since > 0.6 {
                            self.submit(app, Some(value), now);
                        } else {
                            ui.ctx().request_repaint();
                        }
                    } else {
                        self.stable_since = None;
                    }
                }
                Item::Set { target } => {
                    if value == *target {
                        let since = *self.stable_since.get_or_insert(now);
                        if now - since > 0.5 {
                            self.submit(app, Some(value), now);
                        } else {
                            ui.ctx().request_repaint();
                        }
                    } else {
                        self.stable_since = None;
                    }
                }
                _ => {}
            }
        }

        // Stall hint.
        if self.phase == Phase::Working && !typed {
            let delay = match self.mode {
                DrillMode::Guided => 1.5,
                _ => app.save.settings.hint_delay_seconds.max(1.0),
            } as f64;
            if now - self.last_move > delay && app.view.hint.is_empty() {
                let steps = self.hint_steps(&app.view.model);
                if !steps.is_empty() {
                    // Show the first move only (one rod's worth).
                    let first_rod = steps[0].rod;
                    app.view.hint = steps
                        .iter()
                        .take_while(|s| s.rod == first_rod)
                        .copied()
                        .collect();
                    self.hint_given = true;
                }
            }
            ui.ctx().request_repaint_after_secs(0.25);
        }

        // Replay progression.
        if let Phase::Replay {
            steps,
            next,
            next_at,
            ..
        } = &mut self.phase
        {
            if now >= *next_at {
                if *next < steps.len() {
                    let st = steps[*next];
                    app.view.hint = vec![st];
                    app.view.apply_step(&st, now, 0.4);
                    *next += 1;
                    *next_at = now + 0.75;
                } else if *next == steps.len() {
                    app.view.hint.clear();
                    *next += 1;
                    *next_at = now + 1.2;
                } else {
                    self.advance(now);
                }
            }
            ui.ctx().request_repaint_after_secs(0.05);
        }
        if let Phase::Correct { until } = self.phase {
            if now >= until {
                self.advance(now);
            }
            ui.ctx().request_repaint_after_secs(0.05);
        }

        // Walkthrough text for multiplication / division.
        if let Some((proc_, step)) = &self.walkthrough {
            if let Some(ps) = proc_.steps.get(*step) {
                ui.label(RichText::new(&ps.text).color(pal.ink));
            }
        }

        // Input: keyboard + keypad.
        let mut actions = keyboard_actions(ui);
        ui.add_space(6.0);
        let narrow_pad = (app.save.settings.show_keypad || typed) && ui.available_width() < 420.0;
        if narrow_pad {
            // Very narrow: readout above the keypad, actions under it.
            let left = app.save.settings.left_handed;
            let key = egui::vec2(
                ((ui.available_width() - 24.0) / 3.0).clamp(48.0, 96.0),
                46.0,
            );
            let pad_w = crate::keypad::keypad_width(key);
            ui.vertical_centered(|ui| {
                if typed {
                    let shown = if self.buffer.is_empty() {
                        "…".to_string()
                    } else {
                        self.buffer.text.clone()
                    };
                    ui.label(RichText::new(shown).size(28.0).monospace());
                } else {
                    ui.label(
                        RichText::new(format!("Shows {}", app.view.model.value()))
                            .size(18.0)
                            .monospace()
                            .color(pal.ink_soft),
                    );
                }
            });
            if let Some(a) = crate::keypad::keypad_centered(ui, &pal, left, key) {
                actions.push(a);
            }
            ui.add_space(4.0);
            let indent = ((ui.available_width() - pad_w) / 2.0).max(0.0);
            if matches!(item, Item::Multiply { .. } | Item::Divide { .. })
                && self.phase == Phase::Working
            {
                let label = if self.walkthrough.is_some() {
                    "Next step"
                } else {
                    "Walk me through it"
                };
                let clicked = ui
                    .horizontal(|ui| {
                        ui.add_space(indent);
                        ui.add(egui::Button::new(label).min_size(egui::vec2(pad_w, 32.0)))
                            .clicked()
                    })
                    .inner;
                if clicked {
                    self.walkthrough_step(app, now);
                }
            }
            ui.horizontal(|ui| {
                ui.add_space(indent);
                let gap = ui.spacing().item_spacing.x;
                let half = egui::vec2((pad_w - gap) / 2.0, 32.0);
                let clear = if self.phase == Phase::Working {
                    half
                } else {
                    egui::vec2(pad_w, 32.0)
                };
                if ui
                    .add(egui::Button::new("Clear beads").min_size(clear))
                    .clicked()
                {
                    app.view.clear(now);
                    self.last_move = now;
                }
                if self.phase == Phase::Working
                    && crate::ui::primary_button(ui, &pal, "Check", half).clicked()
                {
                    actions.push(KeyAction::Enter);
                }
            });
        } else {
            ui.horizontal(|ui| {
                let left = app.save.settings.left_handed;
                let show_pad = app.save.settings.show_keypad || typed;
                if show_pad {
                    let key = egui::vec2(
                        ((ui.available_width() - 160.0) / 3.0).clamp(48.0, 96.0),
                        46.0,
                    );
                    let total = crate::keypad::keypad_width(key) + 150.0;
                    ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
                    let pad = |ui: &mut Ui, actions: &mut Vec<KeyAction>| {
                        if let Some(a) = keypad(ui, &pal, left, key) {
                            actions.push(a);
                        }
                    };
                    if left {
                        pad(ui, &mut actions);
                    }
                    ui.vertical(|ui| {
                        if typed {
                            let shown = if self.buffer.is_empty() {
                                "…".to_string()
                            } else {
                                self.buffer.text.clone()
                            };
                            ui.label(RichText::new(shown).size(28.0).monospace());
                        } else {
                            ui.label(
                                RichText::new(format!("Shows {}", app.view.model.value()))
                                    .size(18.0)
                                    .monospace()
                                    .color(pal.ink_soft),
                            );
                        }
                        if matches!(item, Item::Multiply { .. } | Item::Divide { .. })
                            && self.phase == Phase::Working
                        {
                            let label = if self.walkthrough.is_some() {
                                "Next step"
                            } else {
                                "Walk me through it"
                            };
                            if ui.button(label).clicked() {
                                self.walkthrough_step(app, now);
                            }
                        }
                        if ui.button("Clear beads").clicked() {
                            app.view.clear(now);
                            self.last_move = now;
                        }
                        if self.phase == Phase::Working
                            && crate::ui::primary_button(ui, &pal, "Check", egui::vec2(0.0, 0.0))
                                .clicked()
                        {
                            actions.push(KeyAction::Enter);
                        }
                    });
                    if !left {
                        pad(ui, &mut actions);
                    }
                } else if self.phase == Phase::Working {
                    ui.label(
                        RichText::new(format!("Shows {}", app.view.model.value()))
                            .monospace()
                            .color(pal.ink_soft),
                    );
                    if crate::ui::primary_button(ui, &pal, "Check", egui::vec2(0.0, 0.0)).clicked()
                    {
                        actions.push(KeyAction::Enter);
                    }
                }
            });
        }

        if self.phase != Phase::Working {
            return;
        }
        for a in actions {
            match a {
                KeyAction::Enter => {
                    // A typed number wins; otherwise read the beads.
                    let given = if typed || !self.buffer.is_empty() {
                        self.buffer.value()
                    } else {
                        Some(app.view.model.value())
                    };
                    self.submit(app, given, now);
                    return;
                }
                KeyAction::Clear => {
                    if typed {
                        self.buffer.clear();
                    } else {
                        app.view.clear(now);
                        self.last_move = now;
                    }
                }
                other => {
                    if typed {
                        self.buffer.apply(other);
                    } else {
                        crate::screens::practice::rod_keyboard(&mut app.view, other, now);
                        self.last_move = now;
                        app.view.hint.clear();
                    }
                }
            }
        }
    }

    fn walkthrough_step(&mut self, app: &mut App, now: f64) {
        let rod_count = app.view.rod_count();
        if self.walkthrough.is_none() {
            if let Some(p) = self.current().and_then(|q| q.item.procedure(rod_count)) {
                app.view.clear(now);
                self.walkthrough = Some((p, usize::MAX));
            } else {
                return;
            }
        }
        let Some((proc_, step)) = &mut self.walkthrough else {
            return;
        };
        let next = if *step == usize::MAX { 0 } else { *step + 1 };
        if let Some(ps) = proc_.steps.get(next) {
            for st in &ps.beads {
                app.view.apply_step(st, now, 0.4);
            }
            app.view.hint = ps.beads.clone();
            app.view.active_rod = ps.focus.first().copied();
            *step = next;
        }
        self.last_move = now;
    }

    fn status_text(&self, item: &Item, typed: bool) -> String {
        match (item, typed, self.mode) {
            (Item::Set { .. }, _, _) => "Move the beads, then it is checked automatically.".into(),
            (Item::Read { .. }, _, _) => "Type the number and press Enter.".into(),
            (Item::Chain { .. }, false, DrillMode::Guided) => {
                "Follow the highlighted beads.".into()
            }
            (Item::Chain { .. }, false, _) => {
                "Work it on the beads; it is checked when you reach the answer.".into()
            }
            (Item::Chain { .. }, true, _) => "Picture the beads, then type the total.".into(),
            (Item::Multiply { .. }, _, _) | (Item::Divide { .. }, _, _) => {
                "Type the answer, or ask to be walked through it.".into()
            }
            (Item::Flash { .. }, _, _) => "Type the sum.".into(),
        }
    }

    fn show_summary(&mut self, app: &mut App, ui: &mut Ui) {
        let pal = app.palette;
        let total = self.results.len();
        let correct = self.results.iter().filter(|r| r.1).count();
        let mean_t = if total > 0 {
            self.results.iter().map(|r| r.2).sum::<f64>() / total as f64
        } else {
            0.0
        };
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.label(
                RichText::new(match self.mode {
                    DrillMode::Guided => "Practice done",
                    DrillMode::Sprint { .. } => "Sprint over",
                    _ => "Drill done",
                })
                .size(crate::ui::size::PROMPT)
                .strong()
                .color(pal.ink),
            );
            ui.add_space(8.0);
            ui.label(RichText::new(format!("{correct} of {total} correct")).size(24.0));
            if total > 0 {
                ui.label(
                    RichText::new(format!("{mean_t:.1} s per item on average")).color(pal.ink_soft),
                );
            }
            for l in &self.summary_lines {
                ui.label(RichText::new(l).color(pal.ok));
            }
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if let Some(next) = app.pending_drills.first() {
                    let label = format!("Next: {}", next.skill.name());
                    if crate::ui::primary_button(ui, &pal, &label, egui::vec2(0.0, 0.0)).clicked() {
                        app.start_next_pending();
                        return;
                    }
                } else {
                    if self.mode == DrillMode::Guided
                        && crate::ui::primary_button(
                            ui,
                            &pal,
                            "Start the drill",
                            egui::vec2(0.0, 0.0),
                        )
                        .clicked()
                    {
                        let mut spec = self.spec.clone();
                        spec.items = 10;
                        app.start_drill(spec, DrillMode::Normal);
                        return;
                    }
                    let again = if self.mode == DrillMode::Guided {
                        ui.button("Practice again")
                    } else {
                        crate::ui::primary_button(ui, &pal, "Again", egui::vec2(0.0, 0.0))
                    };
                    if again.clicked() {
                        let spec = self.spec.clone();
                        let mode = self.mode;
                        app.start_drill(spec, mode);
                        return;
                    }
                }
                if ui.button("Home").clicked() {
                    app.go_home();
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abax_core::moves::apply_steps;

    fn chain(start: u128, ops: Vec<(Op, u128)>, answer: u128) -> Item {
        Item::Chain { start, ops, answer }
    }

    #[test]
    fn chain_replay_starts_from_the_start_value() {
        let rods = 5;
        let item = chain(7, vec![(Op::Add, 6)], 13);
        let steps = replay_steps_for(&item, rods);
        assert!(!steps.is_empty());

        let mut s = replay_start(&item, rods);
        assert_eq!(s.value(), 7);
        apply_steps(&mut s, &steps);
        assert_eq!(s.value(), 13);

        // The replay holds only the "+ 6" moves, not the moves that set 7:
        // it is shorter than the from-empty solution and is exactly the
        // tail that follows the set-7 prefix.
        let full = item.solution_steps(rods);
        assert!(steps.len() < full.len());
        assert_eq!(&full[full.len() - steps.len()..], &steps[..]);
        assert_ne!(&full[..steps.len()], &steps[..]);
    }

    #[test]
    fn chain_replay_does_not_re_add_start() {
        // 7 + 1 needs no carry, so its moves can also be applied to an
        // empty soroban: there they give 1, not 8, proving the start value
        // is not in the replay.
        let rods = 5;
        let item = chain(7, vec![(Op::Add, 1)], 8);
        let steps = replay_steps_for(&item, rods);
        assert_eq!(steps[0].rod, rods - 1);

        let mut s = replay_start(&item, rods);
        apply_steps(&mut s, &steps);
        assert_eq!(s.value(), 8);

        let mut empty = Soroban::new(rods);
        apply_steps(&mut empty, &steps);
        assert_eq!(empty.value(), 1);
    }

    #[test]
    fn set_replay_starts_empty() {
        let rods = 5;
        let item = Item::Set { target: 42 };
        let mut s = replay_start(&item, rods);
        assert!(s.is_clear());
        apply_steps(&mut s, &replay_steps_for(&item, rods));
        assert_eq!(s.value(), 42);
    }
}
