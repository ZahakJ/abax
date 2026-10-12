//! Settings and about.

use abax_core::store::{Finish, InputMode, Theme};
use egui::{RichText, Ui};

use crate::App;

pub fn show(app: &mut App, ui: &mut Ui) {
    let pal = app.palette;
    if crate::ui::header(ui, &pal, None, "Settings", |_| {}) {
        app.go_home();
        return;
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        let mut changed = false;
        {
            let s = &mut app.save.settings;
            crate::ui::section(ui, &pal, "Soroban");
            ui.horizontal(|ui| {
                ui.label("Rods");
                for n in abax_core::soroban::ROD_COUNTS {
                    if ui
                        .selectable_label(s.rod_count == n, n.to_string())
                        .clicked()
                    {
                        s.rod_count = n;
                        changed = true;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Beads move by");
                for (m, label) in [
                    (InputMode::Tap, "tap"),
                    (InputMode::Drag, "drag"),
                    (InputMode::Both, "both"),
                ] {
                    if ui.selectable_label(s.input_mode == m, label).clicked() {
                        s.input_mode = m;
                        changed = true;
                    }
                }
            });
            changed |= ui
                .checkbox(
                    &mut s.show_numbers,
                    "Show the number under the rods in free practice",
                )
                .changed();
            changed |= ui
                .checkbox(&mut s.show_keypad, "Show the on-screen keypad")
                .changed();
            changed |= ui
                .checkbox(
                    &mut s.left_handed,
                    "Left-handed layout (keypad on the left)",
                )
                .changed();
            ui.horizontal(|ui| {
                ui.label("Hint after");
                changed |= ui
                    .add(egui::Slider::new(&mut s.hint_delay_seconds, 1.5..=10.0).suffix(" s"))
                    .changed();
            });

            crate::ui::section(ui, &pal, "Appearance");
            ui.horizontal(|ui| {
                ui.label("Theme");
                for (t, label) in [
                    (Theme::System, "system"),
                    (Theme::Light, "light"),
                    (Theme::Dark, "dark"),
                ] {
                    if ui.selectable_label(s.theme == t, label).clicked() {
                        s.theme = t;
                        changed = true;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Finish");
                for f in Finish::ALL {
                    if ui.selectable_label(s.finish == f, f.name()).clicked() {
                        s.finish = f;
                        changed = true;
                    }
                }
            });
            changed |= ui.checkbox(&mut s.large_text, "Large text").changed();
            changed |= ui.checkbox(&mut s.reduce_motion, "Reduce motion").changed();

            crate::ui::section(ui, &pal, "Sound");
            changed |= ui.checkbox(&mut s.sound, "Bead click").changed();
            ui.horizontal(|ui| {
                ui.label("Volume");
                changed |= ui
                    .add(egui::Slider::new(&mut s.volume, 0.0..=1.0))
                    .changed();
            });
        }
        if !app.audio.available() {
            ui.label(
                RichText::new("No audio output is available on this device in this version.")
                    .color(pal.ink_soft),
            );
        }
        if changed {
            app.apply_settings();
            app.mark_dirty();
            if app.save.settings.sound {
                app.audio.click(false);
            }
        }

        crate::ui::section(ui, &pal, "Data");
        ui.label(
            RichText::new(format!(
                "Profile: {}",
                crate::storage::profile_path().display()
            ))
            .color(pal.ink_soft)
            .small(),
        );
        ui.horizontal(|ui| {
            if ui.button("Save now").clicked() {
                app.save_now();
            }
            if ui
                .button(RichText::new("Reset all progress").color(pal.accent))
                .clicked()
            {
                app.confirm_reset = true;
            }
        });
        reset_one_skill(app, ui);
        if app.confirm_reset {
            ui.label(
                RichText::new("This deletes lessons progress, mastery and history. Settings stay.")
                    .color(pal.accent),
            );
            ui.horizontal(|ui| {
                if ui.button("Yes, reset").clicked() {
                    let settings = app.save.settings.clone();
                    app.save = abax_core::store::SaveFile::new(crate::storage::now_unix());
                    app.save.settings = settings;
                    app.mark_dirty();
                    app.save_now();
                    app.confirm_reset = false;
                }
                if ui.button("Keep it").clicked() {
                    app.confirm_reset = false;
                }
            });
        }

        crate::ui::section(ui, &pal, "About");
        ui.label(format!("Abax {}", env!("CARGO_PKG_VERSION")));
        ui.label(
            RichText::new(
                "A soroban trainer. Rust, egui and wgpu. Offline; nothing leaves this device.",
            )
            .color(pal.ink_soft),
        );
        let (avg, p95) = app.frame_stats();
        ui.label(
            RichText::new(format!(
                "Frame time: {avg:.2} ms average, {p95:.2} ms p95 (last 120 frames)"
            ))
            .color(pal.ink_soft)
            .small(),
        );
        ui.add_space(20.0);
    });
}

/// "Reset one skill": undo "I know this already" (or any progress) for a
/// single skill, with a second tap when real practice would be lost.
fn reset_one_skill(app: &mut App, ui: &mut Ui) {
    let pal = app.palette;
    let id = egui::Id::new("settings-reset-skill");
    let (mut pick, mut armed): (abax_core::curriculum::SkillId, bool) = ui
        .data(|d| d.get_temp(id))
        .unwrap_or((abax_core::curriculum::SkillId::Setting, false));
    ui.horizontal_wrapped(|ui| {
        ui.label("Reset one skill");
        egui::ComboBox::from_id_salt("reset-skill-pick")
            .selected_text(format!("{} {}", pick.code(), pick.short_name()))
            .show_ui(ui, |ui| {
                for s in abax_core::curriculum::SkillId::ALL {
                    let st = app.save.scheduler.state(s);
                    let tag = if st.marked_known && st.n == 0 {
                        " (known)"
                    } else if st.n > 0 {
                        " (practised)"
                    } else {
                        ""
                    };
                    let text = format!("{} {}{}", s.code(), s.short_name(), tag);
                    if ui.selectable_label(pick == s, text).clicked() {
                        pick = s;
                        armed = false;
                    }
                }
            });
        let st = app.save.scheduler.state(pick);
        let started = st.n > 0 || st.marked_known;
        let label = if armed {
            "Tap again to reset"
        } else {
            "Reset this skill"
        };
        if ui
            .add_enabled(
                started,
                egui::Button::new(RichText::new(label).color(pal.accent)),
            )
            .clicked()
        {
            if st.n == 0 || armed {
                app.save.scheduler.reset_skill(pick);
                app.mark_dirty();
                armed = false;
            } else {
                armed = true;
            }
        }
    });
    ui.data_mut(|d| d.insert_temp(id, (pick, armed)));
}
