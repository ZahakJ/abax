//! The adaptive scheduler: Bayesian knowledge tracing per skill, an Elo
//! ability estimate against generated-item difficulty, a difficulty ladder
//! with step-up / step-down rules, a mastery gate with a fluency check, an
//! expanding review schedule with forgetting, and the per-session planner.
//!
//! Time is plain unix seconds (`u64`); randomness comes from a small
//! xorshift generator so every decision is reproducible in tests.
//!
//! Times per item are normalised to seconds *per operand* before they enter
//! the log-time average; benchmarks and time budgets are per operand.
//! The model is documented in docs/manual/scheduler.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::curriculum::{Level, SkillId, Timing};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// BKT prior P(L0).
pub const BKT_P_L0: f64 = 0.10;
/// BKT transition probability P(T).
pub const BKT_P_T: f64 = 0.10;
/// BKT guess probability P(G).
pub const BKT_P_G: f64 = 0.20;
/// BKT slip probability P(S).
pub const BKT_P_S: f64 = 0.10;

/// Accuracy EWMA weight.
pub const ACC_ALPHA: f64 = 0.2;
/// Log-time EWMA weight.
pub const TIME_ALPHA: f64 = 0.3;

/// Elo uncertainty function α.
pub const ELO_ALPHA: f64 = 1.0;
/// Elo uncertainty function β.
pub const ELO_BETA: f64 = 0.06;

/// A prerequisite counts as satisfied at this `p_known`. Since 0.3.0 this
/// only shapes recommendations (the suggested next lesson, the "builds on"
/// note, the planner's light review); no skill is ever locked. It is also
/// the estimate "I know this already" seeds a skill with.
pub const UNLOCK_THRESHOLD: f64 = 0.80;
/// Items in the light review of a weak prerequisite the planner may add.
pub const PREREQ_REVIEW_ITEMS: u8 = 5;
/// `p_known` needed for the mastery flag.
pub const MASTERY_P_KNOWN: f64 = 0.95;
/// Attempts needed for the mastery flag.
pub const MASTERY_MIN_N: u32 = 10;
/// Accuracy needed for the mastery flag.
pub const MASTERY_ACC: f64 = 0.90;
/// Fluency gate: typical time per operand must be within this multiple of
/// the skill benchmark.
pub const MASTERY_TIME_FACTOR: f64 = 1.5;

/// Step-up thresholds.
pub const STEP_UP_ACC: f64 = 0.85;
pub const STEP_UP_P: f64 = 0.85;
pub const STEP_UP_MIN_ITEMS: u32 = 8;
pub const STEP_UP_TIME_FACTOR: f64 = 0.8;
/// A timer is never introduced below this accuracy at the untimed step.
pub const TIMER_INTRO_ACC: f64 = 0.90;

/// Step-down thresholds.
pub const STEP_DOWN_ACC: f64 = 0.60;
pub const STEP_DOWN_MIN_ITEMS: u32 = 6;
pub const STEP_DOWN_CONSECUTIVE_ERRORS: u8 = 2;

/// Floor of the time budget, seconds per operand.
pub const BUDGET_FLOOR_SECONDS: f64 = 1.2;

/// Forgetting time constant is this many days times `review_interval_days`.
pub const FORGET_TAU_DAYS: f64 = 14.0;
/// A mastered skill is due for review below this effective probability.
pub const REVIEW_DUE_P: f64 = 0.80;
pub const REVIEW_GROW: f64 = 1.6;
pub const REVIEW_MAX_DAYS: f64 = 60.0;
pub const REVIEW_GROW_ACC: f64 = 0.90;
pub const REVIEW_RESET_ACC: f64 = 0.70;

const SECONDS_PER_DAY: f64 = 86_400.0;

// ---------------------------------------------------------------------------
// Pure update functions
// ---------------------------------------------------------------------------

/// One Bayesian-knowledge-tracing step: posterior given the observation,
/// then the learning transition.
pub fn bkt_update(l: f64, correct: bool) -> f64 {
    let post = if correct {
        let num = l * (1.0 - BKT_P_S);
        num / (num + (1.0 - l) * BKT_P_G)
    } else {
        let num = l * BKT_P_S;
        num / (num + (1.0 - l) * (1.0 - BKT_P_G))
    };
    post + (1.0 - post) * BKT_P_T
}

/// Elo uncertainty `U(n) = α / (1 + β n)`.
pub fn elo_u(n: u32) -> f64 {
    ELO_ALPHA / (1.0 + ELO_BETA * f64::from(n))
}

/// Logistic probability of a correct answer.
pub fn p_correct(theta: f64, d: f64) -> f64 {
    1.0 / (1.0 + (-(theta - d)).exp())
}

/// Difficulty of a generated item at a ladder step.
pub fn item_difficulty(skill: SkillId, level: &Level) -> f64 {
    let digits = f64::from(level.digits.max(1) - 1);
    let operands = f64::from(level.operands.saturating_sub(2));
    let timed = if level.timing == Timing::Untimed {
        0.0
    } else {
        1.0
    };
    let flash = if level.flash_ms > 0 { 1.0 } else { 0.0 };
    skill.base_difficulty() + 0.40 * digits + 0.25 * operands + 0.50 * timed + 0.30 * flash
}

fn ewma(current: f64, sample: f64, alpha: f64) -> f64 {
    current + alpha * (sample - current)
}

fn operands_of(level: &Level) -> f64 {
    f64::from(level.operands.max(1))
}

// ---------------------------------------------------------------------------
// Per-skill state
// ---------------------------------------------------------------------------

/// Everything the scheduler remembers about one skill.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillState {
    /// BKT probability that the skill is known.
    pub p_known: f64,
    /// Exponentially weighted accuracy.
    pub acc_ewma: f64,
    /// EWMA of ln(seconds per operand), correct answers only.
    pub log_t_ewma: f64,
    /// Elo ability on this skill.
    pub theta: f64,
    /// Attempts on this skill.
    pub n: u32,
    /// Current step on the skill's ladder.
    pub level: usize,
    /// Unix seconds of the last attempt; 0 = never.
    pub last_practised: u64,
    /// Unix seconds when the mastery flag was set.
    pub mastered_at: Option<u64>,
    /// Current spacing of reviews once mastered.
    pub review_interval_days: f64,
    /// Errors in a row on the current step.
    pub consecutive_errors: u8,
    /// Attempts since the last step change.
    pub items_at_level: u32,
    /// Time budget per operand fixed when a timed step was entered; 0 when
    /// unset. Anchoring it keeps the step-up fluency check meaningful (a
    /// budget that followed the live average could never be beaten).
    pub budget_seconds: f64,
    /// The learner said "I know this already" (added in 0.3.0; older saves
    /// read `false`). The estimate was seeded, not earned, so the planner
    /// keeps checking the skill with a few drills.
    pub marked_known: bool,
}

