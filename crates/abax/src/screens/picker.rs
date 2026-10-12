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
        let title = if self.sprint { "Sprint" } else { "Drills" };
        if crate::ui::header(ui, &pal, None, title, |_| {}) {
            app.go_home();
            return;
        }
        ui.label(
            RichText::new(if self.sprint {
                "Sixty seconds, as many items as you can. The clock, not a mistake, ends it."
            } else {
                "Pick what to drill. Difficulty follows your current step for that skill; you can override it."
            })
            .color(pal.ink_soft),
        );
        ui.add_space(8.0);

        if self.skill.is_none() {
            self.skill = Some(default_skill(&app.save.scheduler));
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
            crate::ui::section(ui, &pal, title);
            ui.horizontal_wrapped(|ui| {
                for s in skills {
                    let selected = self.skill == Some(*s);
                    let r = ui.selectable_label(selected, s.short_name());
                    if r.clicked() {
                        self.skill = Some(*s);
                        self.level_idx = None;
                    }
                }
            });
        }
        ui.add_space(4.0);
        if let Some(skill) = self.skill {
            crate::ui::section(ui, &pal, "Difficulty");
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
            if crate::ui::primary_button(ui, &pal, "Start", egui::vec2(140.0, 40.0)).clicked() {
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

/// The skill the picker starts on: the furthest one in teaching order the
/// learner has met, else the suggested next lesson, else S0. Every skill can
/// be picked whatever this returns.
pub fn default_skill(sched: &abax_core::scheduler::Scheduler) -> SkillId {
    let drillable = |s: &SkillId| !matches!(s, SkillId::FlashOne | SkillId::FlashMulti);
    SkillId::ALL
        .into_iter()
        .rev()
        .find(|s| drillable(s) && sched.is_started(*s))
        .or_else(|| sched.next_skill_to_learn().filter(drillable))
        .unwrap_or(SkillId::Setting)
}

#[cfg(test)]
mod tests {
    use super::*;
    use abax_core::scheduler::Scheduler;

    #[test]
    fn default_skill_follows_what_the_learner_met() {
        let mut s = Scheduler::default();
        assert_eq!(default_skill(&s), SkillId::Setting);
        s.record(SkillId::Divide, true, 10.0, 1_000);
        assert_eq!(default_skill(&s), SkillId::Divide);
        s.mark_known(SkillId::FlashMulti);
        assert_eq!(
            default_skill(&s),
            SkillId::Divide,
            "flash is not a drill here"
        );
    }
}
