//! Palette and egui visuals. See DESIGN.md for the tokens.

use egui::{Color32, CornerRadius, Stroke, Visuals};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub paper: Color32,
    pub paper2: Color32,
    pub ink: Color32,
    pub ink_soft: Color32,
    pub frame: Color32,
    pub frame_light: Color32,
    pub rod: Color32,
    pub beam: Color32,
    pub bead: Color32,
    pub bead_shadow: Color32,
    pub bead_hilite: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub ok: Color32,
    pub warn: Color32,
    pub dark: bool,
}

const fn c(hex: u32) -> Color32 {
    Color32::from_rgb(((hex >> 16) & 0xff) as u8, ((hex >> 8) & 0xff) as u8, (hex & 0xff) as u8)
}

pub const LIGHT: Palette = Palette {
    paper: c(0xF4EDE0),
    paper2: c(0xEADFCC),
    ink: c(0x2A2320),
    ink_soft: c(0x6E6259),
    frame: c(0x3B2A1E),
    frame_light: c(0x6B4A2E),
    rod: c(0xB08D5B),
    beam: c(0xC9A66B),
    bead: c(0xEFE6D6),
    bead_shadow: c(0xC8B89E),
    bead_hilite: c(0xFFFBF3),
    accent: c(0xC2452D),
    accent_soft: c(0xE9C4B8),
    ok: c(0x4F7A4A),
    warn: c(0xB8862B),
    dark: false,
};

pub const DARK: Palette = Palette {
    paper: c(0x1C1A17),
    paper2: c(0x26221E),
    ink: c(0xEDE6DA),
    ink_soft: c(0xA69C90),
    frame: c(0x2C1F16),
    frame_light: c(0x4A3424),
    rod: c(0x8C6F47),
    beam: c(0xA8894F),
    bead: c(0xE6DAC4),
    bead_shadow: c(0xA89675),
    bead_hilite: c(0xF7F0E2),
    accent: c(0xD8563B),
    accent_soft: c(0x5C2A20),
    ok: c(0x7FAF78),
    warn: c(0xD4A440),
    dark: true,
};

impl Palette {
    pub fn visuals(&self) -> Visuals {
        let mut v = if self.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        v.override_text_color = Some(self.ink);
        v.panel_fill = self.paper;
        v.window_fill = self.paper2;
        v.faint_bg_color = self.paper2;
        v.extreme_bg_color = if self.dark {
            c(0x141210)
        } else {
            c(0xFBF7EF)
        };
        v.code_bg_color = self.paper2;
        v.hyperlink_color = self.accent;
        v.warn_fg_color = self.warn;
        v.error_fg_color = self.accent;
        v.selection.bg_fill = self.accent_soft;
        v.selection.stroke = Stroke::new(1.0, self.accent);
        v.window_stroke = Stroke::new(1.0, self.frame_light);
        v.window_corner_radius = CornerRadius::same(10);
        v.menu_corner_radius = CornerRadius::same(8);

        let r = CornerRadius::same(8);
        let w = &mut v.widgets;
        w.noninteractive.bg_fill = self.paper2;
        w.noninteractive.weak_bg_fill = self.paper2;
        w.noninteractive.bg_stroke = Stroke::new(1.0, mix(self.paper2, self.ink_soft, 0.25));
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.ink_soft);
        w.noninteractive.corner_radius = r;
        w.inactive.bg_fill = mix(self.paper2, self.ink, 0.06);
        w.inactive.weak_bg_fill = mix(self.paper2, self.ink, 0.04);
        w.inactive.bg_stroke = Stroke::new(1.0, mix(self.paper2, self.ink_soft, 0.35));
        w.inactive.fg_stroke = Stroke::new(1.0, self.ink);
        w.inactive.corner_radius = r;
        w.hovered.bg_fill = mix(self.paper2, self.accent, 0.12);
        w.hovered.weak_bg_fill = mix(self.paper2, self.accent, 0.08);
        w.hovered.bg_stroke = Stroke::new(1.0, self.accent);
        w.hovered.fg_stroke = Stroke::new(1.5, self.ink);
        w.hovered.corner_radius = r;
        w.active.bg_fill = self.accent_soft;
        w.active.weak_bg_fill = self.accent_soft;
        w.active.bg_stroke = Stroke::new(1.0, self.accent);
        w.active.fg_stroke = Stroke::new(2.0, self.ink);
        w.active.corner_radius = r;
        w.open.bg_fill = self.paper2;
        w.open.weak_bg_fill = self.paper2;
        w.open.bg_stroke = Stroke::new(1.0, self.accent);
        w.open.fg_stroke = Stroke::new(1.0, self.ink);
        w.open.corner_radius = r;
        v
    }
}

/// Linear blend of two colours in gamma space (good enough for UI tints).
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgba_premultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

pub fn with_alpha(c: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (alpha.clamp(0.0, 1.0) * 255.0) as u8)
}

/// Install fonts sizes and spacing once.
pub fn apply_style(ctx: &egui::Context, palette: &Palette, large_text: bool) {
    ctx.set_visuals(palette.visuals());
    ctx.style_mut(|style| {
        use egui::{FontFamily, FontId, TextStyle};
        let scale = if large_text { 1.22 } else { 1.0 };
        style.text_styles = [
            (TextStyle::Heading, FontId::new(26.0 * scale, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(16.0 * scale, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(16.0 * scale, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.5 * scale, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(16.0 * scale, FontFamily::Monospace)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.interact_size = egui::vec2(44.0, 36.0);
        style.spacing.slider_width = 180.0;
        style.visuals.button_frame = true;
        style.interaction.selectable_labels = false;
        style.interaction.tooltip_delay = 0.4;
    });
}
