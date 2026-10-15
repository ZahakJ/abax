//! Drill item generators. Every item carries exactly one focus skill and is
//! generated so that the bead moves it needs exercise that skill's rule and
//! nothing the learner has not met yet.
//!
//! Randomness comes from a caller-supplied `dice` closure returning a value
//! below its argument, so tests are deterministic and the core has no rand
//! dependency.

use crate::arith::{divide, multiply, rods_for_divide, rods_for_multiply, Procedure};
use crate::curriculum::{Level, SkillId};
use crate::moves::{
    add_number_at, plan_add_digit, plan_sub_digit, steps_to_show, sub_number_at, Rule, Step,
};
use crate::soroban::Soroban;
use serde::{Deserialize, Serialize};

/// "Random integer below n" source.
pub type Dice<'a> = &'a mut dyn FnMut(u64) -> u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Op {
    Add,
    Sub,
}

/// A drill item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// Set this number on the soroban.
    Set {
        target: u128,
    },
    /// Read the number shown on the soroban.
    Read {
        value: u128,
    },
    /// Start with `start`, then apply each operation in order.
    Chain {
        start: u128,
        ops: Vec<(Op, u128)>,
        answer: u128,
    },
    Multiply {
        a: u128,
        b: u128,
        answer: u128,
    },
    Divide {
        dividend: u128,
        divisor: u128,
        quotient: u128,
    },
    /// Numbers flashed one by one; the answer is their sum.
    Flash {
        numbers: Vec<u128>,
        interval_ms: u16,
        answer: u128,
    },
}

impl Item {
    pub fn answer(&self) -> u128 {
        match self {
            Item::Set { target } => *target,
            Item::Read { value } => *value,
            Item::Chain { answer, .. } => *answer,
            Item::Multiply { answer, .. } => *answer,
            Item::Divide { quotient, .. } => *quotient,
            Item::Flash { answer, .. } => *answer,
        }
    }

    /// Problem text in the form a learner reads it.
    pub fn prompt(&self) -> String {
        match self {
            Item::Set { target } => format!("Set {target}"),
            Item::Read { .. } => "Read the soroban".to_string(),
            Item::Chain { start, ops, .. } => {
                let mut s = start.to_string();
                for (op, n) in ops {
                    s.push_str(match op {
                        Op::Add => " + ",
                        Op::Sub => " − ",
                    });
                    s.push_str(&n.to_string());
                }
                s
            }
            Item::Multiply { a, b, .. } => format!("{a} × {b}"),
            Item::Divide {
                dividend, divisor, ..
            } => format!("{dividend} ÷ {divisor}"),
            Item::Flash { numbers, .. } => {
                let parts: Vec<String> = numbers.iter().map(|n| n.to_string()).collect();
                parts.join(" + ")
            }
        }
    }

    /// Number of operands (for timing budgets).
    pub fn operand_count(&self) -> usize {
        match self {
            Item::Set { .. } | Item::Read { .. } => 1,
            Item::Chain { ops, .. } => ops.len() + 1,
            Item::Multiply { .. } | Item::Divide { .. } => 2,
            Item::Flash { numbers, .. } => numbers.len(),
        }
    }

    /// Bead moves that solve the item on a soroban of `rod_count` rods,
    /// starting from an empty soroban. For multiplication and division the
    /// full narrated procedure is in [`Item::procedure`].
    pub fn solution_steps(&self, rod_count: usize) -> Vec<Step> {
        let mut s = Soroban::new(rod_count);
        let unit = rod_count - 1;
        match self {
            Item::Set { target } | Item::Read { value: target } => steps_to_show(&s, *target),
            Item::Chain { start, ops, .. } => {
                let mut out = add_number_at(&mut s, unit, *start).unwrap_or_default();
                for (op, n) in ops {
                    let steps = match op {
                        Op::Add => add_number_at(&mut s, unit, *n),
                        Op::Sub => sub_number_at(&mut s, unit, *n),
                    };
                    out.extend(steps.unwrap_or_default());
                }
                out
            }
            Item::Flash { numbers, .. } => {
                let mut out = Vec::new();
                for n in numbers {
                    out.extend(add_number_at(&mut s, unit, *n).unwrap_or_default());
                }
                out
            }
            Item::Multiply { .. } | Item::Divide { .. } => self
                .procedure(rod_count)
                .map(|p| p.steps.into_iter().flat_map(|s| s.beads).collect())
                .unwrap_or_default(),
        }
    }

