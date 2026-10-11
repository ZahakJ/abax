//! The interactive soroban widget: drawing, bead animation, tap and drag
//! input, highlights for "move this bead" and the anzan fade.

use abax_core::{Rod, Soroban, Step, StepKind};
use egui::{pos2, vec2, Color32, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::{with_alpha, Palette};

/// How the learner may move beads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Tap,
    Drag,
    Both,
}

/// A bead the user moved, for sound and for drill progress tracking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeadEvent {
    pub rod: usize,
    pub heaven: bool,
    pub before: Rod,
    pub after: Rod,
}

#[derive(Clone, Copy, Debug)]
struct Tween {
    from: f32,
    to: f32,
    start: f64,
    duration: f32,
}

impl Tween {
    fn still(v: f32) -> Self {
        Self {
            from: v,
            to: v,
            start: 0.0,
            duration: 0.0,
        }
    }

    fn at(&self, now: f64) -> f32 {
        if self.duration <= 0.0 {
            return self.to;
        }
        let t = ((now - self.start) as f32 / self.duration).clamp(0.0, 1.0);
        // ease-out cubic
        let e = 1.0 - (1.0 - t).powi(3);
        self.from + (self.to - self.from) * e
    }

    fn done(&self, now: f64) -> bool {
        self.duration <= 0.0 || now >= self.start + self.duration as f64
    }

    fn retarget(&mut self, now: f64, to: f32, duration: f32, delay: f32) {
        let cur = self.at(now);
        if (cur - to).abs() < 1e-4 {
            *self = Tween::still(to);
            return;
        }
        self.from = cur;
        self.to = to;
        self.start = now + delay as f64;
        self.duration = duration;
    }
}

#[derive(Clone, Copy, Debug)]
struct RodAnim {
    /// 0 = heaven bead up (away), 1 = down (at the beam).
    heaven: Tween,
    /// Slot index of each earth bead, 0 = touching the beam.
    earth: [Tween; 4],
}

impl RodAnim {
    fn new(r: Rod) -> Self {
        let h = if r.heaven { 1.0 } else { 0.0 };
        let mut earth = [Tween::still(0.0); 4];
        for (k, e) in earth.iter_mut().enumerate() {
            *e = Tween::still(Self::earth_slot(r, k));
        }
        Self {
            heaven: Tween::still(h),
            earth,
        }
    }

    fn earth_slot(r: Rod, k: usize) -> f32 {
        if (k as u8) < r.earth {
            k as f32
        } else {
            k as f32 + 1.0
        }
    }
}

/// Geometry of one layout pass.
#[derive(Clone, Copy, Debug)]
pub struct Geometry {
    pub rect: Rect,
    pub pitch: f32,
    pub bead_w: f32,
    pub bead_h: f32,
    pub frame_w: f32,
    pub heaven_top: f32,
    pub beam_top: f32,
    pub beam_h: f32,
    pub earth_top: f32,
    pub earth_bottom: f32,
    pub digits_y: f32,
    pub rods_left: f32,
}

impl Geometry {
    pub fn rod_x(&self, i: usize) -> f32 {
        self.rods_left + (i as f32 + 0.5) * self.pitch
    }

    fn heaven_y(&self, t: f32) -> f32 {
        // Heaven bead centre: slot 0 (up) to slot 1 (down, at the beam).
        self.heaven_top + self.bead_h * 0.5 + t * self.bead_h
    }

    fn earth_y(&self, slot: f32) -> f32 {
        self.earth_top + self.bead_h * 0.5 + slot * self.bead_h
    }

    fn rod_at(&self, p: Pos2) -> Option<usize> {
        if p.x < self.rods_left || p.x > self.rect.right() - self.frame_w {
            return None;
        }
        Some(((p.x - self.rods_left) / self.pitch).floor() as usize)
    }
}

/// Animation durations (seconds).
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub user_move: f32,
    pub demo_move: f32,
    pub stagger: f32,
    pub reduce: bool,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            user_move: 0.09,
            demo_move: 0.40,
            stagger: 0.015,
            reduce: false,
        }
    }
}

