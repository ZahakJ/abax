//! History and charts, drawn with the painter.

use abax_core::curriculum::SkillId;
use egui::{RichText, Ui};

use crate::charts;
use crate::App;

#[derive(Clone, Debug, Default)]
pub struct StatsState {
    pub skill: Option<SkillId>,
}

impl StatsState {
    pub fn show(&mut self, app: &mut App, ui: &mut Ui) {
        let pal = app.palette;
        let now = crate::storage::now_unix();
        if crate::ui::header(ui, &pal, None, "Stats", |_| {}) {
            app.go_home();
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mom = app.save.history.momentum(now);
            let tiles = [
                ("items, 14 days", mom.items_14d.to_string()),
                ("minutes, 14 days", format!("{:.0}", mom.minutes_14d)),
                ("days active", mom.days_active_14d.to_string()),
                (
                    "momentum",
                    format!("{}%", (mom.carried * 100.0).round() as i32),
                ),
            ];
            // Equal tiles: one row of four, or two by two on phones.
            let gap = 10.0;
            let cols = if ui.available_width() < 520.0 { 2 } else { 4 };
            let tile_w = (ui.available_width() - gap * (cols as f32 - 1.0)) / cols as f32;
            for row in tiles.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for (label, value) in row {
                        stat(ui, &pal, label, value, tile_w);
                    }
                });
                ui.add_space(gap - ui.spacing().item_spacing.y);
            }
            crate::ui::section(ui, &pal, "Items per day");
            let days = app.save.history.daily_totals(now, 14);
            let values: Vec<f32> = days.iter().map(|d| d.items as f32).collect();
            let labels: Vec<String> = days
                .iter()
                .map(|d| {
                    let dow = ((d.day_start / 86400 + 4) % 7) as usize; // 1970-01-01 was a Thursday
                    ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"][dow].to_string()
                })
                .collect();
            charts::bars(ui, &pal, &values, &labels, 150.0, "");

            crate::ui::section(ui, &pal, "Skills");
            for skill in SkillId::ALL {
                let started = app.save.scheduler.is_started(skill);
                let mastered = app.save.scheduler.is_mastered(skill);
                let p = app.save.scheduler.p_eff(skill, now) as f32;
                let label = if ui.available_width() > 640.0 {
                    skill.name()
                } else {
                    skill.short_name()
                };
                let r = ui.scope(|ui| {
                    charts::mastery_bar(ui, &pal, label, p, mastered, !started);
                });
                if r.response.interact(egui::Sense::click()).clicked() {
                    self.skill = Some(skill);
                }
            }

            let skill = self.skill.unwrap_or(SkillId::AddSimple);
            crate::ui::section(ui, &pal, "Detail");
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("stats-skill")
                    .selected_text(skill.name())
                    .show_ui(ui, |ui| {
                        for s in SkillId::ALL {
                            if ui.selectable_label(skill == s, s.name()).clicked() {
                                self.skill = Some(s);
                            }
                        }
                    });
            });
            let summary = app.save.history.skill_summary(skill);
            ui.label(
                RichText::new(format!(
                    "{} attempts · {}% accurate · typical {:.1} s",
                    summary.attempts,
                    (summary.accuracy * 100.0).round() as i32,
                    summary.median_seconds
                ))
                .color(pal.ink_soft),
            );
            ui.label(RichText::new("Accuracy (per 10 items)").color(pal.ink_soft));
            let acc = app.save.history.accuracy_series(skill, 10);
            charts::line(
                ui,
                &pal,
                &acc,
                120.0,
                Some((0.0, 1.0)),
                &|v| format!("{}%", (v * 100.0).round() as i32),
                pal.accent,
            );
            ui.label(RichText::new("Seconds per item (median of 10)").color(pal.ink_soft));
            let spd = app.save.history.speed_series(skill, 10);
            charts::line(ui, &pal, &spd, 120.0, None, &|v| format!("{v:.1}"), pal.ok);

            crate::ui::section(ui, &pal, "Recent sessions");
            let sessions = app.save.history.sessions(20 * 60);
            for s in sessions.iter().rev().take(10) {
                let mins = ((s.end - s.start) as f32 / 60.0).max(0.1);
                ui.label(format!(
                    "{}  ·  {} items, {} correct, {:.0} min",
                    date_label(s.start),
                    s.items,
                    s.correct,
                    mins
                ));
            }
            if sessions.is_empty() {
                ui.label(RichText::new("Nothing yet.").color(pal.ink_soft));
            }
            ui.add_space(20.0);
        });
    }
}

fn stat(ui: &mut Ui, pal: &crate::theme::Palette, label: &str, value: &str, width: f32) {
    egui::Frame::new()
        .fill(pal.paper2)
        .stroke(egui::Stroke::new(1.0, crate::ui::hairline(pal)))
        .corner_radius(10)
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            // Frame margin (14 each side) and stroke are outside this width.
            ui.set_width(width - 30.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(value)
                        .size(22.0)
                        .strong()
                        .monospace()
                        .color(pal.ink),
                );
                ui.label(RichText::new(label).color(pal.ink_soft).size(12.5));
            });
        });
}

/// YYYY-MM-DD from unix seconds (UTC), without a date dependency.
pub fn date_label(ts: u64) -> String {
    let days = (ts / 86400) as i64;
    // Civil-from-days (Howard Hinnant's algorithm).
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn date_labels() {
        assert_eq!(super::date_label(0), "1970-01-01");
        assert_eq!(super::date_label(951782400), "2000-02-29");
        assert_eq!(super::date_label(1_760_000_000), "2025-10-09");
    }
}