    /// The narrated procedure for multiplication and division items.
    pub fn procedure(&self, rod_count: usize) -> Option<Procedure> {
        match self {
            Item::Multiply { a, b, .. } => multiply(*a, *b, rod_count).ok(),
            Item::Divide {
                dividend, divisor, ..
            } => divide(*dividend, *divisor, rod_count).ok(),
            _ => None,
        }
    }
}

/// The class of bead rules a chain needs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuleUse {
    pub direct: bool,
    pub five: bool,
    pub ten: bool,
    /// A ten-complement whose on-rod move needed a five-complement.
    pub combined: bool,
}

/// Classify the rules used to apply one single-digit operation on a rod
/// (the number already standing may be larger).
pub fn classify_digit_op(s: &Soroban, rod: usize, op: Op, d: u8) -> Option<RuleUse> {
    let steps = match op {
        Op::Add => plan_add_digit(s, rod, d),
        Op::Sub => plan_sub_digit(s, rod, d),
    }
    .ok()?;
    Some(classify_steps(&steps, rod))
}

fn classify_steps(steps: &[Step], rod: usize) -> RuleUse {
    let mut u = RuleUse::default();
    let mut ten_on_rod = false;
    let mut five_on_rod = false;
    for st in steps {
        match st.rule {
            Rule::Direct => u.direct = true,
            Rule::FiveComplement { .. } => {
                u.five = true;
                if st.rod == rod {
                    five_on_rod = true;
                }
            }
            Rule::TenComplement { .. } => {
                u.ten = true;
                if st.rod == rod {
                    ten_on_rod = true;
                }
            }
            Rule::Carry | Rule::Borrow => u.ten = true,
        }
    }
    // Combined: the move needed a carry/borrow and the on-rod half used the
    // small friend. (A carry/borrow is present whenever any Carry/Borrow or
    // TenComplement step exists; the five-complement on this rod then is the
    // combined case.)
    let has_carry = steps.iter().any(|s| {
        matches!(
            s.rule,
            Rule::Carry | Rule::Borrow | Rule::TenComplement { .. }
        )
    });
    if has_carry && five_on_rod {
        u.combined = true;
        // In the combined case the ten-complement rule is implied even when
        // every on-rod step was tagged as the five-complement.
        u.ten = true;
    }
    let _ = ten_on_rod;
    u
}

/// What a skill allows and what it must contain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Constraint {
    pub allow_five: bool,
    pub allow_ten: bool,
    pub allow_combined: bool,
    pub need_five: bool,
    pub need_ten: bool,
    pub need_combined: bool,
    pub ops: OpMix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpMix {
    AddOnly,
    SubOnly,
    Mixed,
}

pub fn constraint_for(skill: SkillId) -> Constraint {
    use SkillId::*;
    let c = |allow_five, allow_ten, allow_combined, need_five, need_ten, need_combined, ops| {
        Constraint {
            allow_five,
            allow_ten,
            allow_combined,
            need_five,
            need_ten,
            need_combined,
            ops,
        }
    };
    match skill {
        AddSimple => c(false, false, false, false, false, false, OpMix::AddOnly),
        SubSimple => c(false, false, false, false, false, false, OpMix::SubOnly),
        AddFive => c(true, false, false, true, false, false, OpMix::AddOnly),
        SubFive => c(true, false, false, true, false, false, OpMix::SubOnly),
        AddTen => c(true, true, false, false, true, false, OpMix::AddOnly),
        SubTen => c(true, true, false, false, true, false, OpMix::SubOnly),
        Combined => c(true, true, true, false, false, true, OpMix::Mixed),
        _ => c(true, true, true, false, false, false, OpMix::Mixed),
    }
}

fn pow10(d: u8) -> u128 {
    10u128.pow(d as u32)
}

/// A random number with exactly `digits` digits (no leading zero).
pub fn number_with_digits(digits: u8, dice: Dice) -> u128 {
    if digits <= 1 {
        return 1 + dice(9) as u128;
    }
    let lo = pow10(digits - 1);
    let hi = pow10(digits);
    lo + dice((hi - lo) as u64) as u128
}

