//! The curriculum: skills, their prerequisites, difficulty ladders and the
//! lesson text that teaches each one. Pure data; the UI renders it.

use serde::{Deserialize, Serialize};

/// Every skill Abax teaches, in teaching order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SkillId {
    /// Read and set numbers on the rods.
    Setting,
    /// Addition with no complements.
    AddSimple,
    /// Subtraction with no complements.
    SubSimple,
    /// Addition with the five-complement (small friends).
    AddFive,
    /// Subtraction with the five-complement.
    SubFive,
    /// Addition with the ten-complement (big friends, carry).
    AddTen,
    /// Subtraction with the ten-complement (borrow).
    SubTen,
    /// Both complements in one move.
    Combined,
    /// Multi-digit addition chains.
    MultiAdd,
    /// Multi-digit mixed chains.
    MultiMixed,
    /// Multiplication.
    Multiply,
    /// Division.
    Divide,
    /// Anzan: visualised chains with fading beads.
    Anzan,
    /// Flash anzan with one-digit numbers.
    FlashOne,
    /// Flash anzan with two or more digits.
    FlashMulti,
}

impl SkillId {
    pub const ALL: [SkillId; 15] = [
        SkillId::Setting,
        SkillId::AddSimple,
        SkillId::SubSimple,
        SkillId::AddFive,
        SkillId::SubFive,
        SkillId::AddTen,
        SkillId::SubTen,
        SkillId::Combined,
        SkillId::MultiAdd,
        SkillId::MultiMixed,
        SkillId::Multiply,
        SkillId::Divide,
        SkillId::Anzan,
        SkillId::FlashOne,
        SkillId::FlashMulti,
    ];

    /// Short code used in the research notes and stats.
    pub fn code(self) -> &'static str {
        match self {
            SkillId::Setting => "S0",
            SkillId::AddSimple => "S1",
            SkillId::SubSimple => "S2",
            SkillId::AddFive => "S3",
            SkillId::SubFive => "S4",
            SkillId::AddTen => "S5",
            SkillId::SubTen => "S6",
            SkillId::Combined => "S7",
            SkillId::MultiAdd => "S8",
            SkillId::MultiMixed => "S9",
            SkillId::Multiply => "S10",
            SkillId::Divide => "S11",
            SkillId::Anzan => "A1",
            SkillId::FlashOne => "A2",
            SkillId::FlashMulti => "A3",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SkillId::Setting => "Reading and setting numbers",
            SkillId::AddSimple => "Adding without complements",
            SkillId::SubSimple => "Subtracting without complements",
            SkillId::AddFive => "Adding with the five-complement",
            SkillId::SubFive => "Subtracting with the five-complement",
            SkillId::AddTen => "Adding with the ten-complement",
            SkillId::SubTen => "Subtracting with the ten-complement",
            SkillId::Combined => "Combined complements",
            SkillId::MultiAdd => "Multi-digit addition",
            SkillId::MultiMixed => "Mixed addition and subtraction",
            SkillId::Multiply => "Multiplication",
            SkillId::Divide => "Division",
            SkillId::Anzan => "Anzan: the mental soroban",
            SkillId::FlashOne => "Flash anzan, one digit",
            SkillId::FlashMulti => "Flash anzan, more digits",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            SkillId::Setting => "Setting",
            SkillId::AddSimple => "Add",
            SkillId::SubSimple => "Subtract",
            SkillId::AddFive => "Add +5",
            SkillId::SubFive => "Sub −5",
            SkillId::AddTen => "Add +10",
            SkillId::SubTen => "Sub −10",
            SkillId::Combined => "Combined",
            SkillId::MultiAdd => "Multi add",
            SkillId::MultiMixed => "Mixed",
            SkillId::Multiply => "Multiply",
            SkillId::Divide => "Divide",
            SkillId::Anzan => "Anzan",
            SkillId::FlashOne => "Flash 1",
            SkillId::FlashMulti => "Flash 2+",
        }
    }

    /// Prerequisites that must reach the unlock threshold first.
    pub fn prerequisites(self) -> &'static [SkillId] {
        use SkillId::*;
        match self {
            Setting => &[],
            AddSimple => &[Setting],
            SubSimple => &[Setting],
            AddFive => &[AddSimple],
            SubFive => &[SubSimple, AddFive],
            AddTen => &[AddFive],
            SubTen => &[SubFive, AddTen],
            Combined => &[AddTen, SubTen],
            MultiAdd => &[Combined],
            MultiMixed => &[MultiAdd],
            Multiply => &[MultiAdd],
            Divide => &[Multiply],
            Anzan => &[Combined],
            FlashOne => &[Anzan, MultiMixed],
            FlashMulti => &[FlashOne],
        }
    }

    /// Elo base difficulty of items for this skill.
    pub fn base_difficulty(self) -> f64 {
        use SkillId::*;
        match self {
            Setting => -1.5,
            AddSimple | SubSimple => -1.0,
            AddFive | SubFive => 0.0,
            AddTen | SubTen => 0.5,
            Combined => 1.0,
            MultiAdd => 1.5,
            MultiMixed => 2.0,
            Multiply => 2.0,
            Divide => 2.5,
            Anzan => 1.5,
            FlashOne => 2.5,
            FlashMulti => 3.0,
        }
    }

    /// Benchmark seconds per operand (or per item for setting) used by the
    /// fluency gate.
    pub fn benchmark_seconds(self) -> f64 {
        use SkillId::*;
        match self {
            Setting => 3.0,
            AddSimple | SubSimple => 2.5,
            AddFive | SubFive => 3.0,
            AddTen | SubTen | Combined => 3.5,
            MultiAdd | MultiMixed => 5.0,
            Multiply => 12.0,
            Divide => 15.0,
            Anzan => 2.0,
            FlashOne | FlashMulti => 1.0,
        }
    }

    /// The difficulty ladder, easiest first.
    pub fn ladder(self) -> &'static [Level] {
        use SkillId::*;
        match self {
            Setting => SETTING_LADDER,
            AddSimple | SubSimple => SIMPLE_LADDER,
            AddFive | SubFive | AddTen | SubTen | Combined => COMPLEMENT_LADDER,
            MultiAdd | MultiMixed => MULTI_LADDER,
            Multiply => MULTIPLY_LADDER,
            Divide => DIVIDE_LADDER,
            Anzan => ANZAN_LADDER,
            FlashOne => FLASH_ONE_LADDER,
            FlashMulti => FLASH_MULTI_LADDER,
        }
    }

    pub fn is_anzan(self) -> bool {
        matches!(
            self,
            SkillId::Anzan | SkillId::FlashOne | SkillId::FlashMulti
        )
    }
}

