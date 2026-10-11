//! The bead-move engine. Given a soroban and "add digit `a` on rod `r`" (or
//! subtract), it produces the exact sequence of bead moves a soroban teacher
//! would make, including the five-complement ("small friends"), the
//! ten-complement ("big friends"), the combined case, and carry/borrow
//! propagation across rods.
//!
//! Conventions follow Kojima's six ordering rules
//! (research/soroban-mechanics-and-curriculum.md §2.2):
//!
//! 1. Adding 6..9 directly: heaven bead down and earth beads up together
//!    (emitted heaven first; the UI animates them as one pinch).
//! 2. Subtracting 6..9 directly: earth beads down first, then heaven up.
//! 3. Five-complement addition: heaven down, then earth down ("set 5, take 1").
//! 4. Five-complement subtraction: earth up, then heaven up ("set 1, take 5").
//! 5. Ten-complement addition: take the complement on this rod, then carry 1 left.
//! 6. Ten-complement subtraction: borrow 1 from the left, then set the complement.
//!
//! The carry/borrow order can be flipped through [`Convention`].

use crate::soroban::Soroban;
use serde::{Deserialize, Serialize};

/// Which finger performs a bead move in the standard two-finger technique.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finger {
    /// The thumb pushes earth beads up (adds 1..4).
    Thumb,
    /// The index finger pushes earth beads down and moves the heaven bead
    /// in both directions.
    Index,
}

/// A single physical bead action on one rod.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepKind {
    /// Push `n` earth beads up to the beam (+n).
    EarthUp(u8),
    /// Push `n` earth beads down, away from the beam (-n).
    EarthDown(u8),
    /// Push the heaven bead down to the beam (+5).
    HeavenDown,
    /// Push the heaven bead up, away from the beam (-5).
    HeavenUp,
}

impl StepKind {
    pub fn finger(self) -> Finger {
        match self {
            StepKind::EarthUp(_) => Finger::Thumb,
            StepKind::EarthDown(_) | StepKind::HeavenDown | StepKind::HeavenUp => Finger::Index,
        }
    }

    /// Change to the rod's digit.
    pub fn delta(self) -> i8 {
        match self {
            StepKind::EarthUp(n) => n as i8,
            StepKind::EarthDown(n) => -(n as i8),
            StepKind::HeavenDown => 5,
            StepKind::HeavenUp => -5,
        }
    }
}

/// Why a step is made. Used by lessons to explain the move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rule {
    /// Beads were available to move directly.
    Direct,
    /// Five-complement: `friend` is the number that makes 5 with the digit
    /// being added or subtracted (e.g. adding 4 uses friend 1: set 5, take 1).
    FiveComplement { friend: u8 },
    /// Ten-complement: `friend` makes 10 with the digit (e.g. adding 8 uses
    /// friend 2: take 2, carry 1).
    TenComplement { friend: u8 },
    /// The +1 on the next rod to the left that completes a ten-complement
    /// addition.
    Carry,
    /// The -1 on the next rod to the left that completes a ten-complement
    /// subtraction.
    Borrow,
}

/// One bead move with its rod and the reason for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Step {
    pub rod: usize,
    pub kind: StepKind,
    pub rule: Rule,
}

impl Step {
    pub const fn new(rod: usize, kind: StepKind, rule: Rule) -> Self {
        Self { rod, kind, rule }
    }

    pub fn finger(&self) -> Finger {
        self.kind.finger()
    }
}

/// Order of the two halves of a ten-complement move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Convention {
    /// Addition: carry to the left rod before taking the complement.
    pub carry_first: bool,
    /// Subtraction: set the complement before borrowing from the left rod.
    pub complement_before_borrow: bool,
}