/// Generate one item for `skill` at `level`.
pub fn generate(skill: SkillId, level: Level, rod_count: usize, dice: Dice) -> Item {
    use SkillId::*;
    match skill {
        Setting => {
            if dice(2) == 0 {
                Item::Set {
                    target: number_with_digits(level.digits, dice),
                }
            } else {
                Item::Read {
                    value: number_with_digits(level.digits, dice),
                }
            }
        }
        AddSimple | SubSimple | AddFive | SubFive | AddTen | SubTen | Combined => {
            single_digit_chain(constraint_for(skill), level.operands.max(2), dice)
        }
        MultiAdd => multi_digit_chain(level.digits, level.operands.max(2), OpMix::AddOnly, dice),
        MultiMixed => multi_digit_chain(level.digits, level.operands.max(2), OpMix::Mixed, dice),
        Anzan => {
            if level.digits <= 1 {
                single_digit_chain(constraint_for(Anzan), level.operands.max(2), dice)
            } else {
                multi_digit_chain(level.digits, level.operands.max(2), OpMix::Mixed, dice)
            }
        }
        Multiply => {
            let (mut k, mut m) = (level.digits.max(1), level.operands.max(1));
            while rods_for_multiply(k as usize, m as usize) > rod_count && (k > 1 || m > 1) {
                if k >= m {
                    k -= 1;
                } else {
                    m -= 1;
                }
            }
            let a = number_with_digits(k, dice);
            let b = number_with_digits(m, dice);
            Item::Multiply {
                a,
                b,
                answer: a * b,
            }
        }
        Divide => {
            let (mut k, m) = (level.digits.max(2), level.operands.max(1));
            while rods_for_divide(k as usize, m as usize) > rod_count && k > m + 1 {
                k -= 1;
            }
            // Exact division: pick divisor and quotient, derive the dividend.
            for _ in 0..1000 {
                let divisor = number_with_digits(m, dice).max(2);
                let q_digits = k.saturating_sub(m).max(1);
                let quotient = number_with_digits(q_digits, dice);
                let dividend = divisor * quotient;
                if dividend >= pow10(k - 1) && dividend < pow10(k) {
                    return Item::Divide {
                        dividend,
                        divisor,
                        quotient,
                    };
                }
            }
            Item::Divide {
                dividend: 84,
                divisor: 4,
                quotient: 21,
            }
        }
        FlashOne | FlashMulti => {
            let n = level.operands.max(2) as usize;
            let numbers: Vec<u128> = (0..n)
                .map(|_| number_with_digits(level.digits.max(1), dice))
                .collect();
            let answer = numbers.iter().sum();
            Item::Flash {
                numbers,
                interval_ms: level.flash_ms.max(200),
                answer,
            }
        }
    }
}

/// A chain of single-digit operations whose moves satisfy `c`. Uses
/// rejection sampling with a bounded number of attempts; the fallback is the
/// last candidate that was at least legal (never negative).
pub fn single_digit_chain(c: Constraint, operands: u8, dice: Dice) -> Item {
    let mut fallback: Option<Item> = None;
    for _ in 0..4000 {
        if let Some((item, ok)) = try_chain(c, operands, dice) {
            if ok {
                return item;
            }
            fallback.get_or_insert(item);
        }
    }
    fallback.unwrap_or(Item::Chain {
        start: 1,
        ops: vec![(Op::Add, 1)],
        answer: 2,
    })
}

