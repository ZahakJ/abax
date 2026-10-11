//! Home: continue where you are, or pick a mode.

use egui::{RichText, Ui};

use crate::screens::drill::DrillMode;
use crate::App;

pub fn show(app: &mut App, ui: &mut Ui) {
    let pal = app.palette;
    let now = crate::storage::now_unix();
    let mom = app.save.history.momentum(now);
    let plan = app.save.scheduler.plan_session(now, &mut app.rng);
    let next_lesson = app.save.scheduler.next_skill_to_learn();

    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new("Abax").size(40.0).strong());
        ui.label(RichText::new("soroban trainer").color(pal.ink_soft));
    });
    ui.add_space(8.0);

    // Momentum line (no streaks).
    let momentum = if mom.items_14d == 0 {
        "No practice yet. A first session takes about five minutes.".to_string()
    } else {
        format!(
            "Last 14 days: {} items in {:.0} min over {} day{}. Momentum {}%.",
            mom.items_14d,
            mom.minutes_14d,
            mom.days_active_14d,
            if mom.days_active_14d == 1 { "" } else { "s" },
            (mom.carried * 100.0).round() as i32
        )
    };
    ui.label(RichText::new(momentum).color(pal.ink_soft));
    ui.add_space(8.0);

    // Continue card.
    egui::Frame::new()
        .fill(pal.paper2)
        .corner_radius(12)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let (title, body) = if let Some(skill) = next_lesson {
                (
                    format!("Next lesson: {}", skill.name()),
                    "A short explanation, a demonstration on the beads, then practice with hints."
                        .to_string(),
                )
            } else {
                let f = &plan.focus;
                (
                    format!("Today's focus: {}", f.skill.name()),
                    format!(
                        "{} items at step {} of {}{}.",
                        f.items,
                        f.skill
                            .ladder()
                            .iter()
                            .position(|l| *l == f.level)
                            .unwrap_or(0)
                            + 1,
                        f.skill.ladder().len(),
                        if plan.warmup.is_some() {
                            ", after a warm-up"
                        } else {
                            ""
                        }
                    ),
                )
            };
            ui.label(RichText::new(title).size(20.0).strong());
            ui.label(RichText::new(body).color(pal.ink_soft));
            ui.add_space(6.0);
            if ui
                .add(
                    egui::Button::new(RichText::new("Continue").strong().size(18.0))
                        .min_size(egui::vec2(160.0, 40.0))
                        .fill(pal.accent_soft),
                )
                .clicked()
            {
                if let Some(skill) = next_lesson {
                    app.screen =
                        crate::Screen::Lesson(crate::screens::lesson::LessonState::new(skill));
                } else {
                    app.start_session(plan.clone());
                }
            }
        });
    ui.add_space(10.0);

    let w = ui.available_width();
    let cols = if w > 560.0 { 4 } else { 2 };
    let bw = (w - (cols as f32 - 1.0) * 10.0) / cols as f32;
    let size = egui::vec2(bw, 56.0);
    let buttons: [(&str, &str); 8] = [
        ("Practice", "Free soroban"),
        ("Lessons", "Learn each idea"),
        ("Drills", "Set, read, add…"),
        ("Anzan", "Fading beads"),
        ("Flash", "Numbers flash"),
        ("Sprint", "60 seconds"),
        ("Stats", "History & charts"),
        ("Settings", "Rods, theme, sound"),
    ];
    egui::Grid::new("home-grid")
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            for (i, (label, sub)) in buttons.iter().enumerate() {
                let mut job = egui::text::LayoutJob::default();
                RichText::new(format!("{label}\n"))
                    .size(15.0)
                    .strong()
                    .append_to(
                        &mut job,
                        ui.style(),
                        egui::FontSelection::Default,
                        egui::Align::LEFT,
                    );
                RichText::new(*sub)
                    .size(15.0)
                    .color(pal.ink_soft)
                    .append_to(
                        &mut job,
                        ui.style(),
                        egui::FontSelection::Default,
                        egui::Align::LEFT,
                    );
                let r = ui.add(egui::Button::new(job).min_size(size));
                if r.clicked() {
                    match *label {
                        "Practice" => app.screen = crate::Screen::Practice,
                        "Lessons" => app.screen = crate::Screen::Lessons,
                        "Drills" => {
                            app.screen = crate::Screen::Picker(
                                crate::screens::picker::PickerState::default(),
                            )
                        }
                        "Anzan" => {
                            let skill = abax_core::curriculum::SkillId::Anzan;
                            let level = app.save.scheduler.current_level(skill);
                            let spec = abax_core::scheduler::DrillSpec {
                                skill,
                                level,
                                items: 8,
                                interleave: Vec::new(),
                                target_success: 0.8,
                            };
                            app.start_drill(spec, DrillMode::Anzan { fade_seconds: 2.5 });
                        }
                        "Flash" => app.start_flash(abax_core::curriculum::SkillId::Anzan),
                        "Sprint" => {
                            let p = crate::screens::picker::PickerState {
                                sprint: true,
                                ..Default::default()
                            };
                            app.screen = crate::Screen::Picker(p);
                        }
                        "Stats" => app.screen = crate::Screen::Stats(Default::default()),
                        "Settings" => app.screen = crate::Screen::Settings,
                        _ => {}
                    }
                }
                if (i + 1) % cols == 0 {
                    ui.end_row();
                }
            }
        });
}
