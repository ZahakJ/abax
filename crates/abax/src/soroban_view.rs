//! The interactive soroban widget: drawing, bead animation, tap and drag
//! input, highlights for "move this bead" and the anzan fade.

use abax_core::{Rod, Soroban, Step, StepKind};
use egui::{pos2, vec2, Color32, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};

use crate::render::{kind, linear, Instance, SorobanRenderer};
use crate::theme::{with_alpha, BeadKind, Material, Palette, RodKind};

/// The settle curve for every bead move: a critically damped spring
/// x(t) = 1 - (1 + k t) e^(-k t), rescaled so it lands exactly on 1 at the
/// end of the move. It starts from rest, accelerates, then decelerates
/// long and soft, the way a bead on a rod slows against friction; it is
/// monotonic, so a bead never passes its stop (no overshoot).
pub fn settle(t: f32) -> f32 {
    const K: f32 = 7.0;
    let t = t.clamp(0.0, 1.0);
    let x = |t: f32| 1.0 - (1.0 + K * t) * (-K * t).exp();
    (x(t) / x(1.0)).min(1.0)
}

/// How a bead looks at a given anzan fade (0 = solid, 1 = gone): the bead
/// turns to glass rather than simply fading. The body goes translucent
/// first, the specular and rim highlights stay until late, and everything
/// vanishes over the last quarter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    /// Opacity of the bead body (albedo and shading).
    pub body: f32,
    /// Strength of the specular/rim highlights.
    pub highlight: f32,
    /// Overall multiplier on everything, shadow included.
    pub overall: f32,
}