/// Returns (item, satisfies all constraints) or None if the chain went
/// negative or was otherwise illegal.
///
/// Digits are chosen rod by rod from the set of digits whose bead moves the
/// constraint allows, so even "no complements" chains of eight operands are
/// found quickly; longer chains use two rods because a single rod cannot
/// absorb more than a few direct additions.
fn try_chain(c: Constraint, operands: u8, dice: Dice) -> Option<(Item, bool)> {
    let mut s = Soroban::new(6);
    let unit = s.unit_rod();
    // Skills without the ten-complement cannot absorb many operations on one
    // rod (at most 1+1+1+1+5 direct additions), so longer chains spread over
    // more rods; with carries allowed a single rod takes any chain.
    let width: usize = if c.allow_ten {
        1
    } else {
        (1 + (operands as usize - 1) / 3).min(3)
    };
    let start: u128 = match (c.ops, c.allow_ten) {
        (OpMix::SubOnly, true) => number_with_digits(width as u8 + 1, dice),
        (OpMix::SubOnly, false) => {
            // Each rod starts high so direct subtraction has room.
            let mut v = 0u128;
            for _ in 0..width {
                v = v * 10 + 6 + dice(4) as u128;
            }
            v
        }
        (_, true) => number_with_digits(width as u8, dice),
        (_, false) => {
            // Start low so the rods can absorb several direct additions.
            let mut v = 0u128;
            while v == 0 {
                v = 0;
                for _ in 0..width {
                    v = v * 10 + dice(3) as u128;
                }
            }
            v
        }
    };
    s.set_value(start);
    let mut ops = Vec::new();
    let mut used = RuleUse::default();
    let needs_unmet = |u: &RuleUse| -> bool {
        (c.need_five && !u.five) || (c.need_ten && !u.ten) || (c.need_combined && !u.combined)
    };
    for _ in 1..operands {
        let op = match c.ops {
            OpMix::AddOnly => Op::Add,
            OpMix::SubOnly => Op::Sub,
            OpMix::Mixed => {
                if s.value() < 10 || dice(2) == 0 {
                    Op::Add
                } else {
                    Op::Sub
                }
            }
        };
        let first_rod = unit + 1 - width;
        let mut tmp = s.clone();
        let mut n: u128 = 0;
        let mut step_use = RuleUse::default();
        for rod in first_rod..=unit {
            // Candidate digits for this rod under the constraint.
            let mut allowed: Vec<(u8, RuleUse)> = Vec::new();
            for d in 1..=9u8 {
                if let Some(class) = classify_digit_op(&tmp, rod, op, d) {
                    let ok = (!class.five || c.allow_five)
                        && (!class.ten || c.allow_ten)
                        && (!class.combined || c.allow_combined);
                    if ok {
                        allowed.push((d, class));
                    }
                }
            }
            let mut merged = used;
            merged.five |= step_use.five;
            merged.ten |= step_use.ten;
            merged.combined |= step_use.combined;
            let wanted: Vec<(u8, RuleUse)> = allowed
                .iter()
                .copied()
                .filter(|(_, cl)| {
                    (c.need_five && !merged.five && cl.five)
                        || (c.need_ten && !merged.ten && cl.ten)
                        || (c.need_combined && !merged.combined && cl.combined)
                })
                .collect();
            let pick = if !wanted.is_empty() && needs_unmet(&merged) && dice(10) < 7 {
                Some(wanted[dice(wanted.len() as u64) as usize])
            } else if allowed.is_empty() || (width > 1 && dice(6) == 0) {
                None // digit 0 on this rod
            } else if c.allow_ten {
                Some(allowed[dice(allowed.len() as u64) as usize])
            } else {
                // Without carries, favour small digits so the rod keeps room.
                let a = allowed[dice(allowed.len() as u64) as usize];
                let b = allowed[dice(allowed.len() as u64) as usize];
                Some(if a.0 <= b.0 { a } else { b })
            };
            let d = match pick {
                Some((d, class)) => {
                    step_use.direct |= class.direct;
                    step_use.five |= class.five;
                    step_use.ten |= class.ten;
                    step_use.combined |= class.combined;
                    match op {
                        Op::Add => crate::moves::add_digit(&mut tmp, rod, d).ok()?,
                        Op::Sub => crate::moves::sub_digit(&mut tmp, rod, d).ok()?,
                    };
                    d
                }
                None => 0,
            };
            n = n * 10 + d as u128;
        }
        if n == 0 {
            return None;
        }
        used.direct |= step_use.direct;
        used.five |= step_use.five;
        used.ten |= step_use.ten;
        used.combined |= step_use.combined;
        s = tmp;
        ops.push((op, n));
    }
    let all_ok = !needs_unmet(&used);
    let answer = s.value();
    Some((Item::Chain { start, ops, answer }, all_ok))
}

/// A chain of `operands` numbers of `digits` digits each; the running total
/// never goes negative.
pub fn multi_digit_chain(digits: u8, operands: u8, mix: OpMix, dice: Dice) -> Item {
    let digits = digits.max(1);
    let start = number_with_digits(digits, dice);
    let mut total = start;
    let mut ops = Vec::new();
    for _ in 1..operands {
        let n = number_with_digits(digits, dice);
        let op = match mix {
            OpMix::AddOnly => Op::Add,
            OpMix::SubOnly => Op::Sub,
            OpMix::Mixed => {
                if n <= total && dice(2) == 0 {
                    Op::Sub
                } else {
                    Op::Add
                }
            }
        };
        match op {
            Op::Add => total += n,
            Op::Sub => {
                if n > total {
                    // Cannot subtract: add instead.
                    total += n;
                    ops.push((Op::Add, n));
                    continue;
                }
                total -= n;
            }
        }
        ops.push((op, n));
    }
    Item::Chain {
        start,
        ops,
        answer: total,
    }
}

