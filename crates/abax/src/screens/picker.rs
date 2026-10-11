//! Choose a drill by hand: skill, item count, sprint.

use abax_core::curriculum::SkillId;
use abax_core::scheduler::DrillSpec;
use egui::{RichText, Ui};

use crate::screens::drill::DrillMode;
use crate::App;

#[derive(Clone, Debug)]
pub struct PickerState {
    pub skill: Option<SkillId>,
    pub items: u8,
    pub sprint: bool,
    pub level_idx: Option<usize>,
}

impl Default for PickerState {
    fn default() -> Self {
        Self {
            skill: None,
            items: 10,
            sprint: false,
            level_idx: None,
        }
    }
}

impl PickerState {
    pub fn show(&mut self, app: &mut App, ui: &mut Ui) {
        let pal = app.palette;
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.go_home();
            }
            ui.heading(if self.sprint { "Sprint" } else { "Drills" });
        });
        ui.label(
            RichText::new(if self.sprint {
                "Sixty seconds, as many items as you can. The clock, not a mistake, ends it."
            } else {
                "Pick what to drill. Difficulty follows your current step for that skill; you can override it."
            })
            .color(pal.ink_soft),
        );
        ui.add_space(8.0);

        let unlocked: Vec<SkillId> = SkillId::ALL
            .iter()
            .copied()
            .filter(|s| {
                app.save.scheduler.is_unlocked(*s)
                    && !matches!(s, SkillId::FlashOne | SkillId::FlashMulti)
            })
            .collect();
        if self.skill.is_none() {
            self.skill = unlocked.last().copied();
        }
        let groups: [(&str, &[SkillId]); 6] = [
            ("Set and read", &[SkillId::Setting]),
            (
                "Add and subtract",
                &[
                    SkillId::AddSimple,
                    SkillId::SubSimple,
                    SkillId::AddFive,
                    SkillId::SubFive,
                    SkillId::AddTen,
                    SkillId::SubTen,
                    SkillId::Combined,
                ],
            ),
            ("Longer numbers", &[SkillId::MultiAdd, SkillId::MultiMixed]),
            ("Multiply", &[SkillId::Multiply]),
            ("Divide", &[SkillId::Divide]),
            ("Anzan", &[SkillId::Anzan]),
        ];
        for (title, skills) in groups {
            ui.label(RichText::new(title).strong());
            ui.horizontal_wrapped(|ui| {
                for s in skills {
                    let on = unlocked.contains(s);
                    let selected = self.skill == Some(*s);
                    let r = ui
                        .add_enabled_ui(on, |ui| ui.selectable_label(selected, s.short_name()))
                        .inner;
                    if r.clicked() {
                        self.skill = Some(*s);
                        self.level_idx = None;
                    }
                }
            });
        }
        ui.add_space(8.0);
        if let Some(skill) = self.skill {
            let ladder = skill.ladder();
            let cur = app.save.scheduler.state(skill).level.min(ladder.len() - 1);
            let idx = self.level_idx.unwrap_or(cur);
            ui.horizontal(|ui| {
                ui.label("Step");
                let mut v = idx + 1;
                ui.add(egui::Slider::new(&mut v, 1..=ladder.len()));
                if v - 1 != idx {
                    self.level_idx = Some(v - 1);
                }
                let l = ladder[idx];
                ui.label(
                    RichText::new(format!(
                        "{} digit{}, {} operand{}{}",
                        l.digits,
                        if l.digits == 1 { "" } else { "s" },
                        l.operands,
                        if l.operands == 1 { "" } else { "s" },
                        match l.timing {
                            abax_core::curriculum::Timing::Untimed => "",
                            abax_core::curriculum::Timing::Gentle => ", gently timed",
                            abax_core::curriculum::Timing::Tight => ", timed",
                        }
                    ))
                    .color(pal.ink_soft),
                );
            });
            if !self.sprint {
                ui.horizontal(|ui| {
                    ui.label("Items");
                    ui.add(egui::Slider::new(&mut self.items, 3..=30));
                });
            }
            ui.add_space(8.0);
            if ui
                .add(
                    egui::Button::new(RichText::new("Start").strong().size(18.0))
                        .min_size(egui::vec2(140.0, 40.0)),
                )
                .clicked()
            {
                let spec = DrillSpec {
                    skill,
                    level: ladder[idx],
                    items: self.items,
                    interleave: Vec::new(),
                    target_success: 0.8,
                };
                let mode = if self.sprint {
                    DrillMode::Sprint { seconds: 60.0 }
                } else {
                    DrillMode::Normal
                };
                app.start_drill(spec, mode);
            }
        }
    }
}