/// Time pressure for a drill step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timing {
    Untimed,
    /// 2.0 x the learner's typical time per operand.
    Gentle,
    /// 1.3 x the learner's typical time per operand.
    Tight,
}

impl Timing {
    pub fn multiplier(self) -> Option<f64> {
        match self {
            Timing::Untimed => None,
            Timing::Gentle => Some(2.0),
            Timing::Tight => Some(1.3),
        }
    }
}

/// One step on a skill's difficulty ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Level {
    /// Digits per operand (for multiplication: digits of the multiplicand;
    /// for division: digits of the dividend).
    pub digits: u8,
    /// Number of operands in a chain (for multiplication/division: digits of
    /// the second operand; for flash: numbers flashed).
    pub operands: u8,
    pub timing: Timing,
    /// Flash anzan display time per number in milliseconds (0 when unused).
    pub flash_ms: u16,
}

const fn lv(digits: u8, operands: u8, timing: Timing) -> Level {
    Level {
        digits,
        operands,
        timing,
        flash_ms: 0,
    }
}

const fn flash(digits: u8, operands: u8, flash_ms: u16) -> Level {
    Level {
        digits,
        operands,
        timing: Timing::Untimed,
        flash_ms,
    }
}

use Timing::*;

pub const SETTING_LADDER: &[Level] = &[
    lv(1, 1, Untimed),
    lv(2, 1, Untimed),
    lv(3, 1, Untimed),
    lv(3, 1, Gentle),
    lv(4, 1, Gentle),
    lv(5, 1, Tight),
];

pub const SIMPLE_LADDER: &[Level] = &[
    lv(1, 2, Untimed),
    lv(1, 3, Untimed),
    lv(1, 5, Untimed),
    lv(1, 5, Gentle),
    lv(1, 8, Gentle),
    lv(1, 8, Tight),
];