impl Default for SkillState {
    fn default() -> Self {
        Self::new(SkillId::AddSimple)
    }
}

impl SkillState {
    /// Fresh state for a skill: the log-time average starts at three times
    /// the skill's benchmark.
    pub fn new(skill: SkillId) -> Self {
        SkillState {
            p_known: BKT_P_L0,
            acc_ewma: 0.5,
            log_t_ewma: (skill.benchmark_seconds() * 3.0).ln(),
            theta: 0.0,
            n: 0,
            level: 0,
            last_practised: 0,
            mastered_at: None,
            review_interval_days: 1.0,
            consecutive_errors: 0,
            items_at_level: 0,
            budget_seconds: 0.0,
            marked_known: false,
        }
    }

    /// The learner's typical seconds per operand.
    pub fn typical_seconds(&self) -> f64 {
        self.log_t_ewma.exp()
    }

    fn days_since(&self, now: u64) -> f64 {
        if self.last_practised == 0 {
            return 0.0;
        }
        now.saturating_sub(self.last_practised) as f64 / SECONDS_PER_DAY
    }
}

/// The mastery gate, as a pure predicate.
pub fn mastery_gate(skill: SkillId, st: &SkillState) -> bool {
    let len = skill.ladder().len();
    st.p_known >= MASTERY_P_KNOWN
        && st.n >= MASTERY_MIN_N
        && st.acc_ewma >= MASTERY_ACC
        && st.level + 2 >= len
        && st.typical_seconds() <= MASTERY_TIME_FACTOR * skill.benchmark_seconds()
}

fn budget_per_operand(level: &Level, typical: f64) -> Option<f64> {
    level
        .timing
        .multiplier()
        .map(|m| (m * typical).max(BUDGET_FLOOR_SECONDS))
}

// ---------------------------------------------------------------------------
// Outcome, plans, rng
// ---------------------------------------------------------------------------

/// What one recorded answer changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub stepped_up: bool,
    pub stepped_down: bool,
    pub newly_mastered: bool,
}

/// One drill inside a session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrillSpec {
    pub skill: SkillId,
    pub level: Level,
    pub items: u8,
    /// Other skills to interleave (about 30 % of items), each at its own
    /// current step.
    pub interleave: Vec<SkillId>,
    pub target_success: f64,
}

/// Warm-up, focus, cool-down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionPlan {
    pub warmup: Option<DrillSpec>,
    pub focus: DrillSpec,
    pub cooldown: Option<DrillSpec>,
}

/// xorshift64* generator: tiny, deterministic, good enough for drill choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng(pub u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `0..n` (`0` when `n == 0`).
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            (self.next_u64() % u64::from(n)) as u32
        }
    }
}

impl Default for Rng {
    fn default() -> Self {
        Rng::new(0)
    }
}

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

/// Per-skill state for the whole curriculum plus the decision rules.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scheduler {
    skills: BTreeMap<SkillId, SkillState>,
}

