//! Practice history and the statistics the UI shows: momentum (not
//! streaks), daily totals, per-skill accuracy and speed curves, and a
//! session log. Pure data over a capped list of attempts.

use serde::{Deserialize, Serialize};

use crate::curriculum::SkillId;

/// Keep at most this many attempts; the oldest are dropped first.
pub const MAX_ATTEMPTS: usize = 50_000;

const SECONDS_PER_DAY: u64 = 86_400;
/// Momentum window.
const MOMENTUM_DAYS: u64 = 14;
/// Half-life of the "carried over" indicator.
const CARRIED_HALF_LIFE_SECONDS: f64 = 3.0 * 86_400.0;

/// The kind of drill an attempt belonged to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrillKind {
    Set,
    Read,
    AddSub,
    Multiply,
    Divide,
    Mixed,
    Sprint,
    Anzan,
    Flash,
    Lesson,
}

/// One answered item.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attempt {
    /// Unix seconds.
    pub ts: u64,
    pub skill: SkillId,
    /// Ladder step the item was drawn from.
    pub level: u8,
    pub kind: DrillKind,
    pub correct: bool,
    pub seconds: f32,
}

/// Totals for one UTC day.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DayTotal {
    pub day_start: u64,
    pub items: u32,
    pub correct: u32,
    pub minutes: f32,
}

/// The 14-day rolling picture.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Momentum {
    pub items_14d: u32,
    pub minutes_14d: f32,
    /// 0..1, halving every three days since the last attempt.
    pub carried: f32,
    pub days_active_14d: u8,
}

/// Per-skill summary.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillSummary {
    pub attempts: u32,
    pub accuracy: f32,
    /// Median seconds over correct attempts (0 when none).
    pub median_seconds: f32,
    pub last_ts: Option<u64>,
}

/// A run of attempts separated by less than a gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub start: u64,
    pub end: u64,
    pub items: u32,
    pub correct: u32,
}

/// All recorded attempts, oldest first as pushed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    attempts: Vec<Attempt>,
}

fn median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f32::total_cmp);
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        values[mid]
    } else {
        (values[mid - 1] + values[mid]) / 2.0
    }
}

