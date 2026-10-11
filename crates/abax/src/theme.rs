//! Palette and egui visuals. See DESIGN.md for the tokens.

use abax_core::store::Finish;
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
    Color32::from_rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
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

/// Rod (and metal beam) material, as the shader shades it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RodKind {
    /// Matte warm bamboo, soft broad highlight.
    Bamboo,
    /// Warm metal: narrow bright highlight and a darker reflection band.
    Brass,
    /// Cool metal.
    Steel,
}

/// Bead material, as the shader shades it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeadKind {
    /// Warm off-white with wrapped (subsurface-like) light, tight specular.
    Bone,
    /// Near-black with a warm sheen.
    Ebony,
    /// Blue-grey stone, soft broad specular.
    Slate,
}

/// Everything the soroban renderer needs for one finish in one mode: the
/// colours of each part and the shading constants. The app identity
/// (paper, ink, vermilion accent) stays in [`Palette`]; the instrument's
/// look comes from here. Colours are sRGB; the renderer converts them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub frame: Color32,
    /// Bevel highlight and the lighter grain.
    pub frame_light: Color32,
    /// The darker grain lines.
    pub frame_grain: Color32,
    /// The recessed well behind the rods.
    pub well: Color32,
    pub rod: Color32,
    pub rod_kind: RodKind,
    pub beam: Color32,
    pub beam_light: Color32,
    /// Whether the beam is metal (shaded like the rods) or lacquered wood.
    pub beam_metal: bool,
    pub bead: Color32,
    pub bead_kind: BeadKind,
    pub bead_shadow: Color32,
    pub bead_hilite: Color32,
    /// Unit-dot pins on the beam.
    pub dot: Color32,
    /// Rod digits printed on the lower frame.
    pub digit: Color32,
    /// Strength of the broad lacquer sheen (0..1).
    pub sheen: f32,
    /// Contrast of the wood grain (0..1).
    pub grain: f32,
    /// Grain line frequency: fine-grained woods (ebony, maple) are higher.
    pub grain_scale: f32,
    /// Contact-shadow opacity under each bead.
    pub shadow_alpha: f32,
}

impl Material {
    /// Whether the well is pale (maple): glows need less strength there.
    pub fn well_is_light(&self) -> bool {
        let w = self.well;
        (w.r() as u32 * 2126 + w.g() as u32 * 7152 + w.b() as u32 * 722) / 10_000 > 110
    }
}