impl Default for Scheduler {
    fn default() -> Self {
        let mut s = Scheduler {
            skills: BTreeMap::new(),
        };
        s.fill_missing();
        s
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add default state for any skill that is missing (after loading a
    /// save written by an older curriculum).
    pub fn fill_missing(&mut self) {
        for skill in SkillId::ALL {
            self.skills
                .entry(skill)
                .or_insert_with(|| SkillState::new(skill));
        }
    }

    pub fn state(&self, skill: SkillId) -> SkillState {
        self.skills
            .get(&skill)
            .copied()
            .unwrap_or_else(|| SkillState::new(skill))
    }

    pub fn set_state(&mut self, skill: SkillId, state: SkillState) {
        self.skills.insert(skill, state);
    }

    pub fn states(&self) -> impl Iterator<Item = (SkillId, &SkillState)> {
        self.skills.iter().map(|(k, v)| (*k, v))
    }

    pub fn current_level(&self, skill: SkillId) -> Level {
        let ladder = skill.ladder();
        ladder[self.state(skill).level.min(ladder.len() - 1)]
    }

    /// Every prerequisite is known well enough (`p_known` at or above
    /// [`UNLOCK_THRESHOLD`]). A recommendation only: every skill can be
    /// opened, drilled and recorded whatever this says.
    pub fn prerequisites_met(&self, skill: SkillId) -> bool {
        self.weak_prerequisites(skill).is_empty()
    }

    /// Direct prerequisites that are not yet known well enough, weakest
    /// first (ties keep teaching order).
    pub fn weak_prerequisites(&self, skill: SkillId) -> Vec<SkillId> {
        let mut weak: Vec<SkillId> = skill
            .prerequisites()
            .iter()
            .copied()
            .filter(|p| self.state(*p).p_known < UNLOCK_THRESHOLD)
            .collect();
        weak.sort_by(|a, b| self.state(*a).p_known.total_cmp(&self.state(*b).p_known));
        weak
    }

    /// The learner has met this skill: practised it or marked it as known.
    pub fn is_started(&self, skill: SkillId) -> bool {
        let st = self.state(skill);
        st.n > 0 || st.marked_known
    }

    /// "I know this already": seed the estimate to [`UNLOCK_THRESHOLD`] (never
    /// to 1.0, and never lower than it already is) so later skills count it
    /// as satisfied while the planner still checks it with a few drills.
    /// Everything else the skill has earned is kept.
    pub fn mark_known(&mut self, skill: SkillId) {
        let mut st = self.state(skill);
        if st.p_known < UNLOCK_THRESHOLD {
            st.p_known = UNLOCK_THRESHOLD;
        }
        let theta = (UNLOCK_THRESHOLD / (1.0 - UNLOCK_THRESHOLD)).ln();
        if st.theta < theta {
            st.theta = theta;
        }
        st.marked_known = true;
        self.skills.insert(skill, st);
    }

    /// "Reset this skill": back to a fresh state, as if never met. Undoes
    /// [`Scheduler::mark_known`]; history (the stats log) is not touched.
    pub fn reset_skill(&mut self, skill: SkillId) {
        self.skills.insert(skill, SkillState::new(skill));
    }

    pub fn is_mastered(&self, skill: SkillId) -> bool {
        self.state(skill).mastered_at.is_some()
    }

    /// `p_known` decayed by time since the last practice.
    pub fn p_eff(&self, skill: SkillId, now: u64) -> f64 {
        let st = self.state(skill);
        if st.last_practised == 0 {
            return st.p_known;
        }
        let tau = FORGET_TAU_DAYS * st.review_interval_days.max(f64::EPSILON);
        st.p_known * (-st.days_since(now) / tau).exp()
    }

    /// A mastered skill is due when its effective probability has decayed
    /// or its review interval has elapsed.
    pub fn is_due(&self, skill: SkillId, now: u64) -> bool {
        let st = self.state(skill);
        st.mastered_at.is_some()
            && (self.p_eff(skill, now) < REVIEW_DUE_P
                || st.days_since(now) >= st.review_interval_days)
    }

    /// Seconds allowed for one item at the current step; `None` when the
    /// step is untimed.
    pub fn time_budget_seconds(&self, skill: SkillId) -> Option<f64> {
        let st = self.state(skill);
        let level = self.current_level(skill);
        level.timing.multiplier()?;
        let per_operand = if st.budget_seconds > 0.0 {
            st.budget_seconds
        } else {
            budget_per_operand(&level, st.typical_seconds())?
        };
        Some(per_operand * operands_of(&level))
    }

    /// Record one answered item. `seconds` is the time for the whole item.
    pub fn record(&mut self, skill: SkillId, correct: bool, seconds: f64, now: u64) -> Outcome {
        let ladder = skill.ladder();
        let was_due = self.is_due(skill, now);
        let mut st = self.state(skill);
        let mut out = Outcome::default();

        let level_idx = st.level.min(ladder.len() - 1);
        let level = ladder[level_idx];
        let d = item_difficulty(skill, &level);
        let p = p_correct(st.theta, d);
        let c = if correct { 1.0 } else { 0.0 };

        st.theta += elo_u(st.n) * (c - p);
        st.p_known = bkt_update(st.p_known, correct);
        st.acc_ewma = ewma(st.acc_ewma, c, ACC_ALPHA);
        if correct && seconds > 0.0 {
            let per_operand = seconds / operands_of(&level);
            st.log_t_ewma = ewma(st.log_t_ewma, per_operand.ln(), TIME_ALPHA);
        }
        st.n += 1;
        st.items_at_level += 1;
        st.consecutive_errors = if correct {
            0
        } else {
            st.consecutive_errors.saturating_add(1)
        };
        st.last_practised = now;
        st.level = level_idx;

        // Step down / step up.
        let step_down = level_idx > 0
            && (st.consecutive_errors >= STEP_DOWN_CONSECUTIVE_ERRORS
                || (st.acc_ewma < STEP_DOWN_ACC && st.items_at_level >= STEP_DOWN_MIN_ITEMS));
        if step_down {
            Self::enter_level(&mut st, ladder, level_idx - 1);
            out.stepped_down = true;
        } else if level_idx + 1 < ladder.len()
            && Self::may_step_up(&st, &level, &ladder[level_idx + 1], d)
        {
            Self::enter_level(&mut st, ladder, level_idx + 1);
            out.stepped_up = true;
        }

        // Mastery gate and review schedule.
        if st.mastered_at.is_none() {
            if mastery_gate(skill, &st) {
                st.mastered_at = Some(now);
                st.review_interval_days = 1.0;
                out.newly_mastered = true;
            }
        } else if st.acc_ewma < REVIEW_RESET_ACC {
            st.mastered_at = None;
            st.review_interval_days = 1.0;
        } else if was_due && st.acc_ewma >= REVIEW_GROW_ACC {
            st.review_interval_days = (st.review_interval_days * REVIEW_GROW).min(REVIEW_MAX_DAYS);
        }

        self.skills.insert(skill, st);
        out
    }

    fn may_step_up(st: &SkillState, current: &Level, next: &Level, d_current: f64) -> bool {
        if st.acc_ewma < STEP_UP_ACC
            || st.items_at_level < STEP_UP_MIN_ITEMS
            || p_correct(st.theta, d_current) < STEP_UP_P
        {
            return false;
        }
        let typical = st.typical_seconds();
        if let Some(budget) = budget_per_operand(current, typical) {
            let anchored = if st.budget_seconds > 0.0 {
                st.budget_seconds
            } else {
                budget
            };
            if typical > STEP_UP_TIME_FACTOR * anchored {
                return false;
            }
        } else if next.timing != Timing::Untimed && st.acc_ewma < TIMER_INTRO_ACC {
            // Never introduce a timer below 0.90 accuracy at the untimed step.
            return false;
        }
        true
    }

    fn enter_level(st: &mut SkillState, ladder: &[Level], idx: usize) {
        st.level = idx;
        st.items_at_level = 0;
        st.consecutive_errors = 0;
        st.budget_seconds = budget_per_operand(&ladder[idx], st.typical_seconds()).unwrap_or(0.0);
    }

    /// The suggested next lesson: the first skill in teaching order whose
    /// prerequisites are met and which the learner has neither practised nor
    /// marked as known. Where "Continue" takes the learner; any other lesson
    /// can still be opened.
    pub fn next_skill_to_learn(&self) -> Option<SkillId> {
        SkillId::ALL
            .into_iter()
            .find(|s| self.prerequisites_met(*s) && !self.is_started(*s))
    }

    /// Cold-start placement: each skill probed with at least two items
    /// gets `p_known = clamp(fraction, 0.1, 0.9)` and the matching `theta`.
    pub fn placement(&mut self, results: &[(SkillId, u32, u32)]) {
        for &(skill, correct, total) in results {
            if total < 2 {
                continue;
            }
            let p = (f64::from(correct) / f64::from(total)).clamp(0.1, 0.9);
            let mut st = self.state(skill);
            st.p_known = p;
            st.theta = (p / (1.0 - p)).ln();
            self.skills.insert(skill, st);
        }
    }

    /// Skills the planner may use: those whose prerequisites are met (the
    /// suggested path) plus every skill the learner has already met, in
    /// whatever order they took them.
    fn plannable(&self) -> Vec<SkillId> {
        SkillId::ALL
            .into_iter()
            .filter(|s| self.prerequisites_met(*s) || self.is_started(*s))
            .collect()
    }

    /// Build the three-drill plan from the plannable skills. It
    /// always returns a plan: a focus skill with weak prerequisites still
    /// runs, and the warm-up becomes a light review of its weakest
    /// prerequisite when no review is due.
    pub fn plan_session(&self, now: u64, rng: &mut Rng) -> SessionPlan {
        let pool = self.plannable();
        if pool.len() <= 1 {
            return SessionPlan {
                warmup: None,
                focus: DrillSpec {
                    skill: SkillId::Setting,
                    level: self.current_level(SkillId::Setting),
                    items: 12,
                    interleave: Vec::new(),
                    target_success: 0.80,
                },
                cooldown: None,
            };
        }

        // Warm-up candidates: the due review with the longest gap, else the
        // most recently mastered skill at its top step. Chosen after the
        // focus, because a weak prerequisite of the focus can take the slot.
        let mastered: Vec<SkillId> = pool
            .iter()
            .copied()
            .filter(|s| self.is_mastered(*s))
            .collect();
        let due = mastered
            .iter()
            .copied()
            .filter(|s| self.is_due(*s, now))
            .max_by(|a, b| {
                self.state(*a)
                    .days_since(now)
                    .total_cmp(&self.state(*b).days_since(now))
            });
        let recent = mastered
            .iter()
            .copied()
            .max_by_key(|s| self.state(*s).mastered_at.unwrap_or(0));

        // Focus: lowest p_eff among plannable, unmastered skills (0.7), or
        // the second lowest (0.3).
        let mut candidates: Vec<SkillId> = pool
            .iter()
            .copied()
            .filter(|s| !self.is_mastered(*s))
            .collect();
        if candidates.is_empty() {
            candidates = pool.clone();
        }
        candidates.sort_by(|a, b| self.p_eff(*a, now).total_cmp(&self.p_eff(*b, now)));
        let idx = if candidates.len() > 1 && rng.next_f64() >= 0.7 {
            1
        } else {
            0
        };
        let focus_skill = candidates[idx];
        let interleave: Vec<SkillId> = pool
            .iter()
            .copied()
            .filter(|s| *s != focus_skill && self.is_started(*s))
            .collect();

        // Warm-up: a due review comes first. Otherwise, when the focus skill
        // was taken out of order, a light untimed look at its weakest
        // prerequisite; otherwise the most recently mastered skill.
        let warmup = if let Some(skill) = due {
            Some(DrillSpec {
                skill,
                level: self.current_level(skill),
                items: 7,
                interleave: Vec::new(),
                target_success: 0.90,
            })
        } else if let Some(&weak) = self.weak_prerequisites(focus_skill).first() {
            let mut level = self.current_level(weak);
            level.timing = Timing::Untimed;
            Some(DrillSpec {
                skill: weak,
                level,
                items: PREREQ_REVIEW_ITEMS,
                interleave: Vec::new(),
                target_success: 0.85,
            })
        } else {
            recent.map(|skill| {
                let ladder = skill.ladder();
                DrillSpec {
                    skill,
                    level: ladder[ladder.len() - 1],
                    items: 7,
                    interleave: Vec::new(),
                    target_success: 0.90,
                }
            })
        };
        let focus = DrillSpec {
            skill: focus_skill,
            level: self.current_level(focus_skill),
            items: 12,
            interleave,
            target_success: 0.80,
        };

        // Cool-down: untimed mix of everything the learner knows well.
        let mut strong: Vec<SkillId> = pool
            .iter()
            .copied()
            .filter(|s| self.is_started(*s) && self.p_eff(*s, now) >= 0.8)
            .collect();
        strong.sort_by(|a, b| self.p_eff(*b, now).total_cmp(&self.p_eff(*a, now)));
        let cooldown = strong.first().map(|&skill| {
            let mut level = self.current_level(skill);
            level.timing = Timing::Untimed;
            DrillSpec {
                skill,
                level,
                items: 6,
                interleave: strong[1..].to_vec(),
                target_success: 0.90,
            }
        });

        SessionPlan {
            warmup,
            focus,
            cooldown,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = 86_400;

    fn close(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn bkt_worked_sequence() {
        let mut l = BKT_P_L0;
        let mut seq = Vec::new();
        for _ in 0..4 {
            l = bkt_update(l, true);
            seq.push(l);
        }
        assert!(close(seq[0], 0.400, 5e-4), "{}", seq[0]);
        assert!(close(seq[1], 0.775, 5e-4), "{}", seq[1]);
        assert!(close(seq[2], 0.9455, 5e-4), "{}", seq[2]);
        assert!(close(seq[3], 0.9886, 5e-4), "{}", seq[3]);
        // With p_S = 0.1 and p_G = 0.2 the next two steps give 0.924 and
        // 0.984.
        l = bkt_update(l, false);
        assert!(close(l, 0.924, 5e-4), "{l}");
        l = bkt_update(l, true);
        assert!(close(l, 0.984, 5e-4), "{l}");
    }

    #[test]
    fn ewma_sequences() {
        let mut acc = 0.5;
        for _ in 0..10 {
            acc = ewma(acc, 1.0, ACC_ALPHA);
        }
        assert!(close(acc, 0.9463, 5e-5), "{acc}");
        let mut acc = 0.9;
        for _ in 0..3 {
            acc = ewma(acc, 0.0, ACC_ALPHA);
        }
        assert!(close(acc, 0.4608, 5e-5), "{acc}");
    }

    #[test]
    fn elo_uncertainty() {
        assert!(close(elo_u(0), 1.0, 1e-12));
        assert!(close(elo_u(10), 0.625, 1e-12));
        assert!(close(elo_u(100), 0.142857, 1e-6));
    }

    #[test]
    fn elo_first_answer_and_monotonicity() {
        assert!(close(p_correct(0.0, 0.0), 0.5, 1e-12));
        let theta = 0.0 + elo_u(0) * (1.0 - p_correct(0.0, 0.0));
        assert!(close(theta, 0.5, 1e-12));

        let mut rng = Rng::new(7);
        for _ in 0..500 {
            let skill = SkillId::ALL[rng.below(15) as usize];
            let mut s = Scheduler::default();
            let mut st = SkillState::new(skill);
            st.theta = rng.next_f64() * 8.0 - 4.0;
            st.n = rng.below(200);
            st.level = rng.below(skill.ladder().len() as u32) as usize;
            s.set_state(skill, st);
            let before = s.state(skill).theta;
            let correct = rng.next_f64() < 0.5;
            s.record(skill, correct, 2.0, 1_000);
            let after = s.state(skill).theta;
            if correct {
                assert!(after >= before);
            } else {
                assert!(after <= before);
            }
        }
    }

    #[test]
    fn item_difficulty_formula() {
        let l = Level {
            digits: 3,
            operands: 5,
            timing: Timing::Gentle,
            flash_ms: 0,
        };
        // 1.5 + 0.8 + 0.75 + 0.5
        assert!(close(item_difficulty(SkillId::MultiAdd, &l), 3.55, 1e-12));
        let f = Level {
            digits: 1,
            operands: 1,
            timing: Timing::Untimed,
            flash_ms: 800,
        };
        assert!(close(item_difficulty(SkillId::FlashOne, &f), 2.8, 1e-12));
    }

    fn ten_for_ten(level: usize, seconds: f64) -> (Scheduler, SkillState) {
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.level = level;
        s.set_state(skill, st);
        for i in 0..10 {
            s.record(skill, true, seconds, 1_000 + i);
        }
        let st = s.state(skill);
        (s, st)
    }

    #[test]
    fn mastery_gate_rejects_four_for_four() {
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.level = skill.ladder().len() - 2;
        s.set_state(skill, st);
        for i in 0..4 {
            let out = s.record(skill, true, 2.0, 1_000 + i);
            assert!(!out.newly_mastered);
        }
        let st = s.state(skill);
        assert_eq!(st.n, 4);
        assert!(st.p_known > 0.95);
        assert!(!mastery_gate(skill, &st));
        assert!(!s.is_mastered(skill));
    }

    #[test]
    fn mastery_gate_accepts_ten_for_ten_when_fluent() {
        let len = SkillId::AddSimple.ladder().len();
        let (s, st) = ten_for_ten(len - 2, 2.0);
        assert!(st.level + 2 >= len);
        assert!(st.typical_seconds() <= 1.5 * SkillId::AddSimple.benchmark_seconds());
        assert!(mastery_gate(SkillId::AddSimple, &st));
        assert!(s.is_mastered(SkillId::AddSimple));
    }

    #[test]
    fn mastery_gate_rejects_slow_ten_for_ten() {
        let len = SkillId::AddSimple.ladder().len();
        // 8 operands at 60 s per item = 7.5 s per operand > 1.5 x 2.5.
        let (s, st) = ten_for_ten(len - 2, 60.0);
        assert!(st.p_known > 0.95 && st.n >= 10 && st.acc_ewma >= 0.9);
        assert!(st.typical_seconds() > 1.5 * SkillId::AddSimple.benchmark_seconds());
        assert!(!mastery_gate(SkillId::AddSimple, &st));
        assert!(!s.is_mastered(SkillId::AddSimple));
    }

    #[test]
    fn mastery_gate_rejects_low_level() {
        let (_, st) = ten_for_ten(0, 2.0);
        assert!(st.level + 2 < SkillId::AddSimple.ladder().len());
        assert!(!mastery_gate(SkillId::AddSimple, &st));
    }

    #[test]
    fn step_down_on_two_consecutive_errors() {
        let skill = SkillId::AddFive;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.level = 2;
        st.acc_ewma = 0.9;
        s.set_state(skill, st);
        let out = s.record(skill, false, 3.0, 1_000);
        assert!(!out.stepped_down);
        assert_eq!(s.state(skill).level, 2);
        let out = s.record(skill, false, 3.0, 1_001);
        assert!(out.stepped_down);
        assert_eq!(s.state(skill).level, 1);
        assert_eq!(s.state(skill).consecutive_errors, 0);
        assert_eq!(s.state(skill).items_at_level, 0);
    }

    #[test]
    fn step_down_never_below_zero() {
        let skill = SkillId::Setting;
        let mut s = Scheduler::default();
        for i in 0..6 {
            s.record(skill, false, 3.0, 1_000 + i);
        }
        assert_eq!(s.state(skill).level, 0);
    }

    #[test]
    fn step_up_blocked_by_time_then_allowed() {
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.level = 3; // (1, 5, Gentle)
        st.acc_ewma = 0.95;
        st.theta = 5.0;
        st.n = 40;
        st.items_at_level = 10;
        st.budget_seconds = 1.2;
        st.log_t_ewma = 1.1f64.ln(); // 1.1 > 0.8 x 1.2
        s.set_state(skill, st);
        assert!(close(s.time_budget_seconds(skill).unwrap(), 6.0, 1e-9));

        let out = s.record(skill, true, 5.5, 1_000);
        assert!(!out.stepped_up);
        assert_eq!(s.state(skill).level, 3);

        let mut st = s.state(skill);
        st.log_t_ewma = 0.5f64.ln();
        s.set_state(skill, st);
        let out = s.record(skill, true, 2.5, 1_001);
        assert!(out.stepped_up);
        assert_eq!(s.state(skill).level, 4);
        assert_eq!(s.state(skill).items_at_level, 0);
        // Entering the next gentle step anchors a fresh budget.
        assert!(s.state(skill).budget_seconds >= BUDGET_FLOOR_SECONDS);
    }

    #[test]
    fn step_up_requires_eight_items_and_high_p() {
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.acc_ewma = 0.95;
        st.theta = 5.0;
        s.set_state(skill, st);
        for i in 0..7 {
            assert!(!s.record(skill, true, 2.0, 1_000 + i).stepped_up);
        }
        assert!(s.record(skill, true, 2.0, 1_010).stepped_up);
        assert_eq!(s.state(skill).level, 1);
    }

    #[test]
    fn timer_not_introduced_below_ninety_percent() {
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.level = 2; // last untimed step
        st.acc_ewma = 0.87;
        st.theta = 6.0;
        st.items_at_level = 20;
        s.set_state(skill, st);
        // One correct moves acc to 0.896: still below 0.90.
        assert!(!s.record(skill, true, 2.0, 1_000).stepped_up);
        assert!(s.record(skill, true, 2.0, 1_001).stepped_up);
        assert_eq!(s.current_level(skill).timing, Timing::Gentle);
    }

    #[test]
    fn untimed_budget_is_none_and_floor_applies() {
        let s = Scheduler::default();
        assert_eq!(s.time_budget_seconds(SkillId::AddSimple), None);
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.level = 5; // (1, 8, Tight)
        st.log_t_ewma = 0.3f64.ln();
        s.set_state(skill, st);
        // 1.3 x 0.3 = 0.39 < floor 1.2; 8 operands -> 9.6 s.
        assert!(close(s.time_budget_seconds(skill).unwrap(), 9.6, 1e-9));
    }

    #[test]
    fn forgetting_curve() {
        let skill = SkillId::AddTen;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.p_known = 0.96;
        st.last_practised = 1_000;
        st.mastered_at = Some(1_000);
        st.review_interval_days = 1.0;
        s.set_state(skill, st);
        let now = 1_000 + 14 * DAY;
        assert!(
            close(s.p_eff(skill, now), 0.353, 5e-4),
            "{}",
            s.p_eff(skill, now)
        );
        assert!(s.is_due(skill, now));

        st.review_interval_days = 10.0;
        s.set_state(skill, st);
        let p = s.p_eff(skill, now);
        assert!(close(p, 0.869, 5e-4), "{p}");
        assert!(p >= REVIEW_DUE_P);
        assert!(s.is_due(skill, now), "due by interval");
        assert!(!s.is_due(skill, 1_000 + 9 * DAY));

        // Never practised: no decay.
        st.last_practised = 0;
        s.set_state(skill, st);
        assert!(close(s.p_eff(skill, now), 0.96, 1e-12));
    }

    #[test]
    fn review_interval_expands_and_resets() {
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = SkillState::new(skill);
        st.p_known = 0.99;
        st.acc_ewma = 0.95;
        st.n = 50;
        st.level = 5;
        st.log_t_ewma = 0.5f64.ln();
        st.last_practised = 1_000;
        st.mastered_at = Some(1_000);
        s.set_state(skill, st);

        let now = 1_000 + 2 * DAY;
        assert!(s.is_due(skill, now));
        s.record(skill, true, 2.0, now);
        assert!(close(s.state(skill).review_interval_days, 1.6, 1e-12));
        // The next item of the same review is no longer due: no growth.
        s.record(skill, true, 2.0, now + 10);
        assert!(close(s.state(skill).review_interval_days, 1.6, 1e-12));

        // Accuracy collapsing below 0.7 clears mastery.
        for i in 0..3 {
            s.record(skill, false, 2.0, now + 20 + i);
        }
        assert!(!s.is_mastered(skill));
        assert!(close(s.state(skill).review_interval_days, 1.0, 1e-12));
    }

    #[test]
    fn readiness_follows_prerequisites() {
        let mut s = Scheduler::default();
        assert!(s.prerequisites_met(SkillId::Setting));
        assert!(!s.prerequisites_met(SkillId::AddSimple));
        let mut st = s.state(SkillId::Setting);
        st.p_known = 0.85;
        s.set_state(SkillId::Setting, st);
        assert!(s.prerequisites_met(SkillId::AddSimple));
        assert!(s.prerequisites_met(SkillId::SubSimple));
        assert!(!s.prerequisites_met(SkillId::AddFive));
        assert_eq!(s.next_skill_to_learn(), Some(SkillId::Setting));
        s.record(SkillId::Setting, true, 2.0, 1_000);
        assert_eq!(s.next_skill_to_learn(), Some(SkillId::AddSimple));
    }

    #[test]
    fn planner_picks_ready_or_started_skills() {
        let mut s = Scheduler::default();
        let mut st = s.state(SkillId::Setting);
        st.p_known = 0.9;
        st.n = 20;
        st.last_practised = 1_000;
        s.set_state(SkillId::Setting, st);
        let mut rng = Rng::new(42);
        for i in 0..300u64 {
            let now = 2_000 + i * 3_600;
            let plan = s.plan_session(now, &mut rng);
            let mut picked = vec![plan.focus.skill];
            picked.extend(plan.focus.interleave.iter().copied());
            for d in [&plan.warmup, &plan.cooldown].into_iter().flatten() {
                picked.push(d.skill);
                picked.extend(d.interleave.iter().copied());
            }
            for p in picked {
                assert!(
                    s.prerequisites_met(p) || s.is_started(p),
                    "{p:?} is neither ready nor started"
                );
            }
            assert!(matches!(
                plan.focus.skill,
                SkillId::AddSimple | SkillId::SubSimple
            ));
            // Only Setting has been met, so only Setting can be interleaved.
            assert_eq!(plan.focus.interleave, vec![SkillId::Setting]);
        }
    }

    #[test]
    fn planner_uses_both_lowest_candidates() {
        let mut s = Scheduler::default();
        let mut st = s.state(SkillId::Setting);
        st.p_known = 0.9;
        s.set_state(SkillId::Setting, st);
        let mut st = s.state(SkillId::SubSimple);
        st.p_known = 0.2;
        s.set_state(SkillId::SubSimple, st);
        let mut rng = Rng::new(3);
        let mut first = 0;
        let mut second = 0;
        for _ in 0..1_000 {
            match s.plan_session(5_000, &mut rng).focus.skill {
                SkillId::AddSimple => first += 1,
                SkillId::SubSimple => second += 1,
                other => panic!("{other:?}"),
            }
        }
        assert!((600..=800).contains(&first), "{first}");
        assert!((200..=400).contains(&second), "{second}");
    }

    #[test]
    fn setting_only_plan() {
        let s = Scheduler::default();
        let plan = s.plan_session(1_000, &mut Rng::new(1));
        assert_eq!(plan.focus.skill, SkillId::Setting);
        assert_eq!(plan.focus.level, SkillId::Setting.ladder()[0]);
        assert!(plan.focus.interleave.is_empty());
        assert!(plan.warmup.is_none());
        assert!(plan.cooldown.is_none());
    }

    #[test]
    fn planner_warmup_and_cooldown_use_mastered_skills() {
        let mut s = Scheduler::default();
        for skill in [SkillId::Setting, SkillId::AddSimple] {
            let mut st = SkillState::new(skill);
            st.p_known = 0.99;
            st.acc_ewma = 0.95;
            st.n = 60;
            st.level = skill.ladder().len() - 1;
            st.last_practised = 1_000;
            st.mastered_at = Some(1_000 + if skill == SkillId::AddSimple { 10 } else { 0 });
            s.set_state(skill, st);
        }
        let plan = s.plan_session(1_000 + DAY / 2, &mut Rng::new(9));
        let warm = plan.warmup.expect("warm-up");
        assert_eq!(warm.skill, SkillId::AddSimple);
        assert_eq!(warm.level.timing, Timing::Tight);
        assert!(matches!(
            plan.focus.skill,
            SkillId::SubSimple | SkillId::AddFive
        ));
        let cool = plan.cooldown.expect("cool-down");
        assert_eq!(cool.level.timing, Timing::Untimed);
        assert!(cool.interleave.len() + 1 == 2);

        // Two days later Setting (interval 1 day) is due and has the longer gap.
        let mut st = s.state(SkillId::Setting);
        st.last_practised = 500;
        s.set_state(SkillId::Setting, st);
        let plan = s.plan_session(1_000 + 2 * DAY, &mut Rng::new(9));
        assert_eq!(plan.warmup.unwrap().skill, SkillId::Setting);
    }

    #[test]
    fn placement_sets_p_and_theta() {
        let mut s = Scheduler::default();
        s.placement(&[
            (SkillId::AddSimple, 3, 3),
            (SkillId::SubSimple, 1, 4),
            (SkillId::AddFive, 1, 1),
        ]);
        let a = s.state(SkillId::AddSimple);
        assert!(close(a.p_known, 0.9, 1e-12));
        assert!(close(a.theta, (0.9f64 / 0.1).ln(), 1e-12));
        let b = s.state(SkillId::SubSimple);
        assert!(close(b.p_known, 0.25, 1e-12));
        assert!(close(b.theta, (0.25f64 / 0.75).ln(), 1e-12));
        assert_eq!(s.state(SkillId::AddFive), SkillState::new(SkillId::AddFive));
    }

    #[test]
    fn rng_is_deterministic_and_bounded() {
        let mut a = Rng::new(123);
        let mut b = Rng::new(123);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        for _ in 0..1_000 {
            let f = a.next_f64();
            assert!((0.0..1.0).contains(&f));
            assert!(a.below(7) < 7);
        }
        assert_eq!(a.below(0), 0);
        assert_ne!(Rng::new(0).next_u64(), 0);
    }

    #[test]
    fn simulated_learner_masters_add_simple() {
        let skill = SkillId::AddSimple;
        let mut s = Scheduler::default();
        let mut st = s.state(SkillId::Setting);
        st.p_known = 0.9;
        s.set_state(SkillId::Setting, st);
        assert!(s.prerequisites_met(skill));
        assert!(!s.prerequisites_met(SkillId::AddFive));

        let mut mastered_after = None;
        for i in 0..60u64 {
            // 95 % correct: every twentieth answer is wrong.
            let correct = i % 20 != 19;
            let out = s.record(skill, correct, 2.0, 10_000 + i * 30);
            if out.newly_mastered {
                mastered_after = Some(i + 1);
                break;
            }
        }
        let n = mastered_after.expect("mastered within 60 items");
        assert!(n <= 60, "{n}");
        assert!(s.is_mastered(skill));
        assert!(s.state(skill).level + 2 >= skill.ladder().len());
        assert!(s.prerequisites_met(SkillId::AddFive));
        assert_eq!(s.next_skill_to_learn(), Some(SkillId::Setting));
    }

    #[test]
    fn serde_round_trip_keeps_state() {
        let mut s = Scheduler::default();
        s.record(SkillId::Setting, true, 2.0, 1_000);
        let json = serde_json::to_string(&s).unwrap();
        let back: Scheduler = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    // --- Out-of-order learning (0.3.0) -----------------------------------

    #[test]
    fn every_skill_records_whatever_its_prerequisites() {
        // Division with nothing else known: drills still run and count.
        let mut s = Scheduler::default();
        assert!(!s.prerequisites_met(SkillId::Divide));
        assert_eq!(
            s.weak_prerequisites(SkillId::Divide),
            vec![SkillId::Multiply]
        );
        for i in 0..30u64 {
            s.record(SkillId::Divide, true, 20.0, 1_000 + i * 30);
        }
        let st = s.state(SkillId::Divide);
        assert_eq!(st.n, 30);
        assert!(st.p_known > 0.95);
        assert!(s.is_started(SkillId::Divide));
        // Its prerequisite is still weak and still the suggested path.
        assert!(!s.prerequisites_met(SkillId::Divide));
        assert_eq!(s.next_skill_to_learn(), Some(SkillId::Setting));
    }

    #[test]
    fn out_of_order_mastery_is_kept_and_counts_for_later_skills() {
        let mut s = Scheduler::default();
        // Learn S5 (big friends, adding) before S3 and S4.
        let skill = SkillId::AddTen;
        let mut mastered = false;
        for i in 0..120u64 {
            let out = s.record(skill, i % 20 != 19, 3.0, 10_000 + i * 30);
            mastered |= out.newly_mastered;
            if mastered {
                break;
            }
        }
        assert!(mastered, "S5 mastered out of order");
        assert!(s.is_mastered(skill));
        // S6 needs S4 and S5: S5 now counts, S4 is the only gap.
        assert_eq!(
            s.weak_prerequisites(SkillId::SubTen),
            vec![SkillId::SubFive]
        );
        // S5 is no longer suggested; the suggestion stays at the start.
        assert_eq!(s.next_skill_to_learn(), Some(SkillId::Setting));
    }

    #[test]
    fn mark_known_seeds_the_threshold_not_one() {
        let mut s = Scheduler::default();
        s.mark_known(SkillId::Setting);
        let st = s.state(SkillId::Setting);
        assert!(close(st.p_known, UNLOCK_THRESHOLD, 1e-12));
        assert!(st.p_known < 1.0);
        assert!(close(st.theta, (0.8f64 / 0.2).ln(), 1e-12));
        assert!(st.marked_known);
        assert_eq!(st.n, 0);
        assert!(st.mastered_at.is_none(), "known is not mastered");
        assert!(s.is_started(SkillId::Setting));
        // Its dependents now count as ready and the suggestion moves on.
        assert!(s.prerequisites_met(SkillId::AddSimple));
        assert_eq!(s.next_skill_to_learn(), Some(SkillId::AddSimple));
    }

    #[test]
    fn mark_known_never_lowers_an_earned_estimate() {
        let mut s = Scheduler::default();
        let mut st = s.state(SkillId::AddSimple);
        st.p_known = 0.97;
        st.theta = 3.0;
        st.n = 40;
        st.level = 4;
        s.set_state(SkillId::AddSimple, st);
        s.mark_known(SkillId::AddSimple);
        let after = s.state(SkillId::AddSimple);
        assert!(close(after.p_known, 0.97, 1e-12));
        assert!(close(after.theta, 3.0, 1e-12));
        assert_eq!(after.n, 40);
        assert_eq!(after.level, 4);
        assert!(after.marked_known);
    }

    #[test]
    fn reset_skill_undoes_mark_known_and_practice() {
        let mut s = Scheduler::default();
        s.mark_known(SkillId::Setting);
        s.record(SkillId::Setting, true, 2.0, 1_000);
        s.mark_known(SkillId::AddSimple);
        s.reset_skill(SkillId::Setting);
        assert_eq!(s.state(SkillId::Setting), SkillState::new(SkillId::Setting));
        assert!(!s.is_started(SkillId::Setting));
        assert!(!s.prerequisites_met(SkillId::AddSimple));
        // Other skills are untouched.
        assert!(s.state(SkillId::AddSimple).marked_known);
        assert_eq!(s.next_skill_to_learn(), Some(SkillId::Setting));
    }

    #[test]
    fn marked_known_skill_is_checked_by_the_planner() {
        let mut s = Scheduler::default();
        s.mark_known(SkillId::Setting);
        s.mark_known(SkillId::AddSimple);
        let mut seen = false;
        let mut rng = Rng::new(5);
        for _ in 0..50 {
            let plan = s.plan_session(5_000, &mut rng);
            // Never practised, but it appears: interleaved or in the cool-down.
            let mut skills: Vec<SkillId> = plan.focus.interleave.clone();
            skills.push(plan.focus.skill);
            if let Some(c) = &plan.cooldown {
                skills.push(c.skill);
                skills.extend(c.interleave.iter().copied());
            }
            seen |= skills.contains(&SkillId::AddSimple);
            assert!(plan.cooldown.is_some(), "known skills feed the cool-down");
        }
        assert!(seen);
    }

    #[test]
    fn planner_with_gaps_adds_a_light_prerequisite_review() {
        // The learner jumped to multiplication; S8 (its prerequisite) is weak.
        let mut s = Scheduler::default();
        for i in 0..6u64 {
            s.record(SkillId::Multiply, i % 2 == 0, 10.0, 1_000 + i * 30);
        }
        // Make Multiply the lowest estimate so it is the focus.
        let mut st = s.state(SkillId::Setting);
        st.p_known = 0.9;
        st.n = 5;
        st.last_practised = 1_000;
        s.set_state(SkillId::Setting, st);
        let mut st = s.state(SkillId::Multiply);
        st.p_known = 0.05;
        s.set_state(SkillId::Multiply, st);
        let mut rng = Rng::new(11);
        let mut multiply_focus = 0;
        for i in 0..200u64 {
            let plan = s.plan_session(2_000 + i * 600, &mut rng);
            if plan.focus.skill != SkillId::Multiply {
                continue;
            }
            multiply_focus += 1;
            assert_eq!(plan.focus.level, s.current_level(SkillId::Multiply));
            let warm = plan.warmup.expect("a light review of the gap");
            assert_eq!(warm.skill, SkillId::MultiAdd);
            assert_eq!(warm.items, PREREQ_REVIEW_ITEMS);
            assert_eq!(warm.level.timing, Timing::Untimed);
        }
        assert!(multiply_focus > 100, "{multiply_focus}");
    }

    #[test]
    fn due_review_keeps_the_warmup_slot_over_a_gap() {
        let mut s = Scheduler::default();
        let mut st = SkillState::new(SkillId::Setting);
        st.p_known = 0.99;
        st.acc_ewma = 0.95;
        st.n = 60;
        st.last_practised = 1_000;
        st.mastered_at = Some(1_000);
        s.set_state(SkillId::Setting, st);
        s.record(SkillId::Divide, false, 30.0, 1_000);
        let plan = s.plan_session(1_000 + 3 * DAY, &mut Rng::new(1));
        assert_eq!(plan.warmup.unwrap().skill, SkillId::Setting);
    }

    #[test]
    fn planner_never_panics_on_any_mix_of_known_and_started() {
        let mut rng = Rng::new(99);
        for round in 0..200u64 {
            let mut s = Scheduler::default();
            for skill in SkillId::ALL {
                match rng.below(4) {
                    0 => s.mark_known(skill),
                    1 => {
                        for i in 0..rng.below(15) {
                            s.record(skill, rng.below(3) > 0, 4.0, 1_000 + u64::from(i) * 20);
                        }
                    }
                    _ => {}
                }
            }
            let plan = s.plan_session(5_000 + round * DAY, &mut rng);
            for d in [&plan.warmup, &Some(plan.focus.clone()), &plan.cooldown]
                .into_iter()
                .flatten()
            {
                assert!(d.items > 0);
                assert!(d.skill.ladder().contains(&d.level) || d.level.timing == Timing::Untimed);
            }
        }
    }
}