fn day_start(ts: u64) -> u64 {
    ts - ts % SECONDS_PER_DAY
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, attempt: Attempt) {
        self.attempts.push(attempt);
        if self.attempts.len() > MAX_ATTEMPTS {
            let excess = self.attempts.len() - MAX_ATTEMPTS;
            self.attempts.drain(..excess);
        }
    }

    pub fn len(&self) -> usize {
        self.attempts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.attempts.is_empty()
    }

    pub fn attempts(&self) -> &[Attempt] {
        &self.attempts
    }

    /// Attempts at or after `ts`.
    pub fn since(&self, ts: u64) -> impl Iterator<Item = &Attempt> {
        self.attempts.iter().filter(move |a| a.ts >= ts)
    }

    fn for_skill(&self, skill: SkillId) -> impl Iterator<Item = &Attempt> {
        self.attempts.iter().filter(move |a| a.skill == skill)
    }

    /// Totals for the last `days` UTC days ending today, oldest first,
    /// with empty days present as zeros.
    pub fn daily_totals(&self, now: u64, days: usize) -> Vec<DayTotal> {
        let today = day_start(now);
        let mut out: Vec<DayTotal> = (0..days)
            .rev()
            .map(|i| DayTotal {
                day_start: today.saturating_sub(i as u64 * SECONDS_PER_DAY),
                items: 0,
                correct: 0,
                minutes: 0.0,
            })
            .collect();
        let Some(first) = out.first().map(|d| d.day_start) else {
            return out;
        };
        for a in &self.attempts {
            if a.ts < first || a.ts >= today + SECONDS_PER_DAY {
                continue;
            }
            let idx = ((day_start(a.ts) - first) / SECONDS_PER_DAY) as usize;
            if let Some(d) = out.get_mut(idx) {
                d.items += 1;
                d.correct += u32::from(a.correct);
                d.minutes += a.seconds / 60.0;
            }
        }
        out
    }

    /// Rolling 14-day totals plus the gently decaying "carried over" value.
    pub fn momentum(&self, now: u64) -> Momentum {
        let from = now.saturating_sub(MOMENTUM_DAYS * SECONDS_PER_DAY);
        let mut items = 0u32;
        let mut minutes = 0f32;
        let mut days: Vec<u64> = Vec::new();
        let mut last: Option<u64> = None;
        for a in &self.attempts {
            if a.ts <= now {
                last = Some(last.map_or(a.ts, |l| l.max(a.ts)));
            }
            if a.ts < from || a.ts > now {
                continue;
            }
            items += 1;
            minutes += a.seconds / 60.0;
            let d = day_start(a.ts);
            if !days.contains(&d) {
                days.push(d);
            }
        }
        let carried = match last {
            None => 0.0,
            Some(l) => {
                let dt = now.saturating_sub(l) as f64;
                0.5f64.powf(dt / CARRIED_HALF_LIFE_SECONDS) as f32
            }
        };
        Momentum {
            items_14d: items,
            minutes_14d: minutes,
            carried,
            days_active_14d: days.len().min(u8::MAX as usize) as u8,
        }
    }

    /// Fraction correct per bucket of `bucket` consecutive attempts on the
    /// skill (the last bucket may be partial).
    pub fn accuracy_series(&self, skill: SkillId, bucket: usize) -> Vec<f32> {
        let xs: Vec<bool> = self.for_skill(skill).map(|a| a.correct).collect();
        xs.chunks(bucket.max(1))
            .map(|c| c.iter().filter(|x| **x).count() as f32 / c.len() as f32)
            .collect()
    }

    /// Median seconds per bucket of `bucket` consecutive *correct* attempts
    /// on the skill (the last bucket may be partial).
    pub fn speed_series(&self, skill: SkillId, bucket: usize) -> Vec<f32> {
        let xs: Vec<f32> = self
            .for_skill(skill)
            .filter(|a| a.correct)
            .map(|a| a.seconds)
            .collect();
        xs.chunks(bucket.max(1))
            .map(|c| median(&mut c.to_vec()))
            .collect()
    }

    pub fn skill_summary(&self, skill: SkillId) -> SkillSummary {
        let mut attempts = 0u32;
        let mut correct = 0u32;
        let mut times = Vec::new();
        let mut last_ts = None;
        for a in self.for_skill(skill) {
            attempts += 1;
            if a.correct {
                correct += 1;
                times.push(a.seconds);
            }
            last_ts = Some(last_ts.map_or(a.ts, |l: u64| l.max(a.ts)));
        }
        SkillSummary {
            attempts,
            accuracy: if attempts == 0 {
                0.0
            } else {
                correct as f32 / attempts as f32
            },
            median_seconds: median(&mut times),
            last_ts,
        }
    }

    /// Group attempts (sorted by time) into sessions: a new session starts
    /// whenever the gap to the previous attempt is `gap_seconds` or more.
    pub fn sessions(&self, gap_seconds: u64) -> Vec<Session> {
        let mut sorted: Vec<&Attempt> = self.attempts.iter().collect();
        sorted.sort_by_key(|a| a.ts);
        let mut out: Vec<Session> = Vec::new();
        for a in sorted {
            match out.last_mut() {
                Some(s) if a.ts - s.end < gap_seconds => {
                    s.end = a.ts;
                    s.items += 1;
                    s.correct += u32::from(a.correct);
                }
                _ => out.push(Session {
                    start: a.ts,
                    end: a.ts,
                    items: 1,
                    correct: u32::from(a.correct),
                }),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = 86_400;

    fn at(ts: u64, skill: SkillId, correct: bool, seconds: f32) -> Attempt {
        Attempt {
            ts,
            skill,
            level: 0,
            kind: DrillKind::AddSub,
            correct,
            seconds,
        }
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn push_caps_and_drops_oldest() {
        let mut h = History::new();
        for i in 0..(MAX_ATTEMPTS as u64 + 5) {
            h.push(at(i, SkillId::Setting, true, 1.0));
        }
        assert_eq!(h.len(), MAX_ATTEMPTS);
        assert_eq!(h.attempts()[0].ts, 5);
        assert_eq!(h.since(MAX_ATTEMPTS as u64 + 2).count(), 3);
    }

    #[test]
    fn daily_totals_are_zero_filled_oldest_first() {
        let mut h = History::new();
        let today = 10 * DAY + 3_600; // 01:00 on day 10
        h.push(at(today, SkillId::AddSimple, true, 30.0));
        h.push(at(today + 60, SkillId::AddSimple, false, 30.0));
        h.push(at(8 * DAY + 10, SkillId::AddSimple, true, 120.0));
        h.push(at(5 * DAY, SkillId::AddSimple, true, 9.0)); // outside window
        h.push(at(11 * DAY, SkillId::AddSimple, true, 9.0)); // tomorrow
        let t = h.daily_totals(today, 3);
        assert_eq!(t.len(), 3);
        assert_eq!(t[0].day_start, 8 * DAY);
        assert_eq!((t[0].items, t[0].correct), (1, 1));
        assert!(close(t[0].minutes, 2.0));
        assert_eq!(t[1].day_start, 9 * DAY);
        assert_eq!(t[1].items, 0);
        assert_eq!(t[2].day_start, 10 * DAY);
        assert_eq!((t[2].items, t[2].correct), (2, 1));
        assert!(close(t[2].minutes, 1.0));
        assert!(h.daily_totals(today, 0).is_empty());
    }

    #[test]
    fn momentum_counts_window_and_decays() {
        let mut h = History::new();
        let now = 100 * DAY;
        h.push(at(now - 20 * DAY, SkillId::Setting, true, 60.0)); // too old
        h.push(at(now - 13 * DAY, SkillId::Setting, true, 60.0));
        h.push(at(now - 13 * DAY + 100, SkillId::Setting, false, 60.0));
        h.push(at(now - 3 * DAY, SkillId::Setting, true, 120.0));
        let m = h.momentum(now);
        assert_eq!(m.items_14d, 3);
        assert!(close(m.minutes_14d, 4.0));
        assert_eq!(m.days_active_14d, 2);
        assert!(close(m.carried, 0.5)); // exactly one half-life ago

        let m0 = h.momentum(now - 3 * DAY);
        assert!(close(m0.carried, 1.0));
        assert_eq!(History::new().momentum(now).carried, 0.0);
        let m6 = h.momentum(now + 3 * DAY);
        assert!(close(m6.carried, 0.25));
    }

    #[test]
    fn accuracy_and_speed_series() {
        let mut h = History::new();
        let pattern = [true, true, false, true, false, false, true];
        let times = [2.0, 4.0, 9.0, 6.0, 9.0, 9.0, 1.0];
        for (i, (c, t)) in pattern.iter().zip(times).enumerate() {
            h.push(at(i as u64, SkillId::AddFive, *c, t));
            h.push(at(i as u64, SkillId::SubFive, true, 100.0)); // other skill
        }
        let acc = h.accuracy_series(SkillId::AddFive, 3);
        assert_eq!(acc.len(), 3);
        assert!(close(acc[0], 2.0 / 3.0));
        assert!(close(acc[1], 1.0 / 3.0));
        assert!(close(acc[2], 1.0));
        // Correct times in order: 2, 4, 6, 1 -> buckets [2,4] [6,1].
        let sp = h.speed_series(SkillId::AddFive, 2);
        assert_eq!(sp, vec![3.0, 3.5]);
        assert!(h.speed_series(SkillId::Anzan, 2).is_empty());
        assert_eq!(h.accuracy_series(SkillId::AddFive, 0).len(), 7);
    }

    #[test]
    fn skill_summary_hand_computed() {
        let mut h = History::new();
        h.push(at(10, SkillId::AddTen, true, 5.0));
        h.push(at(30, SkillId::AddTen, false, 50.0));
        h.push(at(20, SkillId::AddTen, true, 3.0));
        h.push(at(20, SkillId::AddTen, true, 8.0));
        let s = h.skill_summary(SkillId::AddTen);
        assert_eq!(s.attempts, 4);
        assert!(close(s.accuracy, 0.75));
        assert!(close(s.median_seconds, 5.0));
        assert_eq!(s.last_ts, Some(30));
        let none = h.skill_summary(SkillId::Divide);
        assert_eq!(none.attempts, 0);
        assert_eq!(none.accuracy, 0.0);
        assert_eq!(none.median_seconds, 0.0);
        assert_eq!(none.last_ts, None);
    }

    #[test]
    fn sessions_group_by_gap() {
        let mut h = History::new();
        h.push(at(1_000, SkillId::Setting, true, 1.0));
        h.push(at(1_100, SkillId::Setting, false, 1.0));
        h.push(at(1_200, SkillId::Setting, true, 1.0));
        h.push(at(5_000, SkillId::Setting, true, 1.0)); // gap 3800 >= 1800
        h.push(at(6_799, SkillId::Setting, true, 1.0)); // gap 1799 < 1800
        h.push(at(8_599, SkillId::Setting, false, 1.0)); // gap 1800: new session
        let s = h.sessions(1_800);
        assert_eq!(
            s,
            vec![
                Session {
                    start: 1_000,
                    end: 1_200,
                    items: 3,
                    correct: 2
                },
                Session {
                    start: 5_000,
                    end: 6_799,
                    items: 2,
                    correct: 2
                },
                Session {
                    start: 8_599,
                    end: 8_599,
                    items: 1,
                    correct: 0
                },
            ]
        );
        assert!(History::new().sessions(1_800).is_empty());
    }

    #[test]
    fn serde_round_trip() {
        let mut h = History::new();
        h.push(at(1, SkillId::FlashOne, true, 0.5));
        let json = serde_json::to_string(&h).unwrap();
        let back: History = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
        let empty: History = serde_json::from_str("{}").unwrap();
        assert!(empty.is_empty());
    }
}