pub fn glass(fade: f32) -> Glass {
    let f = fade.clamp(0.0, 1.0);
    let smooth = |a: f32, b: f32, x: f32| {
        let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    Glass {
        body: 1.0 - 0.88 * smooth(0.0, 0.6, f),
        highlight: 1.0 - 0.35 * smooth(0.45, 0.75, f),
        overall: 1.0 - smooth(0.75, 1.0, f),
    }
}

/// Seconds a click flash lasts.
const FLASH_SECONDS: f64 = 0.15;

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
        self.from + (self.to - self.from) * settle(t)
    }

    fn done(&self, now: f64) -> bool {
        self.duration <= 0.0 || now + 1e-6 >= self.start + self.duration as f64
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
    /// When each bead's click flash started (0 = heaven, 1..=4 = earth).
    flash: [f64; 5],
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
            flash: [f64::NEG_INFINITY; 5],
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
    /// The finish to draw; the app sets it from the settings each frame.
    pub material: Material,
    /// GPU buffers, created the first time the view draws (None until then,
    /// and forever when the app runs without wgpu: then `paint_classic`).
    renderer: Option<SorobanRenderer>,
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
            material: crate::theme::material(Default::default(), false),
            renderer: None,
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
        let stagger = if self.motion.reduce {
            0.0
        } else {
            self.motion.stagger
        };
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

    /// Start the click flash on every bead of `rod` whose move starts now
    /// (call right after a user move; the click sound plays at the same
    /// moment).
    pub fn flash_rod(&mut self, rod: usize, now: f64) {
        let Some(a) = self.anim.get_mut(rod) else {
            return;
        };
        let starting = |t: &Tween| (t.to - t.from).abs() > 1e-4 && !t.done(now);
        if starting(&a.heaven) {
            a.flash[0] = a.heaven.start;
        }
        for k in 0..4 {
            if starting(&a.earth[k]) {
                a.flash[k + 1] = a.earth[k].start;
            }
        }
        self.animating = true;
    }

    fn flash_at(&self, rod: usize, bead: usize, now: f64) -> f32 {
        let dt = now - self.anim[rod].flash[bead];
        if (0.0..FLASH_SECONDS).contains(&dt) {
            let x = 1.0 - (dt / FLASH_SECONDS) as f32;
            x * x
        } else {
            0.0
        }
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

    /// The height the soroban wants for a given width (so screens can put
    /// controls right under it instead of centring it in empty space).
    pub fn natural_height(&self, width: f32) -> f32 {
        let g = self.layout(Rect::from_min_size(pos2(0.0, 0.0), vec2(width, 100_000.0)));
        g.rect.height() + 8.0
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

        if self.renderer.is_none() {
            if let Some(gpu) = crate::render::gpu_context(ui.ctx()) {
                self.renderer = Some(SorobanRenderer::new(&gpu));
            }
        }
        self.paint(ui, palette, &g, now);

        // Keep animating while any tween or click flash is in flight.
        let mut any = false;
        for a in &self.anim {
            if !a.heaven.done(now)
                || a.earth.iter().any(|t| !t.done(now))
                || a.flash.iter().any(|f| now - f < FLASH_SECONDS)
            {
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

    fn handle_input(
        &mut self,
        response: &egui::Response,
        g: &Geometry,
        now: f64,
    ) -> Vec<BeadEvent> {
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
                    self.flash_rod(d.rod, now);
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
                            self.flash_rod(rod, now);
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
        match &self.renderer {
            Some(r) => {
                let ppp = ui.ctx().pixels_per_point();
                r.paint(
                    ui.painter(),
                    g.rect.min,
                    g.rect.expand(4.0),
                    now as f32,
                    |out| self.build_instances(g, pal.accent, now, ppp, out),
                );
            }
            None => self.paint_classic(ui, pal, g, now),
        }
        self.paint_overlay(ui, pal, g, now);
    }

    /// The well (recessed area behind the rods) for a layout.
    fn well_rect(g: &Geometry) -> Rect {
        Rect::from_min_max(
            pos2(g.rods_left, g.heaven_top - g.bead_h * 0.35),
            pos2(g.rect.right() - g.frame_w, g.earth_bottom + g.bead_h * 0.35),
        )
    }

    /// Hint glow pulse in 0..1 (1.2 s period; steady with reduce motion).
    fn pulse(&self, now: f64) -> f32 {
        if self.motion.reduce {
            1.0
        } else {
            (now * std::f64::consts::TAU / 1.2).sin() as f32 * 0.5 + 0.5
        }
    }

    /// Build this frame's instances, in draw order: board, well, active-rod
    /// glow, rods, beam, unit dots, active underline, hint glows, every
    /// bead shadow, then every bead (so no shadow paints over a neighbour).
    /// Rects are relative to `g.rect.min`. Allocation-free when `out` has
    /// capacity.
    pub fn build_instances(
        &self,
        g: &Geometry,
        accent: Color32,
        now: f64,
        ppp: f32,
        out: &mut Vec<Instance>,
    ) {
        let m = &self.material;
        let o = g.rect.min.to_vec2();
        let rel = |r: Rect| r.translate(-o);
        let n = self.rod_count();
        let cr = (g.pitch * 0.25).clamp(3.0, 10.0);
        let well = Self::well_rect(g);
        let px = 1.0 / ppp.max(0.5);
        let glass = glass(self.fade);
        let grain_scale = m.grain_scale;

        // Board.
        out.push(
            Instance::new(kind::BOARD, 0, rel(g.rect))
                .params([cr, (g.pitch * 0.09).clamp(3.0, 5.0), m.sheen, m.grain])
                .colors(
                    linear(m.frame, 1.0),
                    linear(m.frame_light, 1.0),
                    linear(m.frame_grain, grain_scale),
                ),
        );
        // Well, with the beam position (local y, relative to its centre)
        // for the shadow the beam casts.
        let beam = Rect::from_min_max(
            pos2(well.left(), g.beam_top),
            pos2(well.right(), g.beam_top + g.beam_h),
        );
        out.push(
            Instance::new(kind::WELL, 0, rel(well))
                .params([
                    cr * 0.5,
                    beam.top() - well.center().y,
                    beam.bottom() - well.center().y,
                    m.grain,
                ])
                .colors(
                    linear(m.well, 1.0),
                    linear(m.frame_light, 1.0),
                    linear(crate::theme::mix(m.well, Color32::BLACK, 0.35), grain_scale),
                ),
        );
        // Active rod: a soft glow strip inside the well.
        let active = self.active_rod.filter(|a| *a < n);
        if let Some(a) = active {
            let x = g.rod_x(a);
            let strip = Rect::from_center_size(
                pos2(x, well.center().y),
                vec2(g.pitch * 1.1, well.height() - 2.0),
            );
            out.push(Instance::new(kind::ACTIVE_GLOW, 0, rel(strip)).colors(
                linear(accent, if m.well_is_light() { 0.22 } else { 0.30 }),
                [0.0; 4],
                [0.0; 4],
            ));
        }
        // Rods.
        let rod_w = (g.pitch * 0.085).clamp(1.6, 5.0);
        let rod_variant = match m.rod_kind {
            RodKind::Bamboo => 0,
            RodKind::Brass => 1,
            RodKind::Steel => 2,
        };
        for i in 0..n {
            let r = Rect::from_center_size(
                pos2(g.rod_x(i), well.center().y),
                vec2(rod_w, well.height()),
            );
            out.push(Instance::new(kind::ROD, rod_variant, rel(r)).colors(
                linear(m.rod, 1.0),
                linear(crate::theme::mix(m.rod, Color32::WHITE, 0.6), 1.0),
                [0.0; 4],
            ));
        }
        // Beam and unit dots.
        out.push(
            Instance::new(kind::BEAM, m.beam_metal as u32, rel(beam)).colors(
                linear(m.beam, 1.0),
                linear(m.beam_light, 1.0),
                linear(crate::theme::mix(m.beam, Color32::BLACK, 0.35), grain_scale),
            ),
        );
        let dot_r = (g.pitch * 0.085).clamp(2.0, 5.0);
        for i in 0..n {
            if self.model.has_unit_dot(i) {
                let r = Rect::from_center_size(
                    pos2(g.rod_x(i), beam.center().y),
                    Vec2::splat(dot_r * 2.0),
                );
                out.push(Instance::new(kind::DOT, 0, rel(r)).colors(
                    linear(m.dot, 1.0),
                    linear(crate::theme::mix(m.dot, m.beam_light, 0.45), 1.0),
                    [0.0; 4],
                ));
            }
        }
        // Active rod: a thin accent line under it, on the lower frame.
        if let Some(a) = active {
            let r = Rect::from_center_size(
                pos2(g.rod_x(a), well.bottom() + (g.bead_h * 0.22).max(4.0)),
                vec2(g.pitch * 0.55, (g.bead_h * 0.08).clamp(2.0, 3.0)),
            );
            out.push(Instance::new(kind::UNDERLINE, 0, rel(r)).colors(
                linear(accent, 0.95),
                [0.0; 4],
                [0.0; 4],
            ));
        }

        // Bead centres and touch gaps, computed once per bead.
        let bw = g.bead_w;
        let bh = g.bead_h * 0.98;
        let half_h = bh * 0.5;
        let pulse = self.pulse(now);
        let mut beads: [(f32, f32, f32, f32); 5] = [(0.0, 0.0, 0.0, 0.0); 5];
        // Hint glows (behind everything bead-related).
        for i in 0..n {
            self.bead_column(g, i, now, &well, half_h, &mut beads);
            for (b, &(cy, ..)) in beads.iter().enumerate() {
                let hinted = if b == 0 {
                    self.hint_for(i, true).is_some()
                } else {
                    self.hint_for_earth(i, b - 1).is_some()
                };
                if hinted {
                    // A vermilion halo about 1.9x the bead, pulsing between
                    // 0.55 and 1.0 of its peak (steady at peak with reduce
                    // motion, where `pulse` is 1).
                    let r = Rect::from_center_size(pos2(g.rod_x(i), cy), vec2(bw * 1.9, bh * 2.4));
                    out.push(Instance::new(kind::HINT_GLOW, 0, rel(r)).colors(
                        linear(accent, 0.95 * (0.55 + 0.45 * pulse)),
                        [0.0; 4],
                        [0.0; 4],
                    ));
                }
            }
        }
        if glass.overall <= 0.0 {
            return;
        }
        // Shadows.
        let blur = (3.0 * px).max(g.bead_h * 0.05);
        let shadow_a = m.shadow_alpha * glass.overall * (0.35 + 0.65 * glass.body);
        for i in 0..n {
            self.bead_column(g, i, now, &well, half_h, &mut beads);
            for &(cy, ..) in &beads {
                let r = Rect::from_center_size(
                    pos2(g.rod_x(i) + bw * 0.06, cy + bh * 0.10),
                    vec2(bw, bh),
                );
                out.push(Instance::new(kind::SHADOW, 0, rel(r)).params([blur, shadow_a, 0.0, 0.0]));
            }
        }
        // Beads.
        let variant = match m.bead_kind {
            BeadKind::Bone => 0,
            BeadKind::Ebony => 1,
            BeadKind::Slate => 2,
        };
        let albedo = linear(m.bead, glass.body);
        let tint = linear(m.bead_shadow, glass.highlight);
        let hilite = linear(m.bead_hilite, glass.overall);
        for i in 0..n {
            self.bead_column(g, i, now, &well, half_h, &mut beads);
            for (b, &(cy, above, below, _)) in beads.iter().enumerate() {
                let r = Rect::from_center_size(pos2(g.rod_x(i), cy), vec2(bw, bh));
                out.push(
                    Instance::new(kind::BEAD, variant, rel(r))
                        .params([above, below, self.flash_at(i, b, now), rod_w * 0.5])
                        .colors(albedo, tint, hilite),
                );
            }
        }
    }

    /// Centre y and the free gap above and below (points) of each bead on
    /// rod `i`: index 0 is the heaven bead, 1..=4 the earth beads.
    fn bead_column(
        &self,
        g: &Geometry,
        i: usize,
        now: f64,
        well: &Rect,
        half_h: f32,
        out: &mut [(f32, f32, f32, f32); 5],
    ) {
        let a = &self.anim[i];
        let beam_bottom = g.beam_top + g.beam_h;
        let hy = g.heaven_y(a.heaven.at(now));
        out[0] = (
            hy,
            hy - half_h - well.top(),
            g.beam_top - (hy + half_h),
            0.0,
        );
        let mut ys = [0.0f32; 4];
        for (k, y) in ys.iter_mut().enumerate() {
            *y = g.earth_y(a.earth[k].at(now));
        }
        for k in 0..4 {
            let top = ys[k] - half_h;
            let bottom = ys[k] + half_h;
            let above = if k == 0 {
                top - beam_bottom
            } else {
                top - (ys[k - 1] + half_h)
            };
            let below = if k == 3 {
                well.bottom() - bottom
            } else {
                (ys[k + 1] - half_h) - bottom
            };
            out[k + 1] = (ys[k], above.max(0.0), below.max(0.0), 0.0);
        }
        out[0].1 = out[0].1.max(0.0);
        out[0].2 = out[0].2.max(0.0);
    }

    /// Digits and hint arrows: egui text and shapes painted over the
    /// soroban (after the callback), in both render paths.
    fn paint_overlay(&self, ui: &mut Ui, pal: &Palette, g: &Geometry, now: f64) {
        let painter = ui.painter();
        let alpha = 1.0 - self.fade.clamp(0.0, 1.0);
        if self.show_digits && alpha > 0.05 {
            let font = egui::FontId::new(
                (g.pitch * 0.55).clamp(9.0, 22.0),
                egui::FontFamily::Monospace,
            );
            let col = with_alpha(self.material.digit, 0.9 * alpha);
            for i in 0..self.rod_count() {
                let d = self.model.digit(i);
                painter.text(
                    pos2(g.rod_x(i), g.digits_y),
                    egui::Align2::CENTER_CENTER,
                    d.to_string(),
                    font.clone(),
                    col,
                );
            }
        }
        if self.hint.is_empty() {
            return;
        }
        let pulse = self.pulse(now);
        let half_h = g.bead_h * 0.49;
        let well = Self::well_rect(g);
        let mut beads = [(0.0, 0.0, 0.0, 0.0); 5];
        for i in 0..self.rod_count() {
            self.bead_column(g, i, now, &well, half_h, &mut beads);
            for (b, &(cy, ..)) in beads.iter().enumerate() {
                let dir = if b == 0 {
                    self.hint_for(i, true)
                } else {
                    self.hint_for_earth(i, b - 1)
                };
                if let Some(dir) = dir {
                    hint_arrow(painter, pal, g, pos2(g.rod_x(i), cy), half_h, dir, pulse);
                }
            }
        }
    }

    /// The previous painter path, kept for when wgpu is unavailable: flat
    /// polygons, so the board is never empty.
    fn paint_classic(&self, ui: &mut Ui, pal: &Palette, g: &Geometry, now: f64) {
        let m = &self.material;
        let painter = ui.painter();
        let r = g.rect;
        let cr = (g.pitch * 0.25).clamp(3.0, 10.0);

        // Frame: outer lacquer, inner bevel.
        painter.rect(r, cr, m.frame, Stroke::NONE, StrokeKind::Inside);
        let inner = Self::well_rect(g);
        painter.rect(
            inner,
            cr * 0.5,
            m.well,
            Stroke::new(1.0, m.frame_light),
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
                m.rod,
            );
        }

        // Beam with unit dots.
        let beam = Rect::from_min_max(
            pos2(inner.left(), g.beam_top),
            pos2(inner.right(), g.beam_top + g.beam_h),
        );
        painter.rect(
            beam,
            2.0,
            m.beam,
            Stroke::new(1.0, crate::theme::mix(m.beam, Color32::BLACK, 0.3)),
            StrokeKind::Inside,
        );
        for i in 0..self.rod_count() {
            if self.model.has_unit_dot(i) {
                painter.circle_filled(
                    pos2(g.rod_x(i), beam.center().y),
                    (g.pitch * 0.07).clamp(1.5, 4.0),
                    m.dot,
                );
            }
        }

        // Active rod marker.
        if let Some(a) = self.active_rod {
            if a < self.rod_count() {
                let x = g.rod_x(a);
                painter.rect_filled(
                    Rect::from_center_size(pos2(x, inner.bottom() - 2.0), vec2(g.pitch * 0.7, 3.0)),
                    1.5,
                    pal.accent,
                );
            }
        }

        // Beads.
        let alpha = 1.0 - self.fade.clamp(0.0, 1.0);
        let pulse = self.pulse(now);
        for i in 0..self.rod_count() {
            let x = g.rod_x(i);
            let a = &self.anim[i];
            let hy = g.heaven_y(a.heaven.at(now));
            let h_hint = self.hint_for(i, true);
            self.paint_bead_classic(painter, pal, pos2(x, hy), g, alpha, h_hint, pulse);
            for k in 0..4 {
                let ey = g.earth_y(a.earth[k].at(now));
                let e_hint = self.hint_for_earth(i, k);
                self.paint_bead_classic(painter, pal, pos2(x, ey), g, alpha, e_hint, pulse);
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
    fn paint_bead_classic(
        &self,
        painter: &egui::Painter,
        pal: &Palette,
        c: Pos2,
        g: &Geometry,
        alpha: f32,
        hint: Option<i8>,
        pulse: f32,
    ) {
        let m = &self.material;
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
        if hint.is_some() {
            painter.add(Shape::closed_line(
                lens(c.x, c.y, w + 3.0, h + 3.0),
                Stroke::new(2.0, with_alpha(pal.accent, 0.35 + 0.65 * pulse)),
            ));
        }
        if alpha > 0.01 {
            painter.add(Shape::convex_polygon(
                lens(c.x, c.y + h * 0.18, w, h),
                with_alpha(Color32::BLACK, 0.28 * alpha),
                Stroke::NONE,
            ));
            painter.add(Shape::convex_polygon(
                lens(c.x, c.y, w, h),
                with_alpha(m.bead, alpha),
                Stroke::new(1.0, with_alpha(m.bead_shadow, alpha)),
            ));
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
                with_alpha(m.bead_shadow, 0.45 * alpha),
                Stroke::NONE,
            ));
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - w * 0.55, c.y - h * 0.35),
                    pos2(c.x - w * 0.25, c.y - h * 0.75),
                    pos2(c.x + w * 0.15, c.y - h * 0.75),
                    pos2(c.x - w * 0.15, c.y - h * 0.3),
                ],
                with_alpha(m.bead_hilite, 0.55 * alpha),
                Stroke::NONE,
            ));
        }
    }
}

