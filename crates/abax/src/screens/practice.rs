//! Free practice: the soroban, a keypad, a number display, nothing else.

use egui::{RichText, Ui};

use crate::keypad::{keyboard_actions, keypad_centered, KeyAction};
use crate::soroban_view::SorobanView;
use crate::App;

/// Keyboard entry on the soroban: digits set the active rod and move right,
/// Backspace clears it and moves left.
pub fn rod_keyboard(view: &mut SorobanView, a: KeyAction, now: f64) {
    let n = view.rod_count();
    match a {
        KeyAction::Digit(d) => {
            let rod = view.active_rod.unwrap_or(0).min(n - 1);
            view.set_rod(
                rod,
                abax_core::Rod::from_digit(d),
                now,
                view.motion.user_move,
                0.0,
            );
            view.flash_rod(rod, now);
            view.active_rod = Some((rod + 1).min(n - 1));
        }
        KeyAction::Backspace => {
            let rod = view.active_rod.unwrap_or(n - 1).min(n - 1);
            // If the active rod is empty, step left first (like backspace).
            let target = if view.model.digit(rod) == 0 && rod > 0 {
                rod - 1
            } else {
                rod
            };
            view.set_rod(
                target,
                abax_core::Rod::default(),
                now,
                view.motion.user_move,
                0.0,
            );
            view.flash_rod(target, now);
            view.active_rod = Some(target);
        }
        KeyAction::Clear => view.clear(now),
        KeyAction::Enter => {}
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let now = ui.input(|i| i.time);
    let pal = app.palette;
    let mut rods = app.view.rod_count();
    let mut clear = false;
    let back = crate::ui::header(ui, &pal, None, "Free practice", |ui| {
        if ui.button("Clear").clicked() {
            clear = true;
        }
        egui::ComboBox::from_id_salt("rods")
            .selected_text(format!("{rods} rods"))
            .width(90.0)
            .show_ui(ui, |ui| {
                for n in abax_core::soroban::ROD_COUNTS {
                    ui.selectable_value(&mut rods, n, format!("{n} rods"));
                }
            });
    });
    if back {
        app.go_home();
        return;
    }
    if clear {
        app.view.clear(now);
    }
    if rods != app.view.rod_count() {
        app.view.set_rod_count(rods);
        app.save.settings.rod_count = rods;
        app.mark_dirty();
    }
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(app.view.model.value().to_string())
                .size(34.0)
                .monospace()
                .color(pal.ink),
        );
    });
    // Arrow keys move the active rod.
    ui.input(|i| {
        let n = app.view.rod_count();
        if i.key_pressed(egui::Key::ArrowLeft) {
            app.view.active_rod = Some(app.view.active_rod.unwrap_or(n - 1).saturating_sub(1));
        }
        if i.key_pressed(egui::Key::ArrowRight) {
            app.view.active_rod = Some((app.view.active_rod.unwrap_or(0) + 1).min(n - 1));
        }
    });
    let avail = ui.available_size();
    let pad_h = if app.save.settings.show_keypad {
        4.0 * 46.0 + 40.0
    } else {
        20.0
    };
    let h = app
        .view
        .natural_height(avail.x)
        .min((avail.y - pad_h).max(140.0));
    let events = app.view.show(ui, &pal, egui::vec2(avail.x, h));
    for ev in &events {
        app.audio.click(ev.heaven);
    }
    let mut actions = keyboard_actions(ui);
    if app.save.settings.show_keypad {
        ui.add_space(6.0);
        let key = egui::vec2(
            ((ui.available_width() - 40.0) / 3.0).clamp(48.0, 96.0),
            44.0,
        );
        if let Some(a) = keypad_centered(ui, &pal, app.save.settings.left_handed, key) {
            actions.push(a);
        }
    }
    for a in actions {
        if a != KeyAction::Enter {
            rod_keyboard(&mut app.view, a, now);
            app.audio.click(false);
        }
    }
}