impl Default for Convention {
    /// Japanese textbook order: complement then carry; borrow then complement.
    fn default() -> Self {
        Self {
            carry_first: false,
            complement_before_borrow: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveError {
    /// A carry ran off the left edge of the soroban.
    Overflow,
    /// A borrow was needed but no rod to the left holds anything.
    Underflow,
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::Overflow => write!(f, "the number does not fit on the soroban"),
            MoveError::Underflow => write!(f, "the result would be negative"),
        }
    }
}

impl std::error::Error for MoveError {}

/// Apply one step to the soroban. Panics in debug builds if the step is
/// physically impossible in the current state.
pub fn apply_step(s: &mut Soroban, step: &Step) {
    let rod = s.rod_mut(step.rod);
    match step.kind {
        StepKind::EarthUp(n) => {
            debug_assert!(rod.earth + n <= 4, "not enough earth beads to push up");
            rod.earth += n;
        }
        StepKind::EarthDown(n) => {
            debug_assert!(rod.earth >= n, "not enough earth beads to push down");
            rod.earth -= n;
        }
        StepKind::HeavenDown => {
            debug_assert!(!rod.heaven, "heaven bead is already down");
            rod.heaven = true;
        }
        StepKind::HeavenUp => {
            debug_assert!(rod.heaven, "heaven bead is already up");
            rod.heaven = false;
        }
    }
}

pub fn apply_steps(s: &mut Soroban, steps: &[Step]) {
    for st in steps {
        apply_step(s, st);
    }
}

/// Add digit `a` (1..=9) on rod `rod`, mutating `s` and returning the moves.
pub fn add_digit(s: &mut Soroban, rod: usize, a: u8) -> Result<Vec<Step>, MoveError> {
    add_digit_with(s, rod, a, Convention::default())
}

/// Subtract digit `a` (1..=9) on rod `rod`, mutating `s` and returning the moves.
pub fn sub_digit(s: &mut Soroban, rod: usize, a: u8) -> Result<Vec<Step>, MoveError> {
    sub_digit_with(s, rod, a, Convention::default())
}

/// Like [`add_digit`] but without mutating.
pub fn plan_add_digit(s: &Soroban, rod: usize, a: u8) -> Result<Vec<Step>, MoveError> {
    let mut c = s.clone();
    add_digit(&mut c, rod, a)
}

/// Like [`sub_digit`] but without mutating.
pub fn plan_sub_digit(s: &Soroban, rod: usize, a: u8) -> Result<Vec<Step>, MoveError> {
    let mut c = s.clone();
    sub_digit(&mut c, rod, a)
}

pub fn add_digit_with(
    s: &mut Soroban,
    rod: usize,
    a: u8,
    conv: Convention,
) -> Result<Vec<Step>, MoveError> {
    assert!((1..=9).contains(&a), "add a digit 1..=9");
    let mut out = Vec::new();
    add_digit_inner(s, rod, a, conv, &mut out)?;
    Ok(out)
}

pub fn sub_digit_with(
    s: &mut Soroban,
    rod: usize,
    a: u8,
    conv: Convention,
) -> Result<Vec<Step>, MoveError> {
    assert!((1..=9).contains(&a), "subtract a digit 1..=9");
    let mut out = Vec::new();
    sub_digit_inner(s, rod, a, conv, &mut out)?;
    Ok(out)
}

fn push(s: &mut Soroban, out: &mut Vec<Step>, step: Step) {
    apply_step(s, &step);
    out.push(step);
}

/// Direct moves only: add `a` on a rod where `digit + a <= 9` using the
/// five-complement when the earth beads run out. Returns false if the
/// result would exceed 9 (caller must use the ten-complement).
fn add_within_rod(s: &mut Soroban, rod: usize, a: u8, out: &mut Vec<Step>) -> bool {
    let r = s.rod(rod);
    let d = r.digit();
    if d + a > 9 {
        return false;
    }
    if a == 5 {
        // d + 5 <= 9 implies heaven is up.
        push(s, out, Step::new(rod, StepKind::HeavenDown, Rule::Direct));
    } else if a < 5 {
        if r.earth + a <= 4 {
            push(s, out, Step::new(rod, StepKind::EarthUp(a), Rule::Direct));
        } else {
            // Small friend: a = 5 - friend. Set 5, take friend.
            let friend = 5 - a;
            let rule = Rule::FiveComplement { friend };
            push(s, out, Step::new(rod, StepKind::HeavenDown, rule));
            push(s, out, Step::new(rod, StepKind::EarthDown(friend), rule));
        }
    } else {
        // a in 6..=9, d + a <= 9 implies heaven up and earth + (a-5) <= 4.
        push(s, out, Step::new(rod, StepKind::HeavenDown, Rule::Direct));
        push(
            s,
            out,
            Step::new(rod, StepKind::EarthUp(a - 5), Rule::Direct),
        );
    }
    true
}

/// Direct moves only: subtract `a` on a rod where `digit >= a`, using the
/// five-complement when needed. Returns false if digit < a.
fn sub_within_rod(s: &mut Soroban, rod: usize, a: u8, out: &mut Vec<Step>) -> bool {
    let r = s.rod(rod);
    let d = r.digit();
    if d < a {
        return false;
    }
    if a == 5 {
        push(s, out, Step::new(rod, StepKind::HeavenUp, Rule::Direct));
    } else if a < 5 {
        if r.earth >= a {
            push(s, out, Step::new(rod, StepKind::EarthDown(a), Rule::Direct));
        } else {
            // Small friend (Kojima rule 4): set friend, then take 5.
            let friend = 5 - a;
            let rule = Rule::FiveComplement { friend };
            push(s, out, Step::new(rod, StepKind::EarthUp(friend), rule));
            push(s, out, Step::new(rod, StepKind::HeavenUp, rule));
        }
    } else {
        // a in 6..=9 and d >= a implies heaven down and earth >= a - 5.
        // Kojima rule 2: earth beads first, then the heaven bead.
        push(
            s,
            out,
            Step::new(rod, StepKind::EarthDown(a - 5), Rule::Direct),
        );
        push(s, out, Step::new(rod, StepKind::HeavenUp, Rule::Direct));
    }
    true
}

/// Subtract `friend` (1..=9) on `rod` as the first half of a ten-complement
/// addition. The digit is known to be >= friend. Steps are tagged with the
/// ten-complement rule (or the five-complement when the earth beads do not
/// suffice: the "combined" case).
fn take_complement(s: &mut Soroban, rod: usize, friend: u8, out: &mut Vec<Step>) {
    let r = s.rod(rod);
    let ten = Rule::TenComplement { friend };
    if friend == 5 {
        push(s, out, Step::new(rod, StepKind::HeavenUp, ten));
    } else if friend < 5 {
        if r.earth >= friend {
            push(s, out, Step::new(rod, StepKind::EarthDown(friend), ten));
        } else {
            // Combined: cannot take `friend` directly; set 5 - friend, take 5.
            let small = Rule::FiveComplement { friend: 5 - friend };
            push(s, out, Step::new(rod, StepKind::EarthUp(5 - friend), small));
            push(s, out, Step::new(rod, StepKind::HeavenUp, small));
        }
    } else {
        push(s, out, Step::new(rod, StepKind::EarthDown(friend - 5), ten));
        push(s, out, Step::new(rod, StepKind::HeavenUp, ten));
    }
}

/// Add `friend` (1..=9) on `rod` as the second half of a ten-complement
/// subtraction. The digit is known to be < 10 - friend... i.e. it fits.
fn set_complement(s: &mut Soroban, rod: usize, friend: u8, out: &mut Vec<Step>) {
    let r = s.rod(rod);
    let ten = Rule::TenComplement { friend };
    if friend == 5 {
        push(s, out, Step::new(rod, StepKind::HeavenDown, ten));
    } else if friend < 5 {
        if r.earth + friend <= 4 {
            push(s, out, Step::new(rod, StepKind::EarthUp(friend), ten));
        } else {
            // Combined: set 5, take 5 - friend.
            let small = Rule::FiveComplement { friend: 5 - friend };
            push(s, out, Step::new(rod, StepKind::HeavenDown, small));
            push(
                s,
                out,
                Step::new(rod, StepKind::EarthDown(5 - friend), small),
            );
        }
    } else {
        push(s, out, Step::new(rod, StepKind::HeavenDown, ten));
        push(s, out, Step::new(rod, StepKind::EarthUp(friend - 5), ten));
    }
}

fn add_digit_inner(
    s: &mut Soroban,
    rod: usize,
    a: u8,
    conv: Convention,
    out: &mut Vec<Step>,
) -> Result<(), MoveError> {
    if add_within_rod(s, rod, a, out) {
        return Ok(());
    }
    // Ten-complement: digit + a >= 10.
    if rod == 0 {
        return Err(MoveError::Overflow);
    }
    let friend = 10 - a;
    if conv.carry_first {
        carry(s, rod - 1, conv, out)?;
        take_complement(s, rod, friend, out);
    } else {
        // Plan the carry first so an overflow leaves `s` untouched.
        let mut probe = s.clone();
        let mut scratch = Vec::new();
        carry(&mut probe, rod - 1, conv, &mut scratch)?;
        take_complement(s, rod, friend, out);
        carry(s, rod - 1, conv, out)?;
    }
    Ok(())
}

/// Add 1 to `rod` as a carry, tagging direct moves with [`Rule::Carry`].
fn carry(
    s: &mut Soroban,
    rod: usize,
    conv: Convention,
    out: &mut Vec<Step>,
) -> Result<(), MoveError> {
    let start = out.len();
    add_digit_inner(s, rod, 1, conv, out)?;
    for st in &mut out[start..] {
        if st.rule == Rule::Direct {
            st.rule = Rule::Carry;
        }
    }
    Ok(())
}

fn sub_digit_inner(
    s: &mut Soroban,
    rod: usize,
    a: u8,
    conv: Convention,
    out: &mut Vec<Step>,
) -> Result<(), MoveError> {
    if sub_within_rod(s, rod, a, out) {
        return Ok(());
    }
    // Ten-complement: digit < a, borrow 1 from the left.
    if rod == 0 {
        return Err(MoveError::Underflow);
    }
    let friend = 10 - a;
    if conv.complement_before_borrow {
        let mut probe = s.clone();
        let mut scratch = Vec::new();
        borrow(&mut probe, rod - 1, conv, &mut scratch)?;
        set_complement(s, rod, friend, out);
        borrow(s, rod - 1, conv, out)?;
    } else {
        borrow(s, rod - 1, conv, out)?;
        set_complement(s, rod, friend, out);
    }
    Ok(())
}

fn borrow(
    s: &mut Soroban,
    rod: usize,
    conv: Convention,
    out: &mut Vec<Step>,
) -> Result<(), MoveError> {
    let start = out.len();
    sub_digit_inner(s, rod, 1, conv, out)?;
    for st in &mut out[start..] {
        if st.rule == Rule::Direct {
            st.rule = Rule::Borrow;
        }
    }
    Ok(())
}

/// Add a whole number, units on `unit_rod`, working left to right as
/// taught. On error the soroban is left as it was.
pub fn add_number_at(s: &mut Soroban, unit_rod: usize, n: u128) -> Result<Vec<Step>, MoveError> {
    let snapshot = s.clone();
    let digits = decimal_digits(n);
    if digits.len() > unit_rod + 1 {
        return Err(MoveError::Overflow);
    }
    let mut out = Vec::new();
    let first_rod = unit_rod + 1 - digits.len();
    for (i, d) in digits.iter().enumerate() {
        if *d == 0 {
            continue;
        }
        match add_digit(s, first_rod + i, *d) {
            Ok(steps) => out.extend(steps),
            Err(e) => {
                *s = snapshot;
                return Err(e);
            }
        }
    }
    Ok(out)
}

/// Subtract a whole number, units on `unit_rod`, left to right.
pub fn sub_number_at(s: &mut Soroban, unit_rod: usize, n: u128) -> Result<Vec<Step>, MoveError> {
    let snapshot = s.clone();
    let digits = decimal_digits(n);
    if digits.len() > unit_rod + 1 {
        return Err(MoveError::Underflow);
    }
    let mut out = Vec::new();
    let first_rod = unit_rod + 1 - digits.len();
    for (i, d) in digits.iter().enumerate() {
        if *d == 0 {
            continue;
        }
        match sub_digit(s, first_rod + i, *d) {
            Ok(steps) => out.extend(steps),
            Err(e) => {
                *s = snapshot;
                return Err(e);
            }
        }
    }
    Ok(out)
}

/// Add `n` with units on the rightmost rod.
pub fn add_number(s: &mut Soroban, n: u128) -> Result<Vec<Step>, MoveError> {
    let u = s.unit_rod();
    add_number_at(s, u, n)
}

/// Subtract `n` with units on the rightmost rod.
pub fn sub_number(s: &mut Soroban, n: u128) -> Result<Vec<Step>, MoveError> {
    let u = s.unit_rod();
    sub_number_at(s, u, n)
}

/// The steps that take the soroban from its current state to showing `n`,
/// rod by rod (used for "set this number" demonstrations).
pub fn steps_to_show(s: &Soroban, n: u128) -> Vec<Step> {
    let mut c = s.clone();
    let mut target = Soroban::new(s.rod_count());
    target.set_value(n);
    let mut out = Vec::new();
    for rod in 0..s.rod_count() {
        out.extend(steps_to_set_rod(&mut c, rod, target.digit(rod)));
    }
    out
}

/// Steps that change one rod from its current digit to `d` directly
/// (no complements: this is "setting", not arithmetic).
pub fn steps_to_set_rod(s: &mut Soroban, rod: usize, d: u8) -> Vec<Step> {
    let r = s.rod(rod);
    let t = crate::soroban::Rod::from_digit(d);
    let mut out = Vec::new();
    if r.heaven != t.heaven {
        let kind = if t.heaven {
            StepKind::HeavenDown
        } else {
            StepKind::HeavenUp
        };
        push(s, &mut out, Step::new(rod, kind, Rule::Direct));
    }
    if r.earth < t.earth {
        push(
            s,
            &mut out,
            Step::new(rod, StepKind::EarthUp(t.earth - r.earth), Rule::Direct),
        );
    } else if r.earth > t.earth {
        push(
            s,
            &mut out,
            Step::new(rod, StepKind::EarthDown(r.earth - t.earth), Rule::Direct),
        );
    }
    out
}

/// Big-endian decimal digits of `n` (`[0]` for zero).
pub fn decimal_digits(mut n: u128) -> Vec<u8> {
    if n == 0 {
        return vec![0];
    }
    let mut v = Vec::new();
    while n > 0 {
        v.push((n % 10) as u8);
        n /= 10;
    }
    v.reverse();
    v
}

#[cfg(test)]
mod tests {
    use super::StepKind::*;
    use super::*;