pub struct SorobanView {
    pub model: Soroban,
    anim: Vec<RodAnim>,
    /// Steps to highlight as "move this".
    pub hint: Vec<Step>,
    /// 0 = fully visible, 1 = beads invisible (anzan).
    pub fade: f32,
    pub active_rod: Option<usize>,
    pub show_digits: bool,
    pub input: InputMode,
    pub interactive: bool,
    pub motion: Motion,
    drag: Option<DragState>,
    /// Rods whose last animation finished, for repaint scheduling.
    animating: bool,
    pub last_geometry: Option<Geometry>,
}

#[derive(Clone, Copy, Debug)]
struct DragState {
    rod: usize,
    heaven: bool,
    bead: usize,
    start: Pos2,
    before: Rod,
    applied: bool,
}

impl SorobanView {
    pub fn new(rod_count: usize) -> Self {
        let model = Soroban::new(rod_count);
        let anim = model.rods().iter().map(|r| RodAnim::new(*r)).collect();
        Self {
            model,
            anim,
            hint: Vec::new(),
            fade: 0.0,
            active_rod: None,
            show_digits: true,
            input: InputMode::Both,
            interactive: true,
            motion: Motion::default(),
            drag: None,
            animating: false,
            last_geometry: None,
        }
    }

    pub fn rod_count(&self) -> usize {
        self.model.rod_count()
    }

    pub fn set_rod_count(&mut self, n: usize) {
        if n != self.model.rod_count() {
            self.model.resize(n);
            self.anim = self.model.rods().iter().map(|r| RodAnim::new(*r)).collect();
            self.active_rod = None;
            self.hint.clear();
        }
    }

    /// Replace the whole model instantly (no animation).
    pub fn snap_to(&mut self, s: &Soroban) {
        self.set_rod_count(s.rod_count());
        self.model = s.clone();
        self.anim = self.model.rods().iter().map(|r| RodAnim::new(*r)).collect();
    }

    /// Animate every rod toward `target`.
    pub fn animate_to(&mut self, target: &Soroban, now: f64, duration: f32) {
        self.set_rod_count(target.rod_count());
        let rods: Vec<Rod> = target.rods().to_vec();
        for (i, r) in rods.into_iter().enumerate() {
            self.set_rod(i, r, now, duration, 0.0);
        }
    }

    /// Clear with the tilt sweep: right to left stagger.
    pub fn clear(&mut self, now: f64) {
        let n = self.rod_count();
        let dur = if self.motion.reduce { 0.0 } else { 0.22 };
        for i in 0..n {
            let delay = if self.motion.reduce {
                0.0
            } else {
                0.012 * (n - 1 - i) as f32
            };
            self.set_rod(i, Rod::default(), now, dur, delay);
        }
        self.hint.clear();
    }

    pub fn set_value(&mut self, v: u128, now: f64) {
        let mut t = self.model.clone();
        t.set_value(v);
        let d = self.motion.user_move;
        self.animate_to(&t, now, d);
    }

    /// Set one rod to `r`, animating beads that change.
    pub fn set_rod(&mut self, i: usize, r: Rod, now: f64, duration: f32, delay: f32) {
        let duration = if self.motion.reduce { 0.0 } else { duration };
        let stagger = if self.motion.reduce { 0.0 } else { self.motion.stagger };
        let a = &mut self.anim[i];
        let h = if r.heaven { 1.0 } else { 0.0 };
        a.heaven.retarget(now, h, duration, delay);
        let before = self.model.rod(i);
        // Beads that move as a group get a small stagger in travel order.
        let mut order: Vec<usize> = (0..4).collect();
        if r.earth < before.earth {
            order.reverse();
        }
        let mut k_off = 0;
        for k in order {
            let slot = RodAnim::earth_slot(r, k);
            let moving = (a.earth[k].to - slot).abs() > 1e-4;
            a.earth[k].retarget(now, slot, duration, delay + stagger * k_off as f32);
            if moving {
                k_off += 1;
            }
        }
        *self.model.rod_mut(i) = r;
        self.animating = true;
    }

