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

    let narrow = ui.available_width() < 560.0;
    ui.vertical_centered(|ui| {
        ui.add_space(if narrow { 4.0 } else { 8.0 });
        ui.label(
            RichText::new("Abax")
                .size(crate::ui::size::DISPLAY)
                .strong()
                .color(pal.ink),
        );
        ui.label(RichText::new("soroban trainer").color(pal.ink_soft));
    });
    ui.add_space(6.0);

    // The identity mark: a small live soroban (seven rods, not interactive)
    // showing how many items were practised in the last 14 days.
    {
        let t = ui.input(|i| i.time);
        let items = (mom.items_14d as u128).min(9_999_999);
        if app.home_view.model.value() != items {
            let mut target = abax_core::Soroban::new(7);
            target.set_value(items);
            let d = app.home_view.motion.demo_move;
            app.home_view.animate_to(&target, t, d);
        }
        let w = ui.available_width().min(if narrow { 300.0 } else { 340.0 });
        let h = app
            .home_view
            .natural_height(w)
            .min(if narrow { 150.0 } else { 170.0 });
        ui.vertical_centered(|ui| {
            app.home_view.show(ui, &pal, egui::vec2(w, h));
        });
    }

    // Momentum line (no streaks); the soroban above shows the item count.
    let momentum = if mom.items_14d == 0 {
        "No practice yet. A first session takes about five minutes.".to_string()
    } else {
        format!(
            "{} items in the last 14 days, {:.0} min over {} day{}. Momentum {}%.",
            mom.items_14d,
            mom.minutes_14d,
            mom.days_active_14d,
            if mom.days_active_14d == 1 { "" } else { "s" },
            (mom.carried * 100.0).round() as i32
        )
    };
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(momentum).color(pal.ink_soft));
    });
    ui.add_space(8.0);

    // Continue card.
    crate::ui::card(ui, &pal, |ui| {
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
        ui.label(
            RichText::new(title)
                .size(crate::ui::size::CARD_TITLE)
                .strong()
                .color(pal.ink),
        );
        ui.label(RichText::new(body).color(pal.ink_soft));
        ui.add_space(6.0);
        if crate::ui::primary_button(ui, &pal, "Continue", egui::vec2(160.0, 40.0)).clicked() {
            if let Some(skill) = next_lesson {
                app.screen = crate::Screen::Lesson(crate::screens::lesson::LessonState::new(skill));
            } else {
                app.start_session(plan.clone());
            }
        }
    });
    ui.add_space(10.0);

    let w = ui.available_width();
    let cols = if w > 560.0 { 4 } else { 2 };
    let bw = (w - (cols as f32 - 1.0) * 10.0) / cols as f32;
    let size = egui::vec2(bw, 58.0);
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
                    .size(16.0)
                    .strong()
                    .color(pal.ink)
                    .append_to(
                        &mut job,
                        ui.style(),
                        egui::FontSelection::Default,
                        egui::Align::LEFT,
                    );
                RichText::new(*sub)
                    .size(13.5)
                    .color(pal.ink_soft)
                    .append_to(
                        &mut job,
                        ui.style(),
                        egui::FontSelection::Default,
                        egui::Align::LEFT,
                    );
                let r = ui.add(
                    egui::Button::new(job)
                        .min_size(size)
                        .fill(pal.paper2)
                        .stroke(egui::Stroke::new(1.0, crate::ui::hairline(&pal)))
                        .corner_radius(10),
                );
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