    fn rod_with(d: u8) -> Soroban {
        let mut s = Soroban::new(2);
        s.set_digit(1, d);
        s
    }

    fn kinds(steps: &[Step]) -> Vec<(usize, StepKind)> {
        steps.iter().map(|s| (s.rod, s.kind)).collect()
    }

    // --- exhaustive correctness ------------------------------------------

    #[test]
    fn add_every_digit_pair_is_arithmetically_right() {
        for d in 0..=9u8 {
            for a in 1..=9u8 {
                let mut s = rod_with(d);
                let steps = add_digit(&mut s, 1, a).unwrap();
                assert_eq!(s.value(), (d + a) as u128, "{d}+{a}");
                // Replaying the steps on a fresh copy gives the same result.
                let mut replay = rod_with(d);
                apply_steps(&mut replay, &steps);
                assert_eq!(replay, s, "{d}+{a} replay");
                // Each step is a legal bead move (debug asserts in apply_step).
                // Per-rod digit deltas: -friend on this rod and +1 on the left
                // sum to a - 9 when a ten-complement is used.
                let sum: i32 = steps.iter().map(|s| s.kind.delta() as i32).sum();
                assert_eq!(
                    sum,
                    a as i32 - if d + a >= 10 { 9 } else { 0 },
                    "{d}+{a} delta"
                );
            }
        }
    }

