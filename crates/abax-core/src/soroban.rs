//! The soroban as a pure data model: rods of one heaven bead (5) and four
//! earth beads (1 each), read as a decimal number.

use serde::{Deserialize, Serialize};

/// One rod. `heaven` is true when the five-bead is pushed down against the
/// beam; `earth` counts how many of the four one-beads are pushed up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rod {
    pub heaven: bool,
    pub earth: u8,
}

impl Rod {
    pub const fn new(heaven: bool, earth: u8) -> Self {
        debug_assert!(earth <= 4);
        Self { heaven, earth }
    }

    /// The digit this rod shows, 0..=9.
    pub const fn digit(self) -> u8 {
        (if self.heaven { 5 } else { 0 }) + self.earth
    }

    /// The rod state that shows `d` (0..=9).
    pub const fn from_digit(d: u8) -> Self {
        assert!(d <= 9, "a rod shows a single decimal digit");
        Self {
            heaven: d >= 5,
            earth: d % 5,
        }
    }

    pub const fn is_zero(self) -> bool {
        !self.heaven && self.earth == 0
    }
}

/// Default rod counts offered in the UI. 13 is the small travelling soroban,
/// 23 and 27 are the standard exam sizes.
pub const ROD_COUNTS: [usize; 6] = [9, 13, 17, 21, 23, 27];

/// A soroban. Rod 0 is the leftmost rod; the unit rod is by default the
/// rightmost one. Values are read as big-endian decimal digits.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Soroban {
    rods: Vec<Rod>,
}

impl Soroban {
    pub fn new(rod_count: usize) -> Self {
        assert!(rod_count >= 1, "a soroban needs at least one rod");
        Self {
            rods: vec![Rod::default(); rod_count],
        }
    }

    pub fn rod_count(&self) -> usize {
        self.rods.len()
    }

    pub fn rods(&self) -> &[Rod] {
        &self.rods
    }

    pub fn rod(&self, i: usize) -> Rod {
        self.rods[i]
    }

    pub fn rod_mut(&mut self, i: usize) -> &mut Rod {
        &mut self.rods[i]
    }

    /// Index of the unit rod (the rightmost rod).
    pub fn unit_rod(&self) -> usize {
        self.rods.len() - 1
    }

    /// True when rod `i` carries a unit-point dot. Dots sit on the unit rod
    /// and every third rod away from it, in both directions.
    pub fn has_unit_dot(&self, i: usize) -> bool {
        let unit = self.unit_rod();
        let dist = unit.abs_diff(i);
        dist.is_multiple_of(3)
    }

    /// Clear every rod (the tilt-and-sweep reset).
    pub fn clear(&mut self) {
        for r in &mut self.rods {
            *r = Rod::default();
        }
    }

    pub fn is_clear(&self) -> bool {
        self.rods.iter().all(|r| r.is_zero())
    }

    pub fn digit(&self, i: usize) -> u8 {
        self.rods[i].digit()
    }

    pub fn set_digit(&mut self, i: usize, d: u8) {
        self.rods[i] = Rod::from_digit(d);
    }

    /// The digits shown, left to right.
    pub fn digits(&self) -> Vec<u8> {
        self.rods.iter().map(|r| r.digit()).collect()
    }

    /// The number shown, read with the rightmost rod as units.
    /// Up to 38 rods fit in a u128; the UI never offers more than 27.
    pub fn value(&self) -> u128 {
        self.rods
            .iter()
            .fold(0u128, |acc, r| acc * 10 + r.digit() as u128)
    }

    /// Set the whole soroban to show `n`, units on the rightmost rod.
    /// Digits that do not fit are dropped on the left.
    pub fn set_value(&mut self, mut n: u128) {
        for i in (0..self.rods.len()).rev() {
            self.rods[i] = Rod::from_digit((n % 10) as u8);
            n /= 10;
        }
    }

    /// Set `n` so that its units digit lands on rod `unit`.
    pub fn set_value_at(&mut self, unit: usize, mut n: u128) {
        for i in (0..=unit).rev() {
            self.rods[i] = Rod::from_digit((n % 10) as u8);
            n /= 10;
        }
    }

    /// Read the number whose units digit is on rod `unit`, spanning `len`
    /// rods to the left (inclusive of `unit`).
    pub fn read_at(&self, unit: usize, len: usize) -> u128 {
        let start = (unit + 1).saturating_sub(len);
        self.rods[start..=unit]
            .iter()
            .fold(0u128, |acc, r| acc * 10 + r.digit() as u128)
    }

    /// Resize, keeping the value when it fits.
    pub fn resize(&mut self, rod_count: usize) {
        let v = self.value();
        *self = Self::new(rod_count);
        self.set_value(v);
    }

    /// The largest value this soroban can show.
    pub fn max_value(&self) -> u128 {
        10u128.pow(self.rods.len() as u32) - 1
    }
}

impl std::fmt::Display for Soroban {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in &self.rods {
            write!(f, "{}", r.digit())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rod_digit_round_trip() {
        for d in 0..=9u8 {
            let r = Rod::from_digit(d);
            assert_eq!(r.digit(), d);
            assert_eq!(r.heaven, d >= 5);
            assert_eq!(r.earth, d % 5);
        }
    }

    #[test]
    fn value_round_trip_13_rods() {
        let mut s = Soroban::new(13);
        for n in [0u128, 1, 9, 10, 55, 1234567890123, 9999999999999] {
            s.set_value(n);
            assert_eq!(s.value(), n, "{n}");
        }
    }

    #[test]
    fn value_round_trip_27_rods() {
        let mut s = Soroban::new(27);
        let n: u128 = 123456789012345678901234567;
        s.set_value(n);
        assert_eq!(s.value(), n);
        assert_eq!(s.to_string(), "123456789012345678901234567");
        assert_eq!(s.max_value(), 10u128.pow(27) - 1);
    }

    #[test]
    fn overflow_drops_left_digits() {
        let mut s = Soroban::new(3);
        s.set_value(12345);
        assert_eq!(s.value(), 345);
    }

    #[test]
    fn unit_dots_every_third_rod() {
        let s = Soroban::new(13);
        let dots: Vec<usize> = (0..13).filter(|&i| s.has_unit_dot(i)).collect();
        assert_eq!(dots, vec![0, 3, 6, 9, 12]);
        let s = Soroban::new(17);
        let dots: Vec<usize> = (0..17).filter(|&i| s.has_unit_dot(i)).collect();
        assert_eq!(dots, vec![1, 4, 7, 10, 13, 16]);
    }

    #[test]
    fn clear_and_set_digit() {
        let mut s = Soroban::new(5);
        s.set_digit(4, 7);
        s.set_digit(0, 3);
        assert_eq!(s.value(), 30007);
        assert!(!s.is_clear());
        s.clear();
        assert!(s.is_clear());
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn set_and_read_at_unit() {
        let mut s = Soroban::new(9);
        s.set_value_at(4, 123);
        assert_eq!(s.digits(), vec![0, 0, 1, 2, 3, 0, 0, 0, 0]);
        assert_eq!(s.read_at(4, 3), 123);
        assert_eq!(s.read_at(4, 5), 123);
        assert_eq!(s.read_at(3, 2), 12);
    }

    #[test]
    fn resize_keeps_value() {
        let mut s = Soroban::new(13);
        s.set_value(4242);
        s.resize(27);
        assert_eq!(s.value(), 4242);
        s.resize(9);
        assert_eq!(s.value(), 4242);
    }
}
