//! Shared screen furniture so every screen reads as one instrument: the
//! header row, cards, section labels, the primary button and the type
//! scale.

use egui::{Align, Color32, Layout, Response, RichText, Sense, Stroke, Ui, Vec2};

use crate::theme::{mix, with_alpha, Palette};

/// Type scale (points). One size per level; secondary text uses ink-soft.
pub mod size {
    /// The app name on Home.
    pub const DISPLAY: f32 = 40.0;
    /// Screen title in the header.
    pub const TITLE: f32 = 20.0;
    /// Card titles and prompts that are not the main question.
    pub const CARD_TITLE: f32 = 18.0;
    /// The question in a drill.
    pub const PROMPT: f32 = 30.0;
    /// Section labels.
    pub const SECTION: f32 = 12.5;
}

/// The 1 px line used for card borders and section rules.
pub fn hairline(pal: &Palette) -> Color32 {
    mix(pal.paper2, pal.ink_soft, if pal.dark { 0.30 } else { 0.28 })
}

/// Text on vermilion: warm bone, the same in both modes.
pub const ON_ACCENT: Color32 = Color32::from_rgb(0xFB, 0xF5, 0xEA);

/// The header every screen starts with: a Back button, the title (with an
/// optional ink-soft breadcrumb before it) and trailing controls laid out
/// right to left. Returns true when Back was pressed.
pub fn header(
    ui: &mut Ui,
    pal: &Palette,
    crumb: Option<&str>,
    title: &str,
    trailing: impl FnOnce(&mut Ui),
) -> bool {
    let mut back = false;
    // Phones: a step down so Back, title and trailing controls share a row.
    let title_size = if ui.available_width() < 480.0 {
        size::TITLE * 0.85
    } else {
        size::TITLE
    };
    ui.horizontal(|ui| {
        ui.set_min_height(36.0);
        if ui.button("Back").clicked() {
            back = true;
        }
        ui.add_space(4.0);
        if let Some(c) = crumb {
            ui.label(RichText::new(c).color(pal.ink_soft).size(title_size * 0.8));
            ui.label(
                RichText::new("/")
                    .color(hairline(pal))
                    .size(title_size * 0.8),
            );
        }
        ui.label(
            RichText::new(title)
                .size(title_size)
                .strong()
                .color(pal.ink),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), trailing);
    });
    rule(ui, pal, 4.0);
    back
}

/// A full-width hairline with `space` above and below.
pub fn rule(ui: &mut Ui, pal: &Palette, space: f32) {
    ui.add_space(space);
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), Sense::hover());
    ui.painter()
        .hline(r.x_range(), r.center().y, Stroke::new(1.0, hairline(pal)));
    ui.add_space(space);
}

/// A panel: paper-2 fill, 1 px hairline, a soft shadow, 12 radius, full
/// width.
pub fn card<R>(ui: &mut Ui, pal: &Palette, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new()
        .fill(pal.paper2)
        .stroke(Stroke::new(1.0, hairline(pal)))
        .corner_radius(12)
        .inner_margin(14)
        .shadow(egui::epaint::Shadow {
            offset: [0, 2],
            blur: 10,
            spread: 0,
            color: Color32::from_black_alpha(if pal.dark { 70 } else { 18 }),
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// A small ink-soft section label followed by a hairline.
pub fn section(ui: &mut Ui, pal: &Palette, title: &str) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(title.to_uppercase())
            .size(size::SECTION)
            .color(pal.ink_soft)
            .strong(),
    );
    rule(ui, pal, 2.0);
}

/// The one solid vermilion button per screen (bone text).
pub fn primary_button(ui: &mut Ui, pal: &Palette, text: &str, min: Vec2) -> Response {
    let button = egui::Button::new(RichText::new(text).strong().color(ON_ACCENT))
        .fill(pal.accent)
        .stroke(Stroke::new(1.0, mix(pal.accent, Color32::BLACK, 0.18)));
    if min.x > 0.0 {
        // A button lays its label out with the parent layout's alignment;
        // `add_sized` gives it a centred layout of the requested size.
        let h = min.y.max(ui.spacing().interact_size.y);
        ui.add_sized(egui::vec2(min.x, h), button)
    } else {
        ui.add(button)
    }
}

/// A key-like button: the given fill, a soft shadow beneath, a hairline,
/// pressed state sinks by one point. Used by the keypad.
pub fn key_button(
    ui: &mut Ui,
    pal: &Palette,
    label: &str,
    size: Vec2,
    fill: Color32,
    text: Color32,
    font_size: f32,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let pressed = resp.is_pointer_button_down_on();
        let hovered = resp.hovered();
        let sink = if pressed { 1.0 } else { 0.0 };
        let r = rect.translate(egui::vec2(0.0, sink));
        let shadow = egui::epaint::Shadow {
            offset: [0, if pressed { 1 } else { 2 }],
            blur: if pressed { 3 } else { 6 },
            spread: 0,
            color: Color32::from_black_alpha(if pal.dark { 90 } else { 34 }),
        };
        p.add(shadow.as_shape(r, 9));
        let f = if pressed {
            mix(fill, Color32::BLACK, 0.08)
        } else if hovered {
            mix(fill, Color32::WHITE, if pal.dark { 0.05 } else { 0.25 })
        } else {
            fill
        };
        let edge = if hovered {
            with_alpha(pal.accent, 0.8)
        } else {
            mix(fill, Color32::BLACK, if pal.dark { 0.35 } else { 0.14 })
        };
        p.rect(r, 9.0, f, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
        // A faint top highlight, like a lit key edge.
        p.hline(
            (r.left() + 8.0)..=(r.right() - 8.0),
            r.top() + 1.5,
            Stroke::new(
                1.0,
                with_alpha(Color32::WHITE, if pal.dark { 0.05 } else { 0.5 }),
            ),
        );
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(font_size),
            text,
        );
    }
    resp
}