    #[test]
    fn sub_every_digit_pair_is_arithmetically_right() {
        for d in 0..=9u8 {
            for a in 1..=9u8 {
                let mut s = Soroban::new(2);
                s.set_value(10 + d as u128);
                let steps = sub_digit(&mut s, 1, a).unwrap();
                assert_eq!(s.value(), (10 + d - a) as u128, "{d}-{a}");
                let mut replay = Soroban::new(2);
                replay.set_value(10 + d as u128);
                apply_steps(&mut replay, &steps);
                assert_eq!(replay, s, "{d}-{a} replay");
            }
        }
    }

    // --- direct moves ----------------------------------------------------

    #[test]
    fn direct_earth_add_uses_thumb() {
        let mut s = rod_with(1);
        let steps = add_digit(&mut s, 1, 3).unwrap();
        assert_eq!(kinds(&steps), vec![(1, EarthUp(3))]);
        assert_eq!(steps[0].rule, Rule::Direct);
        assert_eq!(steps[0].finger(), Finger::Thumb);
    }

    #[test]
    fn direct_heaven_add_uses_index() {
        let mut s = rod_with(3);
        let steps = add_digit(&mut s, 1, 5).unwrap();
        assert_eq!(kinds(&steps), vec![(1, HeavenDown)]);
        assert_eq!(steps[0].finger(), Finger::Index);
    }