    /// Apply one bead step with the demo animation (lessons, replays).
    pub fn apply_step(&mut self, step: &Step, now: f64, duration: f32) {
        let mut r = self.model.rod(step.rod);
        match step.kind {
            StepKind::EarthUp(n) => r.earth = (r.earth + n).min(4),
            StepKind::EarthDown(n) => r.earth = r.earth.saturating_sub(n),
            StepKind::HeavenDown => r.heaven = true,
            StepKind::HeavenUp => r.heaven = false,
        }
        self.set_rod(step.rod, r, now, duration, 0.0);
    }

    pub fn is_animating(&self) -> bool {
        self.animating
    }

    /// Compute the layout for the given available size.
    pub fn layout(&self, avail: Rect) -> Geometry {
        let n = self.rod_count() as f32;
        let frame_ratio = 0.9; // frame width in pitches
        // Height in bead_h units: heaven (2 slots) + beam (0.9) + earth (5 slots)
        // + frame top/bottom (0.8 each) + digits row (1.1).
        let units_h = 2.0 + 0.9 + 5.0 + 1.6 + if self.show_digits { 1.1 } else { 0.0 };
        // bead_h = 0.52 * pitch.
        let pitch_from_w = avail.width() / (n + 2.0 * frame_ratio);
        let pitch_from_h = avail.height() / (units_h * 0.52);
        let pitch = pitch_from_w.min(pitch_from_h).max(8.0);
        let bead_h = pitch * 0.52;
        let bead_w = pitch * 0.86;
        let frame_w = pitch * frame_ratio;
        let total_w = pitch * n + 2.0 * frame_w;
        let total_h = bead_h * units_h;
        let left = avail.center().x - total_w / 2.0;
        let top = avail.top() + ((avail.height() - total_h) / 2.0).max(0.0);
        let rect = Rect::from_min_size(pos2(left, top), vec2(total_w, total_h));
        let heaven_top = rect.top() + bead_h * 0.8;
        let beam_top = heaven_top + 2.0 * bead_h;
        let beam_h = bead_h * 0.9;
        let earth_top = beam_top + beam_h;
        let earth_bottom = earth_top + 5.0 * bead_h;
        Geometry {
            rect,
            pitch,
            bead_w,
            bead_h,
            frame_w,
            heaven_top,
            beam_top,
            beam_h,
            earth_top,
            earth_bottom,
            digits_y: earth_bottom + bead_h * 0.8 + bead_h * 0.45,
            rods_left: rect.left() + frame_w,
        }
    }