pub const COMPLEMENT_LADDER: &[Level] = &[
    lv(1, 2, Untimed),
    lv(1, 3, Untimed),
    lv(1, 5, Untimed),
    lv(1, 5, Gentle),
    lv(1, 8, Gentle),
    lv(1, 8, Tight),
];

pub const MULTI_LADDER: &[Level] = &[
    lv(2, 2, Untimed),
    lv(2, 3, Untimed),
    lv(2, 5, Untimed),
    lv(3, 3, Untimed),
    lv(3, 5, Untimed),
    lv(2, 5, Gentle),
    lv(3, 5, Gentle),
    lv(3, 10, Tight),
];

pub const MULTIPLY_LADDER: &[Level] = &[
    lv(1, 1, Untimed),
    lv(2, 1, Untimed),
    lv(3, 1, Untimed),
    lv(2, 2, Untimed),
    lv(2, 2, Gentle),
    lv(3, 2, Gentle),
    lv(3, 2, Tight),
];

pub const DIVIDE_LADDER: &[Level] = &[
    lv(2, 1, Untimed),
    lv(3, 1, Untimed),
    lv(4, 1, Untimed),
    lv(3, 2, Untimed),
    lv(4, 2, Gentle),
    lv(4, 2, Tight),
];

pub const ANZAN_LADDER: &[Level] = &[
    lv(1, 2, Untimed),
    lv(1, 3, Untimed),
    lv(1, 5, Untimed),
    lv(1, 5, Gentle),
    lv(2, 3, Gentle),
    lv(2, 5, Tight),
];

pub const FLASH_ONE_LADDER: &[Level] = &[
    flash(1, 3, 1500),
    flash(1, 5, 1500),
    flash(1, 5, 1200),
    flash(1, 5, 1000),
    flash(1, 8, 800),
    flash(1, 10, 600),
    flash(1, 10, 500),
    flash(1, 15, 400),
];

pub const FLASH_MULTI_LADDER: &[Level] = &[
    flash(2, 3, 1500),
    flash(2, 5, 1200),
    flash(2, 5, 1000),
    flash(2, 8, 800),
    flash(2, 10, 600),
    flash(3, 5, 1000),
    flash(3, 10, 800),
    flash(3, 15, 500),
];

/// A lesson: plain-words explanation plus a demonstration. Demonstrations
/// are `(start value, operation)` pairs the UI plays on the soroban with
/// highlighted beads and finger labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lesson {
    pub skill: SkillId,
    pub title: &'static str,
    pub paragraphs: &'static [&'static str],
    pub demos: &'static [Demo],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demo {
    /// Show the number on an empty soroban.
    Set(u128),
    /// Start from the first number and add the second.
    Add(u128, u128),
    /// Start from the first number and subtract the second.
    Sub(u128, u128),
    /// Multiply the two numbers.
    Mul(u128, u128),
    /// Divide the first number by the second.
    Div(u128, u128),
}

pub fn lesson(skill: SkillId) -> &'static Lesson {
    LESSONS
        .iter()
        .find(|l| l.skill == skill)
        .expect("every skill has a lesson")
}