    #[test]
    fn direct_add_six_to_nine_on_empty_rod_sets_heaven_then_earth() {
        for a in 6..=9u8 {
            let mut s = rod_with(0);
            let steps = add_digit(&mut s, 1, a).unwrap();
            assert_eq!(
                kinds(&steps),
                vec![(1, HeavenDown), (1, EarthUp(a - 5))],
                "{a}"
            );
            assert!(steps.iter().all(|s| s.rule == Rule::Direct));
        }
    }

    #[test]
    fn direct_subtraction() {
        let mut s = rod_with(8);
        let steps = sub_digit(&mut s, 1, 2).unwrap();
        assert_eq!(kinds(&steps), vec![(1, EarthDown(2))]);
        let mut s = rod_with(8);
        let steps = sub_digit(&mut s, 1, 7).unwrap();
        assert_eq!(kinds(&steps), vec![(1, EarthDown(2)), (1, HeavenUp)]);
        let mut s = rod_with(8);
        let steps = sub_digit(&mut s, 1, 5).unwrap();
        assert_eq!(kinds(&steps), vec![(1, HeavenUp)]);
    }

    // --- five-complement (small friends) ---------------------------------

    #[test]
    fn five_complement_addition_table() {
        // (digit, add) -> friend; every case where earth beads run out but the
        // result stays below 10.
        let cases = [
            (1, 4, 1),
            (2, 4, 1),
            (3, 4, 1),
            (4, 4, 1),
            (2, 3, 2),
            (3, 3, 2),
            (4, 3, 2),
            (3, 2, 3),
            (4, 2, 3),
            (4, 1, 4),
        ];
        for (d, a, friend) in cases {
            let mut s = rod_with(d);
            let steps = add_digit(&mut s, 1, a).unwrap();
            assert_eq!(
                kinds(&steps),
                vec![(1, HeavenDown), (1, EarthDown(friend))],
                "{d}+{a}"
            );
            assert!(steps
                .iter()
                .all(|s| s.rule == Rule::FiveComplement { friend }));
        }
    }