/// The "move this bead" arrow: a small rounded triangle in the gap on the
/// side the bead travels toward, in the accent colour, so a
/// highlight is never colour-only.
fn hint_arrow(
    painter: &egui::Painter,
    pal: &Palette,
    g: &Geometry,
    c: Pos2,
    half_h: f32,
    dir: i8,
    pulse: f32,
) {
    // Head size; about 1.5x the 0.1 arrow.
    let s = (g.bead_h * 0.54).clamp(7.0, 16.0);
    let sign = if dir > 0 { 1.0 } else { -1.0 };
    // Shaft from just outside the bead, then the head, all in the
    // direction of travel.
    let start = c.y + sign * (half_h + s * 0.15);
    let shaft_len = s * 0.45;
    let base_y = start + sign * shaft_len;
    let tip = pos2(c.x, base_y + sign * s * 0.75);
    let left = pos2(c.x - s * 0.62, base_y);
    let right = pos2(c.x + s * 0.62, base_y);
    let head = rounded_triangle([tip, left, right], s * 0.14);
    let shaft = Rect::from_two_pos(
        pos2(c.x - s * 0.17, start),
        pos2(c.x + s * 0.17, base_y + sign * s * 0.1),
    );
    // A thin bone outline (drawn first, slightly larger) keeps the arrow
    // readable over the beam and over bone beads.
    let outline = Stroke::new(1.5, crate::ui::ON_ACCENT);
    painter.rect(
        shaft,
        s * 0.17,
        Color32::TRANSPARENT,
        outline,
        StrokeKind::Outside,
    );
    painter.add(Shape::convex_polygon(
        head.clone(),
        Color32::TRANSPARENT,
        outline,
    ));
    let fill = with_alpha(pal.accent, 0.85 + 0.15 * pulse);
    painter.rect_filled(shaft, s * 0.17, fill);
    painter.add(Shape::convex_polygon(head, fill, Stroke::NONE));
}