pub static LESSONS: &[Lesson] = &[
    Lesson {
        skill: SkillId::Setting,
        title: "The rods, the beam and the beads",
        paragraphs: &[
            "Each rod is one decimal place. The rightmost rod is the units, the next one the tens, and so on. The dots on the beam mark every third rod so you can find the thousands and millions at a glance.",
            "A bead counts only when it touches the beam. The single bead above the beam is the heaven bead and is worth 5. The four beads below are earth beads, worth 1 each.",
            "To set 3, push three earth beads up with your thumb. To set 7, push the heaven bead down with your index finger and two earth beads up with your thumb. To clear a rod, move everything away from the beam.",
        ],
        demos: &[Demo::Set(3), Demo::Set(7), Demo::Set(50), Demo::Set(406)],
    },
    Lesson {
        skill: SkillId::AddSimple,
        title: "Adding when the beads are there",
        paragraphs: &[
            "When the beads you need are free, adding is just pushing them to the beam. The thumb pushes earth beads up. The index finger pushes the heaven bead down.",
            "2 + 2: push two more earth beads up. 3 + 5: push the heaven bead down. 1 + 6: heaven bead down and one earth bead up, in one pinch.",
            "Work from the leftmost rod of the number you are adding to the rightmost. That is the order you read numbers in, and it is the order the soroban uses for everything.",
        ],
        demos: &[Demo::Add(2, 2), Demo::Add(3, 5), Demo::Add(1, 6), Demo::Add(12, 36)],
    },
    Lesson {
        skill: SkillId::SubSimple,
        title: "Subtracting when the beads are there",
        paragraphs: &[
            "Subtracting moves beads away from the beam. The index finger pushes earth beads down and the heaven bead up.",
            "4 − 3: push three earth beads down. 8 − 5: push the heaven bead up. 9 − 7: heaven bead up and two earth beads down.",
        ],
        demos: &[Demo::Sub(4, 3), Demo::Sub(8, 5), Demo::Sub(9, 7), Demo::Sub(87, 52)],
    },
    Lesson {
        skill: SkillId::AddFive,
        title: "Small friends: adding through 5",
        paragraphs: &[
            "To add 4 to 1 you would need four free earth beads, but only three are free. Use the five-complement instead: 4 is 5 take away 1. So set the heaven bead and take one earth bead away.",
            "The small friends are the pairs that make 5: 1 and 4, 2 and 3. Whenever an earth bead is missing, set 5 and take away the friend.",
            "The index finger does both moves in one downward stroke: heaven bead down, earth bead down.",
        ],
        demos: &[Demo::Add(1, 4), Demo::Add(3, 3), Demo::Add(4, 2), Demo::Add(23, 34)],
    },
    Lesson {
        skill: SkillId::SubFive,
        title: "Small friends: subtracting through 5",
        paragraphs: &[
            "To take 4 from 6 you would need four earth beads at the beam, but only one is there. So take the heaven bead away and give back its friend: take 5, add 1.",
            "The index finger pushes the heaven bead up, then the thumb pushes the friend earth beads up. Two moves, one small motion.",
        ],
        demos: &[Demo::Sub(6, 4), Demo::Sub(5, 2), Demo::Sub(7, 3), Demo::Sub(65, 42)],
    },
    Lesson {
        skill: SkillId::AddTen,
        title: "Big friends: adding through 10",
        paragraphs: &[
            "To add 8 to 4 the rod would have to show 12, which one rod cannot do. Use the ten-complement: 8 is 10 take away 2. Take 2 from this rod, then add 1 to the rod on the left.",
            "The big friends are the pairs that make 10: 1 and 9, 2 and 8, 3 and 7, 4 and 6, 5 and 5.",
            "Always finish the carry: the 1 on the left is part of the same move.",
        ],
        demos: &[Demo::Add(4, 8), Demo::Add(9, 1), Demo::Add(5, 5), Demo::Add(38, 27)],
    },
    Lesson {
        skill: SkillId::SubTen,
        title: "Big friends: subtracting through 10",
        paragraphs: &[
            "To take 7 from 11 the units rod shows only 1. Borrow: take 1 from the tens rod, then add 7's friend, 3, on the units rod.",
            "Borrow first, then add the friend. The result stays on the same rods you started with.",
        ],
        demos: &[Demo::Sub(11, 7), Demo::Sub(10, 1), Demo::Sub(15, 8), Demo::Sub(62, 38)],
    },
    Lesson {
        skill: SkillId::Combined,
        title: "When both friends are needed",
        paragraphs: &[
            "Sometimes the complement itself cannot be moved directly. 7 + 6: 6 is 10 take away 4, but the rod showing 7 has only two earth beads up. So take 4 through 5: take the heaven bead, add 1. Then carry 1 to the left.",
            "The same happens when subtracting. 12 − 7: borrow 1, then add 3 to the 2 on the units rod. Three earth beads are not free, so set 5 and take 2.",
            "Read the rule twice, then let your fingers learn it. After a few hundred moves it becomes one gesture.",
        ],
        demos: &[Demo::Add(7, 6), Demo::Add(6, 8), Demo::Sub(12, 7), Demo::Sub(13, 6)],
    },
    Lesson {
        skill: SkillId::MultiAdd,
        title: "Longer numbers",
        paragraphs: &[
            "Nothing new: add each digit on its own rod, from left to right. Carries travel left, so a carry never lands on a rod you still have to visit.",
            "Keep your eye one rod ahead of your fingers and say the digits as you go.",
        ],
        demos: &[Demo::Add(478, 365), Demo::Add(1999, 1)],
    },
    Lesson {
        skill: SkillId::MultiMixed,
        title: "Adding and subtracting in one run",
        paragraphs: &[
            "Exam problems mix plus and minus. The soroban does not care: every operation lands on the current total.",
            "Before a subtraction, glance at the rods on the left so you know where a borrow will come from.",
        ],
        demos: &[Demo::Sub(843, 599), Demo::Add(244, 756)],
    },
    Lesson {
        skill: SkillId::Multiply,
        title: "Multiplication",
        paragraphs: &[
            "Set the multiplier on the left and the multiplicand to its right, leaving two empty rods between them. Count the digits of both numbers: the product will end that many rods to the right of the multiplicand's units rod.",
            "Multiply the last digit of the multiplicand by each digit of the multiplier in turn, adding each two-digit partial product to its own place. Then clear that digit of the multiplicand and move to the next.",
            "You only ever need to add a two-digit number to two rods: this is the addition you already know.",
        ],
        demos: &[Demo::Mul(7, 8), Demo::Mul(34, 6), Demo::Mul(23, 12)],
    },
    Lesson {
        skill: SkillId::Divide,
        title: "Division",
        paragraphs: &[
            "Set the divisor on the left and the dividend on the right. The quotient grows to the left of the dividend.",
            "Compare the divisor with the first digits of the dividend, guess a quotient digit, set it, and subtract divisor × digit. If you guessed too high, take one back and add the divisor back; if too low, add one and subtract again.",
            "The remainder is what is left of the dividend when no more quotient digits fit.",
        ],
        demos: &[Demo::Div(56, 7), Demo::Div(84, 4), Demo::Div(391, 17)],
    },
    Lesson {
        skill: SkillId::Anzan,
        title: "Anzan: the soroban in your head",
        paragraphs: &[
            "Look at the soroban as you add. Then the beads fade. Keep moving them with your eyes closed: you are building an image, not remembering digits.",
            "Start with two one-digit numbers and let the fade finish before you answer. The picture gets sharper with every session.",
        ],
        demos: &[Demo::Add(3, 4), Demo::Add(7, 6)],
    },
    Lesson {
        skill: SkillId::FlashOne,
        title: "Flash anzan",
        paragraphs: &[
            "Numbers flash one after another. Add each to the soroban in your head and type the total when the last one disappears.",
            "Speed comes from the image, not from thinking faster. When you start missing, slow the display down a step and build back up.",
        ],
        demos: &[],
    },
    Lesson {
        skill: SkillId::FlashMulti,
        title: "Flash anzan with longer numbers",
        paragraphs: &[
            "Two-digit numbers use two rods of the mental soroban. Picture both rods at once; do not add them one digit at a time.",
        ],
        demos: &[],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn prerequisites_form_a_dag_in_teaching_order() {
        for (i, s) in SkillId::ALL.iter().enumerate() {
            for p in s.prerequisites() {
                let j = SkillId::ALL.iter().position(|x| x == p).unwrap();
                assert!(j < i, "{:?} must come before {:?}", p, s);
            }
        }
    }

    #[test]
    fn exactly_one_root() {
        let roots: Vec<_> = SkillId::ALL
            .iter()
            .filter(|s| s.prerequisites().is_empty())
            .collect();
        assert_eq!(roots, vec![&SkillId::Setting]);
    }

    #[test]
    fn every_skill_has_a_lesson_and_ladder() {
        let mut seen = HashSet::new();
        for s in SkillId::ALL {
            let l = lesson(s);
            assert_eq!(l.skill, s);
            assert!(!l.paragraphs.is_empty());
            assert!(s.ladder().len() >= 3, "{s:?}");
            assert!(seen.insert(s.code()), "duplicate code {}", s.code());
        }
        assert_eq!(LESSONS.len(), SkillId::ALL.len());
    }

    #[test]
    fn ladders_never_introduce_timing_before_untimed_steps() {
        for s in SkillId::ALL {
            assert_eq!(s.ladder()[0].timing, Timing::Untimed, "{s:?}");
        }
    }

    #[test]
    fn flash_ladders_get_faster_or_longer() {
        for ladder in [FLASH_ONE_LADDER, FLASH_MULTI_LADDER] {
            for w in ladder.windows(2) {
                let (a, b) = (w[0], w[1]);
                assert!(b.flash_ms <= a.flash_ms || b.operands > a.operands || b.digits > a.digits);
            }
        }
    }
}