    #[test]
    fn five_complement_subtraction_table() {
        let cases = [
            (5, 4, 1),
            (6, 4, 1),
            (7, 4, 1),
            (8, 4, 1),
            (5, 3, 2),
            (6, 3, 2),
            (7, 3, 2),
            (5, 2, 3),
            (6, 2, 3),
            (5, 1, 4),
        ];
        for (d, a, friend) in cases {
            let mut s = rod_with(d);
            let steps = sub_digit(&mut s, 1, a).unwrap();
            assert_eq!(
                kinds(&steps),
                vec![(1, EarthUp(friend)), (1, HeavenUp)],
                "{d}-{a}"
            );
            assert!(steps
                .iter()
                .all(|s| s.rule == Rule::FiveComplement { friend }));
        }
    }

    // --- ten-complement (big friends) ------------------------------------

    #[test]
    fn ten_complement_add_simple_cases() {
        // 4 + 8: take 2, carry 1.
        let mut s = rod_with(4);
        let steps = add_digit(&mut s, 1, 8).unwrap();
        assert_eq!(kinds(&steps), vec![(1, EarthDown(2)), (0, EarthUp(1))]);
        assert_eq!(steps[0].rule, Rule::TenComplement { friend: 2 });
        assert_eq!(steps[1].rule, Rule::Carry);
        assert_eq!(s.value(), 12);

        // 9 + 1: take 9 (heaven and four earth), carry.
        let mut s = rod_with(9);
        let steps = add_digit(&mut s, 1, 1).unwrap();
        assert_eq!(
            kinds(&steps),
            vec![(1, EarthDown(4)), (1, HeavenUp), (0, EarthUp(1))]
        );

        // 5 + 5: take 5, carry.
        let mut s = rod_with(5);
        let steps = add_digit(&mut s, 1, 5).unwrap();
        assert_eq!(kinds(&steps), vec![(1, HeavenUp), (0, EarthUp(1))]);
    }

    #[test]
    fn ten_complement_add_combined_with_five_complement() {
        // 7 + 6: friend 4 cannot be taken from earth 2: take 5, set 1; carry.
        let mut s = rod_with(7);
        let steps = add_digit(&mut s, 1, 6).unwrap();
        assert_eq!(
            kinds(&steps),
            vec![(1, EarthUp(1)), (1, HeavenUp), (0, EarthUp(1))]
        );
        assert_eq!(steps[0].rule, Rule::FiveComplement { friend: 1 });
        assert_eq!(steps[2].rule, Rule::Carry);
        assert_eq!(s.value(), 13);

        // 8 + 7: friend 3 > earth 3? earth is 3 so direct take.
        let mut s = rod_with(8);
        let steps = add_digit(&mut s, 1, 7).unwrap();
        assert_eq!(kinds(&steps), vec![(1, EarthDown(3)), (0, EarthUp(1))]);

        // 6 + 8: friend 2 > earth 1: take 5, set 3; carry.
        let mut s = rod_with(6);
        let steps = add_digit(&mut s, 1, 8).unwrap();
        assert_eq!(
            kinds(&steps),
            vec![(1, EarthUp(3)), (1, HeavenUp), (0, EarthUp(1))]
        );
        assert_eq!(s.value(), 14);
    }