/// A triangle with its corners rounded by `radius` (as a convex polygon).
fn rounded_triangle(v: [Pos2; 3], radius: f32) -> Vec<Pos2> {
    let mut out = Vec::with_capacity(15);
    for i in 0..3 {
        let p = v[i];
        let a = v[(i + 2) % 3];
        let b = v[(i + 1) % 3];
        let da = (a - p).normalized();
        let db = (b - p).normalized();
        let half_angle = da.dot(db).clamp(-1.0, 1.0).acos() * 0.5;
        // Distance from the corner to the tangent points.
        let t = radius / half_angle.tan().max(1e-3);
        let p0 = p + da * t;
        let p1 = p + db * t;
        // Quadratic Bezier from p0 through the corner to p1.
        for k in 0..5 {
            let u = k as f32 / 4.0;
            let q = p0.to_vec2() * (1.0 - u) * (1.0 - u)
                + p.to_vec2() * 2.0 * u * (1.0 - u)
                + p1.to_vec2() * u * u;
            out.push(q.to_pos2());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_eases_to_target() {
        let mut t = Tween::still(0.0);
        t.retarget(10.0, 1.0, 0.1, 0.0);
        assert!((t.at(10.0) - 0.0).abs() < 1e-6);
        assert!(t.at(10.05) > 0.5, "the settle curve passes halfway early");
        assert!((t.at(10.1) - 1.0).abs() < 1e-6);
        assert!(t.done(10.1));
        assert!(!t.done(10.05));
    }

    #[test]
    fn settle_is_monotonic_and_never_overshoots() {
        assert_eq!(settle(0.0), 0.0);
        assert!((settle(1.0) - 1.0).abs() < 1e-6);
        let mut prev = 0.0;
        for i in 0..=1000 {
            let v = settle(i as f32 / 1000.0);
            assert!(v >= prev - 1e-7, "not monotonic at {i}");
            assert!(v <= 1.0, "overshoot at {i}");
            prev = v;
        }
        // Decelerates: most of the travel happens in the first half.
        assert!(settle(0.5) > 0.8);
        // Starts from rest: the first step is small.
        assert!(settle(0.01) < 0.01);
    }

    #[test]
    fn glass_keeps_highlights_longest_then_vanishes() {
        assert_eq!(
            glass(0.0),
            Glass {
                body: 1.0,
                highlight: 1.0,
                overall: 1.0
            }
        );
        assert_eq!(glass(1.0).overall, 0.0);
        let mut prev = glass(0.0);
        for i in 1..=100 {
            let g = glass(i as f32 / 100.0);
            assert!(g.body <= prev.body && g.highlight <= prev.highlight);
            assert!(g.overall <= prev.overall);
            prev = g;
        }
        // Part-way the bead is glass: little body, highlights intact.
        let mid = glass(0.6);
        assert!(mid.body < 0.2 && mid.highlight > 0.6 && mid.overall > 0.99);
        // Everything goes over the last quarter only.
        assert!(glass(0.74).overall > 0.99);
    }

    fn instances(v: &SorobanView, now: f64) -> Vec<Instance> {
        let g = v.layout(Rect::from_min_size(pos2(0.0, 0.0), vec2(900.0, 400.0)));
        let mut out = Vec::new();
        v.build_instances(&g, Color32::RED, now, 1.0, &mut out);
        out
    }

    #[test]
    fn instances_are_built_in_draw_order() {
        let mut v = SorobanView::new(13);
        v.snap_to(&{
            let mut s = Soroban::new(13);
            s.set_value(4_071_583);
            s
        });
        v.active_rod = Some(4);
        v.hint = vec![Step::new(12, StepKind::HeavenDown, abax_core::Rule::Direct)];
        let out = instances(&v, 10.0);
        let count = |k: u32| out.iter().filter(|i| i.kind[0] == k).count();
        let dots = (0..13).filter(|i| v.model.has_unit_dot(*i)).count();
        assert_eq!(count(kind::BOARD), 1);
        assert_eq!(count(kind::WELL), 1);
        assert_eq!(count(kind::ACTIVE_GLOW), 1);
        assert_eq!(count(kind::UNDERLINE), 1);
        assert_eq!(count(kind::ROD), 13);
        assert_eq!(count(kind::BEAM), 1);
        assert_eq!(count(kind::DOT), dots);
        assert_eq!(count(kind::HINT_GLOW), 1);
        assert_eq!(count(kind::SHADOW), 65);
        assert_eq!(count(kind::BEAD), 65);
        // Board, well, glow, beam, underline, hint glow; rods; dots; beads.
        assert_eq!(out.len(), 6 + 13 + dots + 130);
        // Kinds are numbered in draw order, so the list must be sorted by
        // kind: board first, rods before beads, every shadow before every
        // bead.
        assert!(out.windows(2).all(|w| w[0].kind[0] <= w[1].kind[0]));
        let last_shadow = out.iter().rposition(|i| i.kind[0] == kind::SHADOW);
        let first_bead = out.iter().position(|i| i.kind[0] == kind::BEAD);
        let last_rod = out.iter().rposition(|i| i.kind[0] == kind::ROD);
        assert!(last_shadow < first_bead && last_rod < first_bead);
        // Rects are relative to the soroban's top-left corner.
        assert_eq!(out[0].rect[0], 0.0);
        assert_eq!(out[0].rect[1], 0.0);
    }

    #[test]
    fn instance_budget_covers_forty_rods_fully_hinted() {
        let mut v = SorobanView::new(40);
        v.active_rod = Some(0);
        v.hint = (0..40)
            .flat_map(|r| {
                [
                    Step::new(r, StepKind::HeavenDown, abax_core::Rule::Direct),
                    Step::new(r, StepKind::EarthUp(4), abax_core::Rule::Direct),
                ]
            })
            .collect();
        let out = instances(&v, 0.0);
        assert!(out.len() <= crate::render::MAX_INSTANCES, "{}", out.len());
        assert_eq!(
            out.iter().filter(|i| i.kind[0] == kind::HINT_GLOW).count(),
            200
        );
    }

    #[test]
    fn faded_beads_are_not_built() {
        let mut v = SorobanView::new(5);
        v.fade = 1.0;
        let out = instances(&v, 0.0);
        assert!(out
            .iter()
            .all(|i| i.kind[0] != kind::BEAD && i.kind[0] != kind::SHADOW));
        assert_eq!(out.iter().filter(|i| i.kind[0] == kind::ROD).count(), 5);
    }

    #[test]
    fn click_flash_lights_moving_beads_briefly() {
        let mut v = SorobanView::new(3);
        v.set_rod(1, Rod::from_digit(7), 5.0, 0.09, 0.0);
        v.flash_rod(1, 5.0);
        // Heaven bead and two earth beads move; the others do not.
        assert!(v.flash_at(1, 0, 5.0) > 0.99);
        assert!(v.flash_at(1, 1, 5.05) > 0.0);
        assert_eq!(v.flash_at(1, 3, 5.0), 0.0);
        assert_eq!(v.flash_at(0, 0, 5.0), 0.0);
        assert_eq!(v.flash_at(1, 0, 5.0 + FLASH_SECONDS), 0.0);
        let mid = v.flash_at(1, 0, 5.075);
        assert!(mid > 0.0 && mid < 1.0);
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
        v.hint = vec![Step::new(
            1,
            StepKind::EarthDown(1),
            abax_core::Rule::Direct,
        )];
        assert_eq!(v.hint_for_earth(1, 1), Some(1));
        assert_eq!(v.hint_for_earth(1, 0), None);
        v.hint = vec![Step::new(1, StepKind::HeavenDown, abax_core::Rule::Direct)];
        assert_eq!(v.hint_for(1, true), Some(1));
        assert_eq!(v.hint_for(0, true), None);
    }
}