    /// Draw and handle input. Returns the bead moves the user made this frame.
    pub fn show(&mut self, ui: &mut Ui, palette: &Palette, desired: Vec2) -> Vec<BeadEvent> {
        let (outer, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
        let g = self.layout(outer);
        self.last_geometry = Some(g);
        let now = ui.input(|i| i.time);
        let mut events = Vec::new();

        if self.interactive && ui.is_enabled() {
            events = self.handle_input(&response, &g, now);
        }

        self.paint(ui, palette, &g, now);

        // Keep animating while any tween is in flight.
        let mut any = false;
        for a in &self.anim {
            if !a.heaven.done(now) || a.earth.iter().any(|t| !t.done(now)) {
                any = true;
                break;
            }
        }
        self.animating = any;
        if any || !self.hint.is_empty() {
            ui.ctx().request_repaint();
        }
        events
    }

    fn handle_input(&mut self, response: &egui::Response, g: &Geometry, now: f64) -> Vec<BeadEvent> {
        let mut events = Vec::new();
        let allow_tap = matches!(self.input, InputMode::Tap | InputMode::Both);
        let allow_drag = matches!(self.input, InputMode::Drag | InputMode::Both);

        if response.drag_started() {
            if let Some(p) = response.interact_pointer_pos() {
                if let Some(rod) = g.rod_at(p) {
                    if rod < self.rod_count() {
                        let (heaven, bead) = self.bead_at(g, rod, p.y);
                        self.drag = Some(DragState {
                            rod,
                            heaven,
                            bead,
                            start: p,
                            before: self.model.rod(rod),
                            applied: false,
                        });
                        self.active_rod = Some(rod);
                    }
                }
            }
        }

        if response.dragged() && allow_drag {
            if let (Some(d), Some(p)) = (self.drag, response.interact_pointer_pos()) {
                let dy = p.y - d.start.y;
                let threshold = g.bead_h * 0.45;
                let before = d.before;
                let mut after = before;
                if d.heaven {
                    if dy > threshold {
                        after.heaven = true;
                    } else if dy < -threshold {
                        after.heaven = false;
                    } else {
                        after.heaven = before.heaven;
                    }
                } else {
                    // Follow the finger: number of beads up implied by how far
                    // the grabbed bead travelled.
                    let slots = (dy / g.bead_h).round() as i32;
                    let k = d.bead as i32;
                    let was_up = (d.bead as u8) < before.earth;
                    let new_earth = if was_up {
                        // Dragging down pushes bead k and those below it down.
                        if slots >= 1 {
                            k
                        } else {
                            before.earth as i32
                        }
                    } else if slots <= -1 {
                        // Dragging up brings bead k and those above it up.
                        k + 1
                    } else {
                        before.earth as i32
                    };
                    after.earth = new_earth.clamp(0, 4) as u8;
                }
                if after != self.model.rod(d.rod) {
                    let prev = self.model.rod(d.rod);
                    self.set_rod(d.rod, after, now, self.motion.user_move, 0.0);
                    events.push(BeadEvent {
                        rod: d.rod,
                        heaven: d.heaven,
                        before: prev,
                        after,
                    });
                    if let Some(ds) = &mut self.drag {
                        ds.applied = true;
                    }
                }
            }
        }

        if response.drag_stopped() {
            self.drag = None;
        }

        if response.clicked() && allow_tap {
            if let Some(p) = response.interact_pointer_pos() {
                if let Some(rod) = g.rod_at(p) {
                    if rod < self.rod_count() {
                        let (heaven, bead) = self.bead_at(g, rod, p.y);
                        let before = self.model.rod(rod);
                        let mut after = before;
                        if heaven {
                            after.heaven = !before.heaven;
                        } else if (bead as u8) < before.earth {
                            // Bead at the beam: push it and those below down.
                            after.earth = bead as u8;
                        } else {
                            // Bead away: bring it and those above to the beam.
                            after.earth = (bead as u8 + 1).min(4);
                        }
                        self.active_rod = Some(rod);
                        if after != before {
                            self.set_rod(rod, after, now, self.motion.user_move, 0.0);
                            events.push(BeadEvent {
                                rod,
                                heaven,
                                before,
                                after,
                            });
                        }
                    }
                }
            }
        }
        events
    }

    /// Which bead region a vertical position falls in: (is_heaven, earth bead
    /// index). Earth index is resolved from the bead's current drawn slot so
    /// taps land on the bead the user sees.
    fn bead_at(&self, g: &Geometry, rod: usize, y: f32) -> (bool, usize) {
        if y < g.beam_top + g.beam_h * 0.5 {
            return (true, 0);
        }
        let now_slot = ((y - g.earth_top) / g.bead_h).floor().clamp(0.0, 4.0) as usize;
        // Map slot to bead: beads occupy slots 0..earth-1 and earth+1..4.
        let earth = self.model.rod(rod).earth as usize;
        let bead = if now_slot < earth {
            now_slot
        } else if now_slot == earth {
            // The gap: choose the nearer neighbour by half-slot.
            let frac = (y - g.earth_top) / g.bead_h - now_slot as f32;
            if frac < 0.5 && earth > 0 {
                earth - 1
            } else {
                earth.min(3)
            }
        } else {
            (now_slot - 1).min(3)
        };
        (false, bead)
    }

    fn paint(&self, ui: &mut Ui, pal: &Palette, g: &Geometry, now: f64) {
        let painter = ui.painter();
        let r = g.rect;
        let cr = (g.pitch * 0.25).clamp(3.0, 10.0);

        // Frame: outer lacquer, inner bevel.
        painter.rect(r, cr, pal.frame, Stroke::NONE, StrokeKind::Inside);
        let inner = Rect::from_min_max(
            pos2(g.rods_left, g.heaven_top - g.bead_h * 0.35),
            pos2(r.right() - g.frame_w, g.earth_bottom + g.bead_h * 0.35),
        );
        painter.rect(
            inner,
            cr * 0.5,
            crate::theme::mix(pal.frame, Color32::BLACK, 0.25),
            Stroke::new(1.0, pal.frame_light),
            StrokeKind::Inside,
        );

        // Rods.
        let rod_w = (g.pitch * 0.09).clamp(1.5, 5.0);
        for i in 0..self.rod_count() {
            let x = g.rod_x(i);
            painter.rect_filled(
                Rect::from_center_size(
                    pos2(x, (inner.top() + inner.bottom()) / 2.0),
                    vec2(rod_w, inner.height()),
                ),
                1.0,
                pal.rod,
            );
        }

        // Beam with unit dots.
        let beam = Rect::from_min_max(
            pos2(inner.left(), g.beam_top),
            pos2(inner.right(), g.beam_top + g.beam_h),
        );
        painter.rect(beam, 2.0, pal.beam, Stroke::new(1.0, crate::theme::mix(pal.beam, Color32::BLACK, 0.3)), StrokeKind::Inside);
        for i in 0..self.rod_count() {
            if self.model.has_unit_dot(i) {
                painter.circle_filled(
                    pos2(g.rod_x(i), beam.center().y),
                    (g.pitch * 0.07).clamp(1.5, 4.0),
                    crate::theme::mix(pal.beam, Color32::BLACK, 0.55),
                );
            }
        }

        // Active rod marker.
        if let Some(a) = self.active_rod {
            if a < self.rod_count() {
                let x = g.rod_x(a);
                painter.rect_filled(
                    Rect::from_center_size(
                        pos2(x, inner.bottom() - 2.0),
                        vec2(g.pitch * 0.7, 3.0),
                    ),
                    1.5,
                    pal.accent,
                );
            }
        }

        // Beads.
        let alpha = 1.0 - self.fade.clamp(0.0, 1.0);
        let pulse = 0.55 + 0.45 * ((now * 3.0).sin() as f32 * 0.5 + 0.5);
        for i in 0..self.rod_count() {
            let x = g.rod_x(i);
            let a = &self.anim[i];
            let hy = g.heaven_y(a.heaven.at(now));
            let h_hint = self.hint_for(i, true);
            self.paint_bead(painter, pal, pos2(x, hy), g, alpha, h_hint, pulse);
            for k in 0..4 {
                let ey = g.earth_y(a.earth[k].at(now));
                let e_hint = self.hint_for_earth(i, k);
                self.paint_bead(painter, pal, pos2(x, ey), g, alpha, e_hint, pulse);
            }
        }

        // Digits.
        if self.show_digits {
            let font = egui::FontId::new((g.pitch * 0.55).clamp(9.0, 22.0), egui::FontFamily::Monospace);
            let col = with_alpha(pal.bead, 0.85 * alpha.max(0.0));
            for i in 0..self.rod_count() {
                let d = self.model.digit(i);
                painter.text(
                    pos2(g.rod_x(i), g.digits_y),
                    egui::Align2::CENTER_CENTER,
                    d.to_string(),
                    font.clone(),
                    if alpha > 0.05 { col } else { with_alpha(pal.bead, 0.0) },
                );
            }
        }
    }

    /// Direction hint for the heaven bead of rod `i`: Some(+1) move down,
    /// Some(-1) move up.
    fn hint_for(&self, i: usize, heaven: bool) -> Option<i8> {
        if !heaven {
            return None;
        }
        self.hint.iter().find_map(|s| {
            if s.rod != i {
                return None;
            }
            match s.kind {
                StepKind::HeavenDown => Some(1),
                StepKind::HeavenUp => Some(-1),
                _ => None,
            }
        })
    }

    fn hint_for_earth(&self, i: usize, k: usize) -> Option<i8> {
        let earth = self.model.rod(i).earth as usize;
        self.hint.iter().find_map(|s| {
            if s.rod != i {
                return None;
            }
            match s.kind {
                // Beads earth..earth+n move up.
                StepKind::EarthUp(n) => {
                    if k >= earth && k < earth + n as usize {
                        Some(-1)
                    } else {
                        None
                    }
                }
                // Beads earth-n..earth move down.
                StepKind::EarthDown(n) => {
                    if k < earth && k + (n as usize) >= earth {
                        Some(1)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_bead(
        &self,
        painter: &egui::Painter,
        pal: &Palette,
        c: Pos2,
        g: &Geometry,
        alpha: f32,
        hint: Option<i8>,
        pulse: f32,
    ) {
        let w = g.bead_w / 2.0;
        let h = g.bead_h / 2.0 * 0.92;
        let lens = |cx: f32, cy: f32, w: f32, h: f32| -> Vec<Pos2> {
            // Biconvex lens: flat-ish rhombus with rounded flanks.
            vec![
                pos2(cx - w, cy),
                pos2(cx - w * 0.72, cy - h * 0.62),
                pos2(cx - w * 0.36, cy - h * 0.94),
                pos2(cx, cy - h),
                pos2(cx + w * 0.36, cy - h * 0.94),
                pos2(cx + w * 0.72, cy - h * 0.62),
                pos2(cx + w, cy),
                pos2(cx + w * 0.72, cy + h * 0.62),
                pos2(cx + w * 0.36, cy + h * 0.94),
                pos2(cx, cy + h),
                pos2(cx - w * 0.36, cy + h * 0.94),
                pos2(cx - w * 0.72, cy + h * 0.62),
            ]
        };
        if alpha > 0.01 {
            // Shadow.
            painter.add(Shape::convex_polygon(
                lens(c.x, c.y + h * 0.18, w, h),
                with_alpha(Color32::BLACK, 0.28 * alpha),
                Stroke::NONE,
            ));
            // Body.
            painter.add(Shape::convex_polygon(
                lens(c.x, c.y, w, h),
                with_alpha(pal.bead, alpha),
                Stroke::new(1.0, with_alpha(pal.bead_shadow, alpha)),
            ));
            // Lower shade.
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - w, c.y),
                    pos2(c.x + w, c.y),
                    pos2(c.x + w * 0.72, c.y + h * 0.62),
                    pos2(c.x + w * 0.36, c.y + h * 0.94),
                    pos2(c.x, c.y + h),
                    pos2(c.x - w * 0.36, c.y + h * 0.94),
                    pos2(c.x - w * 0.72, c.y + h * 0.62),
                ],
                with_alpha(pal.bead_shadow, 0.45 * alpha),
                Stroke::NONE,
            ));
            // Specular.
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - w * 0.55, c.y - h * 0.35),
                    pos2(c.x - w * 0.25, c.y - h * 0.75),
                    pos2(c.x + w * 0.15, c.y - h * 0.75),
                    pos2(c.x - w * 0.15, c.y - h * 0.3),
                ],
                with_alpha(pal.bead_hilite, 0.55 * alpha),
                Stroke::NONE,
            ));
            // Hole hint.
            painter.circle_filled(c, (g.pitch * 0.05).clamp(1.0, 3.0), with_alpha(pal.frame, 0.35 * alpha));
        }
        if let Some(dir) = hint {
            let ring = lens(c.x, c.y, w + 3.0, h + 3.0);
            painter.add(Shape::closed_line(
                ring,
                Stroke::new(2.0, with_alpha(pal.accent, 0.35 + 0.65 * pulse)),
            ));
            // Arrow in the direction of travel.
            let s = (g.bead_h * 0.45).clamp(5.0, 14.0);
            let ay = if dir > 0 { c.y + h + s * 0.9 } else { c.y - h - s * 0.9 };
            let tip = pos2(c.x, if dir > 0 { ay + s * 0.6 } else { ay - s * 0.6 });
            let base = if dir > 0 { ay - s * 0.4 } else { ay + s * 0.4 };
            painter.add(Shape::convex_polygon(
                vec![tip, pos2(c.x - s * 0.6, base), pos2(c.x + s * 0.6, base)],
                with_alpha(pal.accent, 0.5 + 0.5 * pulse),
                Stroke::NONE,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_eases_to_target() {
        let mut t = Tween::still(0.0);
        t.retarget(10.0, 1.0, 0.1, 0.0);
        assert!((t.at(10.0) - 0.0).abs() < 1e-6);
        assert!(t.at(10.05) > 0.5, "ease-out passes halfway early");
        assert!((t.at(10.1) - 1.0).abs() < 1e-6);
        assert!(t.done(10.1));
        assert!(!t.done(10.05));
    }

    #[test]
    fn earth_slots_leave_a_gap_below_the_beam_beads() {
        let r = Rod::new(false, 2);
        assert_eq!(RodAnim::earth_slot(r, 0), 0.0);
        assert_eq!(RodAnim::earth_slot(r, 1), 1.0);
        assert_eq!(RodAnim::earth_slot(r, 2), 3.0);
        assert_eq!(RodAnim::earth_slot(r, 3), 4.0);
    }

    #[test]
    fn layout_fits_width_and_height() {
        let v = SorobanView::new(27);
        let g = v.layout(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 300.0)));
        assert!(g.rect.width() <= 1000.0 + 0.5);
        assert!(g.rect.height() <= 300.0 + 0.5);
        let v = SorobanView::new(13);
        let g = v.layout(Rect::from_min_size(pos2(0.0, 0.0), vec2(360.0, 2000.0)));
        assert!(g.rect.width() <= 360.5);
        assert!((0..13).all(|i| g.rod_x(i) > g.rect.left() && g.rod_x(i) < g.rect.right()));
    }

    #[test]
    fn tap_rules_via_bead_at() {
        let mut v = SorobanView::new(3);
        let g = v.layout(Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 400.0)));
        // Empty rod: slot 1 (first earth bead, which sits in slot 1) -> bead 0.
        let y = g.earth_y(1.0);
        assert_eq!(v.bead_at(&g, 0, y), (false, 0));
        // Heaven region.
        assert_eq!(v.bead_at(&g, 0, g.heaven_y(0.0)), (true, 0));
        // Rod showing 3: beads 0..2 in slots 0..2; slot 4 holds bead 3.
        v.model.set_digit(1, 3);
        assert_eq!(v.bead_at(&g, 1, g.earth_y(4.0)), (false, 3));
        assert_eq!(v.bead_at(&g, 1, g.earth_y(1.0)), (false, 1));
    }

    #[test]
    fn hint_marks_the_right_earth_beads() {
        let mut v = SorobanView::new(2);
        v.model.set_digit(1, 2);
        v.hint = vec![Step::new(1, StepKind::EarthUp(2), abax_core::Rule::Direct)];
        assert_eq!(v.hint_for_earth(1, 1), None);
        assert_eq!(v.hint_for_earth(1, 2), Some(-1));
        assert_eq!(v.hint_for_earth(1, 3), Some(-1));
        v.hint = vec![Step::new(1, StepKind::EarthDown(1), abax_core::Rule::Direct)];
        assert_eq!(v.hint_for_earth(1, 1), Some(1));
        assert_eq!(v.hint_for_earth(1, 0), None);
        v.hint = vec![Step::new(1, StepKind::HeavenDown, abax_core::Rule::Direct)];
        assert_eq!(v.hint_for(1, true), Some(1));
        assert_eq!(v.hint_for(0, true), None);
    }
}