    #[test]
    fn ten_complement_sub_simple_cases() {
        // 11 - 7: borrow 1, set 3.
        let mut s = Soroban::new(2);
        s.set_value(11);
        let steps = sub_digit(&mut s, 1, 7).unwrap();
        assert_eq!(kinds(&steps), vec![(0, EarthDown(1)), (1, EarthUp(3))]);
        assert_eq!(steps[0].rule, Rule::Borrow);
        assert_eq!(steps[1].rule, Rule::TenComplement { friend: 3 });
        assert_eq!(s.value(), 4);

        // 12 - 7: borrow 1, then set 3 on earth 2 needs the small friend:
        // set 5, take 2.
        let mut s = Soroban::new(2);
        s.set_value(12);
        let steps = sub_digit(&mut s, 1, 7).unwrap();
        assert_eq!(
            kinds(&steps),
            vec![(0, EarthDown(1)), (1, HeavenDown), (1, EarthDown(2))]
        );
        assert_eq!(s.value(), 5);

        // 10 - 1: borrow, set 9.
        let mut s = Soroban::new(2);
        s.set_value(10);
        let steps = sub_digit(&mut s, 1, 1).unwrap();
        assert_eq!(
            kinds(&steps),
            vec![(0, EarthDown(1)), (1, HeavenDown), (1, EarthUp(4))]
        );
        assert_eq!(s.value(), 9);
    }

    #[test]
    fn ten_complement_sub_combined_with_five_complement() {
        // 13 - 6: borrow 1, then set 4 on earth 3: set 5, take 1.
        let mut s = Soroban::new(2);
        s.set_value(13);
        let steps = sub_digit(&mut s, 1, 6).unwrap();
        assert_eq!(
            kinds(&steps),
            vec![(0, EarthDown(1)), (1, HeavenDown), (1, EarthDown(1))]
        );
        assert_eq!(steps[1].rule, Rule::FiveComplement { friend: 1 });
        assert_eq!(s.value(), 7);
    }

    // --- carry and borrow propagation ------------------------------------

    #[test]
    fn carry_propagates_through_nines() {
        let mut s = Soroban::new(5);
        s.set_value(9999);
        let steps = add_digit(&mut s, 4, 1).unwrap();
        assert_eq!(s.value(), 10000);
        // Rods 4,3,2,1 each clear 9; rod 0 gets one bead.
        assert_eq!(steps.last().unwrap().rod, 0);
        assert_eq!(steps.last().unwrap().kind, EarthUp(1));
        assert!(steps[..steps.len() - 1]
            .iter()
            .all(|s| matches!(s.rule, Rule::TenComplement { friend: 9 })));
    }

    /// The exhaustive move tables in research/soroban-mechanics-and-curriculum.md
    /// (§3.2 and §3.3) were derived independently from Kojima's rules. Every
    /// row must match the engine exactly, including the order of moves.
    #[test]
    fn research_fixture_tables_match_engine() {
        let doc = include_str!("../../../research/soroban-mechanics-and-curriculum.md");
        let mut rows = 0;
        for line in doc.lines() {
            let cols: Vec<&str> = line.split('|').map(str::trim).collect();
            // "| d | +a | = | class | moves |" has 7 split parts (empty ends).
            if cols.len() != 7 || cols[1].len() != 1 || !cols[1].as_bytes()[0].is_ascii_digit() {
                continue;
            }
            let d: u8 = cols[1].parse().unwrap();
            let op = &cols[2][..1];
            let a: u8 = cols[2][1..].parse().unwrap();
            let expected = parse_moves(cols[5]);
            let mut s = Soroban::new(2);
            let steps = if op == "+" {
                s.set_digit(1, d);
                add_digit(&mut s, 1, a).unwrap()
            } else {
                s.set_value(10 + d as u128);
                sub_digit(&mut s, 1, a).unwrap()
            };
            assert_eq!(kinds(&steps), expected, "row: {line}");
            rows += 1;
        }
        assert_eq!(rows, 180, "both 90-row tables must be present");
    }

    fn parse_moves(cell: &str) -> Vec<(usize, StepKind)> {
        let cell = cell.replace("(together)", "");
        cell.split([',', '&'])
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|t| match t {
                "+5" => (1, HeavenDown),
                "-5" => (1, HeavenUp),
                "+1L" => (0, EarthUp(1)),
                "-1L" => (0, EarthDown(1)),
                _ if t.starts_with("+e") => (1, EarthUp(t[2..].parse().unwrap())),
                _ if t.starts_with("-e") => (1, EarthDown(t[2..].parse().unwrap())),
                _ => panic!("unknown move token {t:?}"),
            })
            .collect()
    }