/// The finish × mode table. Walnut is the identity look; Ebony and Maple
/// keep the same vermilion accent and the same layout.
pub fn material(finish: Finish, dark: bool) -> Material {
    match (finish, dark) {
        (Finish::Walnut, false) => Material {
            frame: c(0x3E2B1E),
            frame_light: c(0x75512F),
            frame_grain: c(0x2A1C12),
            well: c(0x221710),
            rod: c(0xB8935C),
            rod_kind: RodKind::Bamboo,
            beam: c(0xC9A66B),
            beam_light: c(0xE6CB92),
            beam_metal: false,
            bead: c(0xEFE6D6),
            bead_kind: BeadKind::Bone,
            bead_shadow: c(0xC2B092),
            bead_hilite: c(0xFFFCF5),
            dot: c(0x3A2818),
            digit: c(0xE9DCC4),
            sheen: 0.10,
            grain: 0.55,
            grain_scale: 1.0,
            shadow_alpha: 0.32,
        },
        (Finish::Walnut, true) => Material {
            frame: c(0x2E2017),
            frame_light: c(0x5A3F28),
            frame_grain: c(0x1E140D),
            well: c(0x160F0A),
            rod: c(0x9A7A4C),
            rod_kind: RodKind::Bamboo,
            beam: c(0x9C7E4B),
            beam_light: c(0xBE9F68),
            beam_metal: false,
            bead: c(0xE6DAC4),
            bead_kind: BeadKind::Bone,
            bead_shadow: c(0xA89675),
            bead_hilite: c(0xF7F0E2),
            dot: c(0x2E2014),
            digit: c(0xD9CCB4),
            sheen: 0.06,
            grain: 0.5,
            grain_scale: 1.0,
            shadow_alpha: 0.45,
        },
        (Finish::Ebony, false) => Material {
            frame: c(0x221D1A),
            frame_light: c(0x4A403A),
            frame_grain: c(0x15110F),
            well: c(0x120F0D),
            rod: c(0xB08E48),
            rod_kind: RodKind::Brass,
            beam: c(0x9C7C3C),
            beam_light: c(0xD6BC80),
            beam_metal: true,
            bead: c(0xEEE5D3),
            bead_kind: BeadKind::Bone,
            bead_shadow: c(0xBFAE90),
            bead_hilite: c(0xFFFCF5),
            dot: c(0x3A2A12),
            digit: c(0xE4D3AE),
            sheen: 0.12,
            grain: 0.35,
            grain_scale: 2.2,
            shadow_alpha: 0.36,
        },
        (Finish::Ebony, true) => Material {
            frame: c(0x221D1A),
            frame_light: c(0x4E433C),
            frame_grain: c(0x100D0B),
            well: c(0x0D0B0A),
            rod: c(0x9A7A3A),
            rod_kind: RodKind::Brass,
            beam: c(0x87692F),
            beam_light: c(0xBFA266),
            beam_metal: true,
            bead: c(0xE4D8C1),
            bead_kind: BeadKind::Bone,
            bead_shadow: c(0xA59271),
            bead_hilite: c(0xF6EFE0),
            dot: c(0x2E220E),
            digit: c(0xCDBD98),
            sheen: 0.07,
            grain: 0.3,
            grain_scale: 2.2,
            shadow_alpha: 0.5,
        },
        (Finish::Maple, false) => Material {
            frame: c(0xD9BF95),
            frame_light: c(0xF1DDB8),
            frame_grain: c(0xC2A174),
            well: c(0xB79A70),
            rod: c(0xA3ABB3),
            rod_kind: RodKind::Steel,
            beam: c(0x6E4F35),
            beam_light: c(0x93714F),
            beam_metal: false,
            bead: c(0x5B6773),
            bead_kind: BeadKind::Slate,
            bead_shadow: c(0x3B444E),
            bead_hilite: c(0xD5DEE7),
            dot: c(0xE6D3AE),
            digit: c(0x3A2B1E),
            sheen: 0.10,
            grain: 0.4,
            grain_scale: 1.8,
            shadow_alpha: 0.30,
        },
        (Finish::Maple, true) => Material {
            frame: c(0x9C805D),
            frame_light: c(0xBDA07A),
            frame_grain: c(0x86694A),
            well: c(0x6A543C),
            rod: c(0x8E979F),
            rod_kind: RodKind::Steel,
            beam: c(0x4E3826),
            beam_light: c(0x6E5238),
            beam_metal: false,
            bead: c(0x66727E),
            bead_kind: BeadKind::Slate,
            bead_shadow: c(0x3A424B),
            bead_hilite: c(0xC8D2DC),
            dot: c(0xC9B590),
            digit: c(0x2A1F16),
            sheen: 0.06,
            grain: 0.35,
            grain_scale: 1.8,
            shadow_alpha: 0.45,
        },
    }
}

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
        v.extreme_bg_color = if self.dark { c(0x141210) } else { c(0xFBF7EF) };
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
    let l = |x: u8, y: u8| {
        (x as f32 + (y as f32 - x as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgba_premultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

pub fn with_alpha(c: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (alpha.clamp(0.0, 1.0) * 255.0) as u8)
}

/// Install fonts sizes and spacing once.
pub fn apply_style(ctx: &egui::Context, palette: &Palette, large_text: bool) {
    ctx.set_visuals(palette.visuals());
    ctx.all_styles_mut(|style| {
        use egui::{FontFamily, FontId, TextStyle};
        let scale = if large_text { 1.22 } else { 1.0 };
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(26.0 * scale, FontFamily::Proportional),
            ),
            (
                TextStyle::Body,
                FontId::new(16.0 * scale, FontFamily::Proportional),
            ),
            (
                TextStyle::Button,
                FontId::new(16.0 * scale, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(12.5 * scale, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(16.0 * scale, FontFamily::Monospace),
            ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_table_covers_every_finish_and_mode() {
        let mut all = Vec::new();
        for f in Finish::ALL {
            let light = material(f, false);
            let dark = material(f, true);
            assert_ne!(light, dark, "{f:?}: dark mode must differ");
            // The well is recessed: darker than the frame face.
            for m in [light, dark] {
                let lum = |c: Color32| c.r() as u32 + c.g() as u32 + c.b() as u32;
                assert!(lum(m.well) < lum(m.frame), "{f:?}: well must be darker");
                assert!(m.shadow_alpha > 0.0 && m.shadow_alpha < 0.6);
            }
            // Dark mode shadows are stronger, sheen lower.
            assert!(dark.shadow_alpha > light.shadow_alpha);
            assert!(dark.sheen <= light.sheen);
            all.push((f, light));
            all.push((f, dark));
        }
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                if a.0 != b.0 {
                    assert_ne!(a.1.frame, b.1.frame, "{:?} vs {:?}", a.0, b.0);
                }
            }
        }
        assert_eq!(material(Finish::Walnut, false).rod_kind, RodKind::Bamboo);
        assert_eq!(material(Finish::Ebony, false).rod_kind, RodKind::Brass);
        assert!(material(Finish::Ebony, true).beam_metal);
        assert_eq!(material(Finish::Maple, false).bead_kind, BeadKind::Slate);
        assert_eq!(material(Finish::Maple, true).rod_kind, RodKind::Steel);
    }
}