/// The exact rule classes a whole chain uses (for tests and the stats
/// "item class" key).
pub fn classify_chain(item: &Item) -> Option<RuleUse> {
    let Item::Chain { start, ops, .. } = item else {
        return None;
    };
    let mut s = Soroban::new(8);
    let unit = s.unit_rod();
    s.set_value(*start);
    let mut used = RuleUse::default();
    for (op, n) in ops {
        if *n > 9 {
            // Multi-digit: classify per digit from the left.
            let digits = crate::moves::decimal_digits(*n);
            let first = unit + 1 - digits.len();
            let mut tmp = s.clone();
            for (i, d) in digits.iter().enumerate() {
                if *d == 0 {
                    continue;
                }
                let class = classify_digit_op(&tmp, first + i, *op, *d)?;
                used.direct |= class.direct;
                used.five |= class.five;
                used.ten |= class.ten;
                used.combined |= class.combined;
                match op {
                    Op::Add => crate::moves::add_digit(&mut tmp, first + i, *d).ok()?,
                    Op::Sub => crate::moves::sub_digit(&mut tmp, first + i, *d).ok()?,
                };
            }
            s = tmp;
        } else {
            let class = classify_digit_op(&s, unit, *op, *n as u8)?;
            used.direct |= class.direct;
            used.five |= class.five;
            used.ten |= class.ten;
            used.combined |= class.combined;
            match op {
                Op::Add => add_number_at(&mut s, unit, *n).ok()?,
                Op::Sub => sub_number_at(&mut s, unit, *n).ok()?,
            };
        }
    }
    Some(used)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curriculum::Timing;

    struct X(u64);
    impl X {
        fn dice(&mut self) -> impl FnMut(u64) -> u64 + '_ {
            move |n| {
                self.0 ^= self.0 << 13;
                self.0 ^= self.0 >> 7;
                self.0 ^= self.0 << 17;
                self.0 % n.max(1)
            }
        }
    }

    fn level(digits: u8, operands: u8) -> Level {
        Level {
            digits,
            operands,
            timing: Timing::Untimed,
            flash_ms: 0,
        }
    }

    #[test]
    fn classify_examples() {
        let mut s = Soroban::new(2);
        s.set_digit(1, 1);
        let c = classify_digit_op(&s, 1, Op::Add, 3).unwrap();
        assert_eq!(
            c,
            RuleUse {
                direct: true,
                ..Default::default()
            }
        );
        let c = classify_digit_op(&s, 1, Op::Add, 4).unwrap();
        assert!(c.five && !c.ten && !c.combined);
        s.set_digit(1, 4);
        let c = classify_digit_op(&s, 1, Op::Add, 8).unwrap();
        assert!(c.ten && !c.five && !c.combined);
        s.set_digit(1, 7);
        let c = classify_digit_op(&s, 1, Op::Add, 6).unwrap();
        assert!(c.ten && c.five && c.combined);
        s.set_value(12);
        let c = classify_digit_op(&s, 1, Op::Sub, 7).unwrap();
        assert!(c.combined);
        s.set_value(11);
        let c = classify_digit_op(&s, 1, Op::Sub, 7).unwrap();
        assert!(c.ten && !c.combined);
    }

    #[test]
    fn single_digit_skills_generate_exactly_their_rules() {
        let mut x = X(42);
        let skills = [
            SkillId::AddSimple,
            SkillId::SubSimple,
            SkillId::AddFive,
            SkillId::SubFive,
            SkillId::AddTen,
            SkillId::SubTen,
            SkillId::Combined,
        ];
        for skill in skills {
            let c = constraint_for(skill);
            for operands in [2u8, 3, 5, 8] {
                for _ in 0..40 {
                    let item = generate(skill, level(1, operands), 13, &mut x.dice());
                    let Item::Chain { ops, answer, start } = &item else {
                        panic!("chain expected")
                    };
                    assert_eq!(ops.len() + 1, operands as usize, "{skill:?}");
                    let used = classify_chain(&item).expect("legal chain");
                    assert!(!used.five || c.allow_five, "{skill:?} {item:?}");
                    assert!(!used.ten || c.allow_ten, "{skill:?} {item:?}");
                    assert!(!used.combined || c.allow_combined, "{skill:?} {item:?}");
                    assert!(!c.need_five || used.five, "{skill:?} {item:?}");
                    assert!(!c.need_ten || used.ten, "{skill:?} {item:?}");
                    assert!(!c.need_combined || used.combined, "{skill:?} {item:?}");
                    match c.ops {
                        OpMix::AddOnly => assert!(ops.iter().all(|(o, _)| *o == Op::Add)),
                        OpMix::SubOnly => assert!(ops.iter().all(|(o, _)| *o == Op::Sub)),
                        OpMix::Mixed => {}
                    }
                    // Answer is right.
                    let mut v = *start as i128;
                    for (o, n) in ops {
                        match o {
                            Op::Add => v += *n as i128,
                            Op::Sub => v -= *n as i128,
                        }
                        assert!(v >= 0);
                    }
                    assert_eq!(v as u128, *answer);
                }
            }
        }
    }

    #[test]
    fn multi_digit_chains_have_right_digit_counts_and_answers() {
        let mut x = X(7);
        for digits in [2u8, 3] {
            for _ in 0..50 {
                let item = generate(SkillId::MultiMixed, level(digits, 5), 13, &mut x.dice());
                let Item::Chain { start, ops, answer } = &item else {
                    panic!()
                };
                assert_eq!(start.to_string().len(), digits as usize);
                let mut v = *start;
                for (o, n) in ops {
                    assert_eq!(n.to_string().len(), digits as usize);
                    match o {
                        Op::Add => v += n,
                        Op::Sub => v -= n,
                    }
                }
                assert_eq!(v, *answer);
                assert_eq!(ops.len(), 4);
            }
        }
    }

    #[test]
    fn setting_items_have_requested_digits() {
        let mut x = X(3);
        for _ in 0..30 {
            match generate(SkillId::Setting, level(3, 1), 13, &mut x.dice()) {
                Item::Set { target } | Item::Read { value: target } => {
                    assert_eq!(target.to_string().len(), 3)
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn multiply_and_divide_items_fit_the_rods_and_are_exact() {
        let mut x = X(99);
        for _ in 0..50 {
            let item = generate(SkillId::Multiply, level(3, 2), 13, &mut x.dice());
            let Item::Multiply { a, b, answer } = item.clone() else {
                panic!()
            };
            assert_eq!(a * b, answer);
            assert!(item.procedure(13).is_some(), "{item:?}");
            let item = generate(SkillId::Divide, level(4, 2), 13, &mut x.dice());
            let Item::Divide {
                dividend,
                divisor,
                quotient,
            } = item.clone()
            else {
                panic!()
            };
            assert_eq!(dividend, divisor * quotient);
            assert_eq!(dividend.to_string().len(), 4);
            assert!(item.procedure(13).is_some(), "{item:?}");
        }
        // Small soroban: operands shrink to fit.
        let item = generate(SkillId::Multiply, level(3, 2), 9, &mut x.dice());
        assert!(item.procedure(9).is_some());
    }

    #[test]
    fn flash_items_sum_their_numbers() {
        let mut x = X(5);
        let lv = Level {
            digits: 2,
            operands: 6,
            timing: Timing::Untimed,
            flash_ms: 600,
        };
        let item = generate(SkillId::FlashMulti, lv, 13, &mut x.dice());
        let Item::Flash {
            numbers,
            interval_ms,
            answer,
        } = &item
        else {
            panic!()
        };
        assert_eq!(numbers.len(), 6);
        assert_eq!(*interval_ms, 600);
        assert_eq!(numbers.iter().sum::<u128>(), *answer);
        assert!(numbers.iter().all(|n| (10..100).contains(n)));
    }

    #[test]
    fn solution_steps_reach_the_answer() {
        let mut x = X(11);
        for skill in [
            SkillId::Combined,
            SkillId::MultiMixed,
            SkillId::Setting,
            SkillId::FlashOne,
        ] {
            for _ in 0..20 {
                let lv = if skill == SkillId::FlashOne {
                    Level {
                        digits: 1,
                        operands: 5,
                        timing: Timing::Untimed,
                        flash_ms: 1000,
                    }
                } else {
                    level(2, 4)
                };
                let item = generate(skill, lv, 13, &mut x.dice());
                let mut s = Soroban::new(13);
                crate::moves::apply_steps(&mut s, &item.solution_steps(13));
                assert_eq!(s.value(), item.answer(), "{item:?}");
            }
        }
    }

    #[test]
    fn prompt_formatting() {
        let item = Item::Chain {
            start: 7,
            ops: vec![(Op::Add, 6), (Op::Sub, 4)],
            answer: 9,
        };
        assert_eq!(item.prompt(), "7 + 6 − 4");
        assert_eq!(item.operand_count(), 3);
        assert_eq!(
            Item::Multiply {
                a: 3,
                b: 4,
                answer: 12
            }
            .prompt(),
            "3 × 4"
        );
    }
}