    #[test]
    fn borrow_propagates_through_zeros() {
        let mut s = Soroban::new(5);
        s.set_value(10000);
        let steps = sub_digit(&mut s, 4, 1).unwrap();
        assert_eq!(s.value(), 9999);
        assert_eq!(steps[0], Step::new(0, EarthDown(1), Rule::Borrow));
    }

    #[test]
    fn overflow_and_underflow_leave_soroban_untouched() {
        let mut s = Soroban::new(2);
        s.set_value(99);
        let before = s.clone();
        assert_eq!(add_digit(&mut s, 1, 1), Err(MoveError::Overflow));
        assert_eq!(s, before);

        let mut s = Soroban::new(2);
        s.set_value(3);
        let before = s.clone();
        assert_eq!(sub_digit(&mut s, 1, 4), Err(MoveError::Underflow));
        assert_eq!(s, before);
    }

    #[test]
    fn carry_first_convention() {
        let conv = Convention {
            carry_first: true,
            complement_before_borrow: true,
        };
        let mut s = rod_with(4);
        let steps = add_digit_with(&mut s, 1, 8, conv).unwrap();
        assert_eq!(kinds(&steps), vec![(0, EarthUp(1)), (1, EarthDown(2))]);
        let mut s = Soroban::new(2);
        s.set_value(11);
        let steps = sub_digit_with(&mut s, 1, 7, conv).unwrap();
        assert_eq!(kinds(&steps), vec![(1, EarthUp(3)), (0, EarthDown(1))]);
        assert_eq!(s.value(), 4);
    }

    // --- whole numbers ---------------------------------------------------

    #[test]
    fn add_and_subtract_numbers_left_to_right() {
        let mut s = Soroban::new(6);
        s.set_value(478);
        let steps = add_number(&mut s, 365).unwrap();
        assert_eq!(s.value(), 843);
        // First step touches the hundreds rod (left to right).
        assert_eq!(steps[0].rod, 3);
        let steps = sub_number(&mut s, 599).unwrap();
        assert_eq!(s.value(), 244);
        assert_eq!(steps[0].rod, 3);
    }

    #[test]
    fn number_overflow_restores_state() {
        let mut s = Soroban::new(3);
        s.set_value(950);
        let before = s.clone();
        assert_eq!(add_number(&mut s, 60), Err(MoveError::Overflow));
        assert_eq!(s, before);
        assert_eq!(sub_number(&mut s, 951), Err(MoveError::Underflow));
        assert_eq!(s, before);
        assert_eq!(add_number(&mut s, 1000), Err(MoveError::Overflow));
    }

    #[test]
    fn random_walk_matches_integer_arithmetic() {
        // A deterministic LCG keeps this reproducible without a rand dependency.
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut s = Soroban::new(13);
        let mut v: i128 = 0;
        for _ in 0..5000 {
            let n = (next() % 100_000) as i128;
            if next() % 2 == 0 || v - n < 0 {
                if (v + n) as u128 <= s.max_value() {
                    add_number(&mut s, n as u128).unwrap();
                    v += n;
                }
            } else {
                sub_number(&mut s, n as u128).unwrap();
                v -= n;
            }
            assert_eq!(s.value(), v as u128);
        }
    }

    #[test]
    fn steps_to_show_sets_each_rod_directly() {
        let s = Soroban::new(3);
        let steps = steps_to_show(&s, 407);
        assert_eq!(
            kinds(&steps),
            vec![(0, EarthUp(4)), (2, HeavenDown), (2, EarthUp(2))]
        );
        let mut t = s.clone();
        apply_steps(&mut t, &steps);
        assert_eq!(t.value(), 407);
        // From 407 to 0 we go down.
        let steps = steps_to_show(&t, 0);
        assert_eq!(
            kinds(&steps),
            vec![(0, EarthDown(4)), (2, HeavenUp), (2, EarthDown(2))]
        );
    }

    #[test]
    fn decimal_digits_examples() {
        assert_eq!(decimal_digits(0), vec![0]);
        assert_eq!(decimal_digits(7), vec![7]);
        assert_eq!(decimal_digits(1203), vec![1, 2, 0, 3]);
    }
}
