# Soroban mechanics, technique, and curriculum — research notes for Abax

Compiled 2026-10 from primary and semi-primary sources (Kojima 1954, Japanese
federation exam pages, Wikipedia/Wikibooks, sorobanexam.org, research papers).
Where sources disagree this is stated. Section 3 contains the exhaustive
bead-move tables intended to become unit-test fixtures.

Notation used throughout:

| token | meaning |
|---|---|
| `+eN` | thumb pushes N earth (1-)beads UP toward the beam (adds N) |
| `-eN` | index finger pushes N earth beads DOWN away from the beam (subtracts N) |
| `+5` | index finger pushes the heaven (5-)bead DOWN to the beam (adds 5) |
| `-5` | index finger pushes the heaven bead UP away from the beam (subtracts 5) |
| `+1L` | add 1 on the rod immediately to the LEFT (carry) |
| `-1L` | subtract 1 on the rod immediately to the LEFT (borrow) |

---

## 1. Mechanics of the instrument

### 1.1 Parts and values

- Each rod (桁 *keta*) carries **one heaven bead** (五玉 *go-dama*, value 5) above the
  beam and **four earth beads** (一玉 *ichi-dama*, value 1 each) below it. The horizontal
  bar is the **beam / reckoning bar** (梁 *hari*). [Wikipedia-Soroban], [Kojima]
- A bead counts only when it touches the beam: earth beads are pushed **up**, the
  heaven bead is pushed **down**. Each rod therefore shows one decimal digit 0–9 in
  bi-quinary form (0–4 = earth beads only; 5–9 = heaven bead + 0–4 earth beads).
  [Wikipedia-Soroban]
- Rods read left-to-right as ordinary decimal notation (Wikipedia's example: 8036).
- **Unit points** (定位点 *teiiten*): Kojima — "the beam is marked with a unit point at
  every third rod." The operator chooses one dotted rod as the units rod; dots to its
  left then fall on thousands, millions, etc.; rods to its right hold decimals except in
  multiplication/division where the product/quotient position is governed by the rules in
  §4. [Kojima], [Wikipedia-Soroban]

### 1.2 Rod counts

| source | statement |
|---|---|
| Kojima (1954) | "The most common type of abacus in Japan has twenty-one bamboo rods"; "larger types with twenty-seven or thirty-one rods, and smaller ones with seventeen or thirteen rods, are also used." |
| Wikipedia | rod count "is always odd and never fewer than seven"; basic models 13; practical/standard models 21, 23, 27 or 31. |
| Japanese retail/teaching articles (coeteco 12694) | the three mainstream sizes sold today are **13, 23 and 27** rods; 27 was traditionally the western-Japan norm. |

What exams need: 日商 1級 みとり算 is 10-digit × 10-term addition/subtraction, かけ算 has
multiplicand + multiplier = 11 digits, わり算 divisor + quotient = 10 digits (§5.2).
A product of 11 digits plus a 2-rod gap plus the multiplier, or a 10-digit dividend
plus a 4-rod gap plus the divisor, fits on 23 rods, which is why the 23-rod soroban is
the de-facto student/exam instrument; no federation page fetched specifies a rod count
as a rule. **Recommendation for Abax: default 23, offer 13/17/21/27.**

### 1.3 Beads: shape, material, size

- Shape: Wikipedia and sorobanexam.org describe the beads as **biconal** (two cones
  base-to-base) — sorobanexam adds "with a thin slice", i.e. a sharp equatorial ridge.
  This is the "diamond"/算盤玉 profile; it lets the fingertip catch the ridge and move a
  group of beads in one stroke. [Wikipedia-Soroban], [sorobanexam-basics]
- Material: traditionally wood; Japanese articles say the most common student bead is
  **birch** (樺, "カバ玉") for its resistance to bead-wobble; frames wood, rods bamboo
  (Kojima: "bamboo rods"), rattan or metal; plastic/one-touch (ワンタッチ) frames with a
  reset button are common for children. [coeteco 12694], [Wikipedia-Soroban]
- Dimensions: **no normative dimension was found** in any source. One retail listing
  for a Tomoe 23-rod model gives overall size 13.07 × 2.56 × 0.63 in ≈ 332 × 65 × 16 mm,
  which implies a rod pitch of roughly 14 mm — treat as an estimate only.
- Main production centres: 播州 (Ono, Hyōgo) and 雲州 (Okuizumo, Shimane). [ja-Wikipedia-そろばん]

### 1.4 History of the 1:4 layout (sources disagree on dates)

- Suanpan-derived soroban had 2 heaven + 5 earth beads. Around 1850 one heaven bead
  was dropped. Wikipedia: Irie Garyū removed one earth bead in **1891**; the 1:4 form was
  "reintroduced in 1930 and became popular in the 1940s". Kojima: upper beads cut to one
  "toward the close of the nineteenth century" and "around **1920**" the lower beads went
  from five to four. Japanese Wikipedia: the 1:4 form was fixed as the children's standard
  in the **Shōwa 10s (1935–1944)**. [Wikipedia-Soroban], [Kojima], [ja-Wikipedia-そろばん]

### 1.5 Clearing to zero (ご破算 *gohasan*)

Kojima, verbatim: "Before using the abacus, make sure that all the beads are in the
neutral position representing zero." — "move all beads down by slanting the upper edge
toward your body" — then "raise all 5-unit beads by moving the right index finger from
left to right along the upper edge of the beam." I.e. **tilt** (gravity drops every bead,
earth beads to the bottom and heaven beads onto the beam), lay flat, then **sweep** the
index finger along the top of the beam to push the heaven beads up. [Kojima]
English handbooks confirm tilt-to-clear; a thumb+index sweep is an alternative.

---

## 2. Finger technique (運指)

### 2.1 Which finger moves what

Kojima (standard Japanese two-finger method, right hand): "Use the index finger to move
5-unit beads up and down and to move 1-unit beads down", "using the thumb only to move
1-unit beads up." Japanese beginner instruction states the same three rules:
1. 1玉を上にあげるときは親指 — earth beads up: **thumb**.
2. 1玉を下にさげるときは人差し指 — earth beads down: **index**.
3. 5玉はあげるときもさげるときも人差し指 — heaven bead both ways: **index**.
[Kojima], [coeteco 12692]

Variants: the Chinese suanpan tradition uses thumb, index and middle finger; MEXT's
braille-soroban material teaches blind learners a six-finger method (thumb, index and
middle of both hands). A one-finger (index-only) style exists informally but no
Japanese federation source endorsing it was found. [MEXT]

### 2.2 Order of operations within a rod — Kojima's six rules (verbatim)

1. "Move down a 5-unit bead and move up one or more 1-unit beads at the same time."
   (adding 6, 7, 8, 9 directly: pinch — index down on the 5, thumb up on the 1s)
2. "First move down one or more 1-unit beads, and then move up a 5-unit bead."
   (subtracting 6–9 directly: earth beads first, then heaven)
3. "In quick succession first move down a 5-unit bead, and then one or more 1-unit beads."
   (five-complement **addition**, e.g. 4+3: `+5` then `-e2` — **heaven then earth**)
4. "In quick succession first move up one or more 1-unit beads, and then a 5-unit bead."
   (five-complement **subtraction**, e.g. 7−3: `+e2` then `-5` — **earth then heaven**)
5. "In addition, after finishing operation on the unit rod, move up a 1-unit bead on
   the tens' rod." (carry **after** the complement is removed)
6. "In subtraction, after subtracting a 1-unit bead from the tens' rod, operate on the
   unit rod." (borrow **before** the complement is added)
[Kojima, ch. III]

Japanese school materials agree: a Miyagi 4th-grade worksheet — "5玉を入れ1玉をはらうときは
①人差し指で先に5玉を入れ、②次に1玉をはらいます" (set the 5 first, then clear the 1s) — and a
Japanese teaching article's 8+9 walk-through: subtract 1 on the units rod, **then** add
1 on the tens rod. Some English tutorials narrate the carry first ("add 10, take away
3"); the hand order taught in Japan is complement first, carry second (addition) and
borrow first, complement second (subtraction). [Miyagi], [coeteco 12690]

### 2.3 Order across rods

Kojima: "add or subtract beginning with the highest-place digit" — multi-digit numbers
are entered **left to right**, one rod at a time, carrying/borrowing as you go. [Kojima]
Groups of beads are moved as a group, never one at a time (coeteco 12690).

---

## 3. Complement rules and exhaustive bead-move tables

### 3.1 The rules

- **Five-complements** (5の合成 / "small friends"): 1↔4, 2↔3. Used when the result stays
  on the rod (≤ 9 or ≥ 0) but the earth beads cannot do it alone.
  Add *a* (1–4): `+5, -e(5-a)`. Subtract *s* (1–4): `+e(5-s), -5`.
  Kojima notes 5 "has only two pairs (4 and 1, 3 and 2)".
- **Ten-complements** (10の合成 / "big friends"): 1↔9, 2↔8, 3↔7, 4↔6, 5↔5.
  Add *a* when d+a ≥ 10: remove (10−a) on the rod, then `+1L`.
  Subtract *s* when d < s: `-1L`, then add (10−s) on the rod.
  Kojima: "10 has only five groups of complementary digits."
- **Combined** (TF): the ten-complement move itself needs a five-complement.
  Addition: removing c = 10−a from a rod showing 5+e with e < c is done as
  `+e(5-c), -5` (e.g. 7+7: 3 cannot be taken from 7's two earth beads, so **add 2, take
  5**, then carry → 14). Exactly the pairs 5+6..5+9, 6+6..6+8, 7+6, 7+7, 8+6 (matches
  sorobanexam's "combined" list). Subtraction: after borrowing, adding c = 10−s to a rod
  showing 0–4 with d+c ≥ 5 is `+5, -e(5-c)` (e.g. 13−7: borrow, then 3 cannot take +3
  directly, so **set 5, take 2** → 6). Pairs 1−6..4−6, 2−7..4−7, 3−8, 4−8, 4−9.
  (Note: 8+7 is *not* combined — 8 has three earth beads, so `-e3, +1L` → 15.)
[sorobanexam-add], [Kojima]

Class codes in the tables: **D** direct, **F** five-complement, **T** ten-complement
(carry/borrow, complement moved directly), **TF** ten-complement whose on-rod move is
itself a five-complement. Orders follow Kojima's rules 1–6. "together" = rule 1 pinch.

### 3.2 Addition fixtures: rod shows d, add a (1–9)

| d | +a | = | class | moves (in order) |
|---|----|---|-------|------------------|
| 0 | +1 | 1 | D | +e1 |
| 0 | +2 | 2 | D | +e2 |
| 0 | +3 | 3 | D | +e3 |
| 0 | +4 | 4 | D | +e4 |
| 0 | +5 | 5 | D | +5 |
| 0 | +6 | 6 | D | +5 & +e1 (together) |
| 0 | +7 | 7 | D | +5 & +e2 (together) |
| 0 | +8 | 8 | D | +5 & +e3 (together) |
| 0 | +9 | 9 | D | +5 & +e4 (together) |
| 1 | +1 | 2 | D | +e1 |
| 1 | +2 | 3 | D | +e2 |
| 1 | +3 | 4 | D | +e3 |
| 1 | +4 | 5 | F | +5, -e1 |
| 1 | +5 | 6 | D | +5 |
| 1 | +6 | 7 | D | +5 & +e1 (together) |
| 1 | +7 | 8 | D | +5 & +e2 (together) |
| 1 | +8 | 9 | D | +5 & +e3 (together) |
| 1 | +9 | 10 | T | -e1, +1L |
| 2 | +1 | 3 | D | +e1 |
| 2 | +2 | 4 | D | +e2 |
| 2 | +3 | 5 | F | +5, -e2 |
| 2 | +4 | 6 | F | +5, -e1 |
| 2 | +5 | 7 | D | +5 |
| 2 | +6 | 8 | D | +5 & +e1 (together) |
| 2 | +7 | 9 | D | +5 & +e2 (together) |
| 2 | +8 | 10 | T | -e2, +1L |
| 2 | +9 | 11 | T | -e1, +1L |
| 3 | +1 | 4 | D | +e1 |
| 3 | +2 | 5 | F | +5, -e3 |
| 3 | +3 | 6 | F | +5, -e2 |
| 3 | +4 | 7 | F | +5, -e1 |
| 3 | +5 | 8 | D | +5 |
| 3 | +6 | 9 | D | +5 & +e1 (together) |
| 3 | +7 | 10 | T | -e3, +1L |
| 3 | +8 | 11 | T | -e2, +1L |
| 3 | +9 | 12 | T | -e1, +1L |
| 4 | +1 | 5 | F | +5, -e4 |
| 4 | +2 | 6 | F | +5, -e3 |
| 4 | +3 | 7 | F | +5, -e2 |
| 4 | +4 | 8 | F | +5, -e1 |
| 4 | +5 | 9 | D | +5 |
| 4 | +6 | 10 | T | -e4, +1L |
| 4 | +7 | 11 | T | -e3, +1L |
| 4 | +8 | 12 | T | -e2, +1L |
| 4 | +9 | 13 | T | -e1, +1L |
| 5 | +1 | 6 | D | +e1 |
| 5 | +2 | 7 | D | +e2 |
| 5 | +3 | 8 | D | +e3 |
| 5 | +4 | 9 | D | +e4 |
| 5 | +5 | 10 | T | -5, +1L |
| 5 | +6 | 11 | TF | +e1, -5, +1L |
| 5 | +7 | 12 | TF | +e2, -5, +1L |
| 5 | +8 | 13 | TF | +e3, -5, +1L |
| 5 | +9 | 14 | TF | +e4, -5, +1L |
| 6 | +1 | 7 | D | +e1 |
| 6 | +2 | 8 | D | +e2 |
| 6 | +3 | 9 | D | +e3 |
| 6 | +4 | 10 | T | -e1, -5, +1L |
| 6 | +5 | 11 | T | -5, +1L |
| 6 | +6 | 12 | TF | +e1, -5, +1L |
| 6 | +7 | 13 | TF | +e2, -5, +1L |
| 6 | +8 | 14 | TF | +e3, -5, +1L |
| 6 | +9 | 15 | T | -e1, +1L |
| 7 | +1 | 8 | D | +e1 |
| 7 | +2 | 9 | D | +e2 |
| 7 | +3 | 10 | T | -e2, -5, +1L |
| 7 | +4 | 11 | T | -e1, -5, +1L |
| 7 | +5 | 12 | T | -5, +1L |
| 7 | +6 | 13 | TF | +e1, -5, +1L |
| 7 | +7 | 14 | TF | +e2, -5, +1L |
| 7 | +8 | 15 | T | -e2, +1L |
| 7 | +9 | 16 | T | -e1, +1L |
| 8 | +1 | 9 | D | +e1 |
| 8 | +2 | 10 | T | -e3, -5, +1L |
| 8 | +3 | 11 | T | -e2, -5, +1L |
| 8 | +4 | 12 | T | -e1, -5, +1L |
| 8 | +5 | 13 | T | -5, +1L |
| 8 | +6 | 14 | TF | +e1, -5, +1L |
| 8 | +7 | 15 | T | -e3, +1L |
| 8 | +8 | 16 | T | -e2, +1L |
| 8 | +9 | 17 | T | -e1, +1L |
| 9 | +1 | 10 | T | -e4, -5, +1L |
| 9 | +2 | 11 | T | -e3, -5, +1L |
| 9 | +3 | 12 | T | -e2, -5, +1L |
| 9 | +4 | 13 | T | -e1, -5, +1L |
| 9 | +5 | 14 | T | -5, +1L |
| 9 | +6 | 15 | T | -e4, +1L |
| 9 | +7 | 16 | T | -e3, +1L |
| 9 | +8 | 17 | T | -e2, +1L |
| 9 | +9 | 18 | T | -e1, +1L |

Counts: D 36, F 10, T 34, TF 10 (90 pairs).

### 3.3 Subtraction fixtures: rod shows d, subtract s (1–9)

"=" is the digit left on this rod (when d < s the true result is 10 + that digit
minus 10 on the left rod).

| d | -s | = | class | moves (in order) |
|---|----|---|-------|------------------|
| 0 | -1 | 9 | T | -1L, +5 & +e4 (together) |
| 0 | -2 | 8 | T | -1L, +5 & +e3 (together) |
| 0 | -3 | 7 | T | -1L, +5 & +e2 (together) |
| 0 | -4 | 6 | T | -1L, +5 & +e1 (together) |
| 0 | -5 | 5 | T | -1L, +5 |
| 0 | -6 | 4 | T | -1L, +e4 |
| 0 | -7 | 3 | T | -1L, +e3 |
| 0 | -8 | 2 | T | -1L, +e2 |
| 0 | -9 | 1 | T | -1L, +e1 |
| 1 | -1 | 0 | D | -e1 |
| 1 | -2 | 9 | T | -1L, +5 & +e3 (together) |
| 1 | -3 | 8 | T | -1L, +5 & +e2 (together) |
| 1 | -4 | 7 | T | -1L, +5 & +e1 (together) |
| 1 | -5 | 6 | T | -1L, +5 |
| 1 | -6 | 5 | TF | -1L, +5, -e1 |
| 1 | -7 | 4 | T | -1L, +e3 |
| 1 | -8 | 3 | T | -1L, +e2 |
| 1 | -9 | 2 | T | -1L, +e1 |
| 2 | -1 | 1 | D | -e1 |
| 2 | -2 | 0 | D | -e2 |
| 2 | -3 | 9 | T | -1L, +5 & +e2 (together) |
| 2 | -4 | 8 | T | -1L, +5 & +e1 (together) |
| 2 | -5 | 7 | T | -1L, +5 |
| 2 | -6 | 6 | TF | -1L, +5, -e1 |
| 2 | -7 | 5 | TF | -1L, +5, -e2 |
| 2 | -8 | 4 | T | -1L, +e2 |
| 2 | -9 | 3 | T | -1L, +e1 |
| 3 | -1 | 2 | D | -e1 |
| 3 | -2 | 1 | D | -e2 |
| 3 | -3 | 0 | D | -e3 |
| 3 | -4 | 9 | T | -1L, +5 & +e1 (together) |
| 3 | -5 | 8 | T | -1L, +5 |
| 3 | -6 | 7 | TF | -1L, +5, -e1 |
| 3 | -7 | 6 | TF | -1L, +5, -e2 |
| 3 | -8 | 5 | TF | -1L, +5, -e3 |
| 3 | -9 | 4 | T | -1L, +e1 |
| 4 | -1 | 3 | D | -e1 |
| 4 | -2 | 2 | D | -e2 |
| 4 | -3 | 1 | D | -e3 |
| 4 | -4 | 0 | D | -e4 |
| 4 | -5 | 9 | T | -1L, +5 |
| 4 | -6 | 8 | TF | -1L, +5, -e1 |
| 4 | -7 | 7 | TF | -1L, +5, -e2 |
| 4 | -8 | 6 | TF | -1L, +5, -e3 |
| 4 | -9 | 5 | TF | -1L, +5, -e4 |
| 5 | -1 | 4 | F | +e4, -5 |
| 5 | -2 | 3 | F | +e3, -5 |
| 5 | -3 | 2 | F | +e2, -5 |
| 5 | -4 | 1 | F | +e1, -5 |
| 5 | -5 | 0 | D | -5 |
| 5 | -6 | 9 | T | -1L, +e4 |
| 5 | -7 | 8 | T | -1L, +e3 |
| 5 | -8 | 7 | T | -1L, +e2 |
| 5 | -9 | 6 | T | -1L, +e1 |
| 6 | -1 | 5 | D | -e1 |
| 6 | -2 | 4 | F | +e3, -5 |
| 6 | -3 | 3 | F | +e2, -5 |
| 6 | -4 | 2 | F | +e1, -5 |
| 6 | -5 | 1 | D | -5 |
| 6 | -6 | 0 | D | -e1, -5 |
| 6 | -7 | 9 | T | -1L, +e3 |
| 6 | -8 | 8 | T | -1L, +e2 |
| 6 | -9 | 7 | T | -1L, +e1 |
| 7 | -1 | 6 | D | -e1 |
| 7 | -2 | 5 | D | -e2 |
| 7 | -3 | 4 | F | +e2, -5 |
| 7 | -4 | 3 | F | +e1, -5 |
| 7 | -5 | 2 | D | -5 |
| 7 | -6 | 1 | D | -e1, -5 |
| 7 | -7 | 0 | D | -e2, -5 |
| 7 | -8 | 9 | T | -1L, +e2 |
| 7 | -9 | 8 | T | -1L, +e1 |
| 8 | -1 | 7 | D | -e1 |
| 8 | -2 | 6 | D | -e2 |
| 8 | -3 | 5 | D | -e3 |
| 8 | -4 | 4 | F | +e1, -5 |
| 8 | -5 | 3 | D | -5 |
| 8 | -6 | 2 | D | -e1, -5 |
| 8 | -7 | 1 | D | -e2, -5 |
| 8 | -8 | 0 | D | -e3, -5 |
| 8 | -9 | 9 | T | -1L, +e1 |
| 9 | -1 | 8 | D | -e1 |
| 9 | -2 | 7 | D | -e2 |
| 9 | -3 | 6 | D | -e3 |
| 9 | -4 | 5 | D | -e4 |
| 9 | -5 | 4 | D | -5 |
| 9 | -6 | 3 | D | -e1, -5 |
| 9 | -7 | 2 | D | -e2, -5 |
| 9 | -8 | 1 | D | -e3, -5 |
| 9 | -9 | 0 | D | -e4, -5 |

Counts: D 36, F 10, T 34, TF 10 (90 pairs) — the subtraction table is the mirror of
the addition table (sub(d, s) undoes add(d−s, s)).

### 3.4 Generating rule (for the implementation)

Write d = 5h + e (h ∈ {0,1}, e ∈ 0..4) and the amount x = 5h' + e'.
- add on rod, no carry (d+x ≤ 9): if h+h' ≤ 1 and e+e' ≤ 4 → direct (both bead types
  together when h'=1 and e'>0); else (necessarily h=0, x ≤ 4, e+x ≥ 5) → `+5, -e(5-x)`.
- subtract on rod (d ≥ x): if h ≥ h' and e ≥ e' → direct (earth first, then heaven
  when both); else (h=1, x ≤ 4, e < x) → `+e(5-x), -5`.
- add with carry (d+x ≥ 10): subtract (10−x) on rod by the rule above, then `+1L`.
- subtract with borrow (d < x): `-1L`, then add (10−x) on rod by the rule above.
Carries/borrows chain leftward when the left rod shows 9 / 0 (e.g. 99+1, 100−1).

---

## 4. Multiplication and division (standard Japanese methods)

Terminology: 実 (multiplicand / dividend), 法 (multiplier / divisor), 商 (quotient).
Exam specs speak of "実法合わせて N 桁" (digits of both factors sum to N) and
"法商合わせて N 桁" (digits of divisor + quotient sum to N). [shuzan.jp-kentei]

### 4.1 Multiplication — Kojima / Abacus Committee "standard method"

Placement (Kojima, sorobanexam agree):
- Set the **multiplicand** in the centre-right with its units digit on a unit rod.
- Set the **multiplier to the left**, leaving **two empty rods** between them (experts
  often omit setting a short multiplier). [Kojima], [sorobanexam-multiply]

Procedure:
1. Take the **rightmost** digit of the multiplicand. Multiply it by the multiplier's
   digits from **left to right**.
2. Position rule (Kojima, verbatim): "the first rod to the right of that figure in the
   multiplicand which is multiplied is the tens' rod of the product." The two-digit
   partial product (pad 0x if < 10) is **added** with its tens on that rod and its units
   one further right. Each subsequent multiplier digit shifts the partial product **one
   rod further right**.
3. Clear the multiplicand digit just used, move to the next multiplicand digit to the
   left, repeat. The multiplicand is consumed; the product grows to its right.
4. Final position — Kojima's **Rule A**: the product's units rod is to the right of the
   multiplicand's units rod "by as many rods plus one as there are whole digits in the
   multiplier". For a decimal multiplier whose first significant digit is in the tenths
   place (Rule C) the product's last digit lands on the first rod right of the
   multiplicand's last digit, shifting one rod left per further place of reduction.
   [Kojima]

Worked example 36 × 24 = 864 (rods labelled A…J left to right; F is the unit rod):

| step | action | rods E F G H I | note |
|---|---|---|---|
| 0 | set 24 on A–B, two gaps, 36 on E–F | 3 6 . . . | multiplicand 36, F = units |
| 1 | 6 (rod F) × 2 = 12 → tens on G, units on H | 3 6 1 2 . | first rod right of F is tens |
| 2 | 6 × 4 = 24 → shifted right: tens on H, units on I | 3 6 1 4 4 | H: 2+2 = 4 |
| 3 | clear 6 from F | 3 0 1 4 4 | 144 = 6 × 24 |
| 4 | 3 (rod E) × 2 = 06 → tens on F, units on G | 3 0 7 4 4 | G: 1+6 = 7 |
| 5 | 3 × 4 = 12 → tens on G, units on H | 3 0 8 6 4 | G: 7+1 = 8, H: 4+2 = 6 |
| 6 | clear 3 from E | 0 0 8 6 4 | read 864 |

Check Rule A: multiplier has 2 whole digits → product units rod = F + (2+1) = I. ✓

Japanese classroom variants (same arithmetic, different bookkeeping):
- 両置き *ryō-oki*: both factors set (Kojima's method is of this family).
- 片落とし *kata-otoshi*: only the multiplicand set.
- 両落とし *ryō-otoshi*: neither set, only the product; start **highest** place first,
  finger starting (digits of multiplicand + digits of multiplier − 2) rods left of the
  chosen unit point, shifting right one rod per digit pair. Example 3 × 365: 3×3=9 at
  hundreds (900), 3×6=18 one place right (1080), 3×5=15 (1095). [coeteco 12711]
片落とし and 両落とし are the forms commonly used in Japanese schools today. Older
head-first methods (新頭乗法 etc.) are catalogued on Japanese Wikipedia. [ja-Wikipedia-珠算]

### 4.2 Division — 商除法 *shōjohō* (standard since the 1930s)

History: the division-table method (帰除法, 八算 *hassan*) fell out of general use
"since about 1930" (Kojima); Wikipedia dates the standard method to 1935 and traces it
to Momokawa Chūbei, 1645. It needs only the multiplication table. [Kojima], [Wikipedia-Soroban]

Placement (Kojima; the Abacus Committee "favors four"):
- **Dividend** right of centre, units digit on a unit rod.
- **Divisor** to the left, **four empty rods** between (for a whole-number divisor its
  last digit sits "on the fifth rod to the left of the dividend").
- Beginners set the dividend first; experts often reverse. [Kojima], [sorobanexam-divide]

Where the quotient digit goes (the "one-rod / two-rod" rule, from Kojima's examples):
- If the dividend's current leading digit is **≥** the divisor's leading digit (the
  divisor "goes into" one digit), the quotient digit goes **two rods left** of that
  dividend digit (one empty rod is skipped). Kojima 837 ÷ 3: "the quotient figure 2 on D,
  the second rod to the left of 8".
- If it is **<** (two dividend digits are needed), the quotient digit goes on the rod
  **immediately left** of the dividend's leading digit.
- Equal leading digits: compare the next digit; if the dividend's second digit is
  smaller try 9 one rod left, if larger set 1 two rods left.
- Kojima's **Rule B**: the quotient's units rod is left of the dividend's units rod by
  (whole digits in divisor + 1). Use this to read off the answer / decimal point.
  [Kojima]

Procedure per digit:
1. Estimate a trial quotient digit q from the leading digits (Kojima's keys: divisor
   beginning with 5 → try twice the dividend's first digit; divisor beginning with 9 →
   try the dividend's first digit; if the divisor's first digit > its second, try small;
   if first < second, try large).
2. Subtract q × (divisor's first digit) from the dividend at the aligned rods, then
   q × (second digit) one rod right, etc. The subtracted dividend digits are consumed.
3. Repeat with the remainder until it is smaller than the divisor. The quotient sits on
   its own rods; what is left of the dividend is the remainder.

Revision (Kojima):
- **Trial too large** (a partial product cannot be subtracted): "subtract 1 from the
  quotient digit and add back the divisor times the difference" — i.e. lower q by 1 and
  **restore** the first-digit product already taken, then continue with the smaller q.
  Kojima's wording: "you ought to have tried a quotient figure one less than 9."
- **Trial too small** (remainder still ≥ divisor at that position): raise q by 1 and
  subtract the divisor once more. "You ought to have tried 9, i.e., a quotient figure
  one more than 8."
- Check: multiplying quotient by divisor regenerates the dividend on the same rods.

Worked example with a revision, 2208 ÷ 48 = 46 (dividend on D E F G, G = unit rod;
divisor 48 far left):

| step | action | rods C D E F G | note |
|---|---|---|---|
| 0 | set | . 2 2 0 8 | leading 2 < 4 → quotient goes on C (one rod left of D) |
| 1 | trial 22 ÷ 4 → q = 5 on C; 5×4 = 20 from DE | 5 0 2 0 8 | 22 − 20 = 2 |
| 2 | 5×8 = 40 from EF (= 20) — impossible | — | **too large** |
| 3 | revise: C 5 → 4; add back 1×4 = 4 to DE | 4 0 6 0 8 | restore |
| 4 | 4×8 = 32 from EF (= 60) | 4 0 2 8 8 | remainder 288 |
| 5 | next: 2 < 4 → quotient on D; trial 28 ÷ 4 → 7; 7×4 = 28 from EF | 4 7 0 0 8 | |
| 6 | 7×8 = 56 from FG (= 08) — impossible | — | **too large** |
| 7 | revise: D 7 → 6; add back 4 to EF | 4 6 0 4 8 | |
| 8 | 6×8 = 48 from FG (= 48) | 4 6 0 0 0 | remainder 0 |

Quotient 46 on C–D. Rule B: divisor has 2 whole digits → quotient units rod = G − 3 = D. ✓

Simple example without revision (sorobanexam): 782 ÷ 23 → 7 ≥ 2 so q=3 two rods left;
3×2=6 from 7 → 1; 3×3=9 from 18 → 9; 9 ≥ 2 → q=4; 4×2=8 from 9 → 1; 4×3=12 from 12 → 0;
quotient 34. [sorobanexam-divide]

Decimals: Kojima's Rules A–D locate the units rod from the count of whole digits in the
multiplier/divisor; alternatively (sorobanexam) compute as integers and place the point
afterwards. Exams require rounding: 日商 1–3級 round 無名数 below the third decimal place
and 名数 (yen amounts) below the yen. [Kojima], [shuzan.jp-kentei]

---

## 5. Teaching progression and the kyu/dan system

### 5.1 Order in which concepts are introduced

Reconstructed from the sorobanexam.org training ladder (which mirrors Japanese
textbooks), the level contents in §5.2–5.3, Kojima's chapter order, and MEXT's
braille-soroban syllabus (addition/subtraction → decimals → multiplication → decimal
multiplication → division → decimal division):

1. Parts, holding, clearing; **reading and setting** 0–9 then multi-digit numbers.
2. **Direct addition/subtraction** on one rod, earth beads only (1–4).
3. Direct add/sub involving the heaven bead (5–9) — pinch and earth-then-heaven orders.
4. **Five-complement addition**, then five-complement subtraction.
5. **Ten-complement addition** (carry), then subtraction with borrow.
6. **Combined** cases (TF) and chained carries across several rods.
7. Multi-digit みとり算 (mitorizan, "look-and-add"): columns of 2-digit numbers, 3–5
   terms, mixed + and −; grows to 10 terms × 10 digits at 1級.
8. **Multiplication** — already examined at 日珠連 9/10級 (1-digit × 2-digit, 実法合わせて
   3桁) and 全珠連 9級.
9. **Division** — enters at 日珠連 8級 / 全珠連 8級.
10. **Decimals and rounding**, 名数 (yen) problems — upper kyu (日商 1–3級).
11. Elective/advanced subjects (全珠連 準3級+): 伝票算 (voucher addition), 応用計算
    (word problems), 暗算; roots (開法) at dan level.
12. **Anzan** runs in parallel: Wikipedia — "Anzan drills start shortly after soroban
    study begins"; Kojima — mental calculation is practised by visualising the beads.
[sorobanexam-training], [shuzan.jp-kentei], [soroban.or.jp-exam], [MEXT], [Kojima], [Wikipedia-Soroban]

### 5.2 日商 / 日珠連 珠算能力検定 (JCCI 1–3級; 日珠連 準1–準3級, 4–10級)

Source: 日本珠算連盟 exam page (shuzan.jp). 1–3級 are run by the Japan Chamber of
Commerce and Industry; the rest by 日珠連 with JCCI sponsorship. みとり算 "10口・100字"
means 10 terms totalling 100 digits (i.e. 10-digit numbers).

| 級 | みとり算 (題 · 口 · 字) | かけ算 (題 · 実法合計桁) | わり算 (題 · 法商合計桁) | time | pass (of 300) |
|---|---|---|---|---|---|
| 1級 | 10 · 10口 · 100字 | 20 · 11桁 | 20 · 10桁 | 30 min | 240 |
| 準1級 | 10 · 10口 · 90字 | 20 · 10桁 | 20 · 9桁 | 30 min | 240 |
| 2級 | 10 · 10口 · 80字 | 20 · 9桁 | 20 · 8桁 | 30 min | 240 |
| 準2級 | 10 · 10口 · 70字 | 20 · 8桁 | 20 · 7桁 | 30 min | 240 |
| 3級 | 10 · 10口 · 60字 | 20 · 7桁 | 20 · 6桁 | 30 min | 240 |
| 準3級 | 10 · 10口 · 50字 (5題) + 60字 (5題) | 20 · 7桁 | 20 · 6桁 | 30 min | 240 |
| 4級 | 10 · 10口 · 50字 | 20 · 7桁 | 20 · 6桁 | 30 min | 210 |
| 5級 | 10 · 10口 · 40字 | 20 · 6桁 | 20 · 5桁 | 30 min | 210 |
| 6級 | 10 · 10口 · 30字 | 20 · 5桁 | 20 · 4桁 | 30 min | 210 |
| 7級 | 10 · 2桁 · 10口 · 20字 | 10 · 4桁 | 10 · 4桁 | 20 min | 120 of 200 |
| 8級 | 10 · 2桁 · 8口 · 16字 | 10 · 4桁 | 10 · 3桁 | 20 min | 120 of 200 |
| 9級 | 10 · 2桁 · 5口 · 10字 | 20 · 3桁 | — | 20 min | 120 of 200 |
| 10級 | 10 · 2桁 · 5口 · 10字 | 20 · 3桁 | — | 20 min | 60 of 200 |

Scoring: 1–6級 and 準級 each subject 100 points (みとり算 10/problem, かけ・わり 5/problem);
7–8級 みとり算 100, かけ 50, わり 50; 9–10級 みとり算 100, かけ算 100. 1–3級 use decimals
with rounding rules (§4.2). The three subjects are taken as one 30-minute (or 20-minute)
paper of 50 (or 20–30) problems. [shuzan.jp-kentei], [coeteco 12704]

Dan (段位認定, 日珠連): problem volume is **three times** 日商 1級 (since 2003; 伝票算
dropped); 珠算 and 暗算 dan examined separately since 2010. Pass thresholds: 準初段 90,
初段 100, 準二段 110, 二段 120, 準三段 130, 三段 140, 四段 160, 五段 180, 六段 200, 七段 220,
八段 240, 九段 260, 十段 290. [shuzan.jp-dani]

Kojima (1954) on the same system: licences since 1931; a first-grade candidate needed
about 80 % accuracy "within five minutes per group (one minute for the mental-calculation
group)"; a third-grade licence qualified for public-corporation employment (Wikipedia
repeats this). [Kojima], [Wikipedia-Soroban]

### 5.3 全珠連 (全国珠算教育連盟) 珠算検定 — 15級 to 1級, 準初段 to 十段

Every subject is a separate **7-minute** paper (暗算 3 min, 応用計算 10 min), 150 points
each (段位 300); digits per level are not published on the page. [soroban.or.jp-exam]

| range | subjects | pass |
|---|---|---|
| 15–10級 | 見取算 only (two papers A, B) | both ≥ 100/150 |
| 9級 | 乗算 + 見取算 | both ≥ 100 |
| 8–4級 | 乗算, 除算, 見取算 | all three ≥ 100 |
| 準3級–1級 | the three + 2 electives from 伝票算 / 暗算 / 応用計算 | all five ≥ 100 |
| 準初段–十段 | the three + 3 from 伝票算 / 暗算 / 応用計算 / 開法 | all six ≥ 80 → 準初段; 十段 needs ≥ 280/300 |

全珠連 暗算検定: 1–6級 乗暗算・除暗算・見取暗算 (3 min each, ≥ 70/100 each); 7級 drops
除暗算; 8–10級 見取暗算 only; 段位 1 subject 200 points, 十段 ≥ 190. [soroban.or.jp-anzan]

### 5.4 日珠連 暗算検定 (mental calculation, written answers)

All levels: 12 minutes, 500 points, pass 400 (7–10級 share one paper and are graded by
score: 400/350/300/250). [shuzan.jp-anzan]

| 級 | みとり暗算 (20題) | かけ暗算 (30題) | わり暗算 (30題) |
|---|---|---|---|
| 1級 | 3桁 · 10口 | 実法合計 5桁 | 法商合計 5桁 |
| 2級 | 3桁 · 7口 | 4桁 | 4桁 |
| 3級 | 3桁 · 5口 | 4桁 | 4桁 |
| 4級 | 2桁 · 5口 | 3桁 | 3桁 |
| 5級 | 1–2桁 · 5口 (8字) | 3桁 | 3桁 |
| 6級 | 1–2桁 · 5口 (6字) | 3桁 | 3桁 |
| 7–10級 | 50題: 3口, 4口, 5口 of 1-digit numbers | — | — |

(準級 mix the adjacent levels half-and-half.) A press source describes 日珠連 暗算1級 as
"3桁×2桁 in about 8 seconds per problem" and 暗算十段 as 4-digit multiplication, 8-digit
division and みとり, 40 problems each in 3 minutes, failing on 3 errors in any subject.

### 5.5 Age and time

- 日珠連 FAQ (via hugkum): children typically start in kindergarten or lower primary;
  **average starting age about 9**. Japanese articles recommend starting around 1st grade
  (ages 6–7); 全珠連's chairman argues 1st grade is best because written-column habits
  set in by grades 3–4 and hinder mental-abacus imagery. [hugkum], [coeteco 12692]
- School curriculum: MEXT's Course of Study places soroban addition/subtraction in
  **grade 3** (with follow-up in grade 4). Wikipedia: once required in grades 2–6, now
  mostly private after-school classes (そろばん塾). [MEXT], [Wikipedia-Soroban]
- Time to level (Kojima): "average teenage students may pass third grade after about
  half a year of one-hour daily practice"; adults starting after their teens "rarely pass
  first grade". A 2023 parent survey (reported via rurubu) shows most pupils attend 1–3
  years, then 3–5 years. [Kojima], [rurubu]
- 全珠連 exam pages state no age limit. [soroban.or.jp-exam]

---

## 6. Anzan (珠算式暗算) and flash anzan

### 6.1 How mental abacus is trained (Kojima, ch. VIII; modern practice)

- Visualise a cleared soroban, set the first number as a bead image, then move the
  imagined beads exactly as the hand would. Kojima: close the eyes, picture 76, add 24;
  "aid the image by flicking the index finger and thumb of the right hand as if
  calculating on a real abacus" (the characteristic air-finger movement later reported by
  Hatano & Osawa and Stigler).
- Start with sums that make round numbers (76+24, 222+555+223); keep track of hundreds
  by folding a left-hand finger each time the running sum passes 100.
- "Practising a few minutes at a time for many days is worth more than practising
  hours on a single day." Experts are "twice as quick in mental calculation as on the
  abacus". A 1952 demonstration: 50 division problems in 1 min 18.4 s. [Kojima]
- Modern pedagogy adds a fading ladder: calculate on the real soroban → with the
  soroban covered/eyes closed → finger movements only → no movement; and 読上算 /
  読上暗算 (dictated numbers) as an auditory variant; Japanese articles stress mastering
  the physical soroban first and automating the 1-digit pairs that cross 10. [coeteco 12712]

### 6.2 Flash anzan (フラッシュ暗算)

- Invented by Yūji Miyamoto (宮本裕史), head of 日本フラッシュ暗算協会, which supplies the
  official software for Japanese exams, competitions and Guinness attempts; the
  association says the method has "over 40 years" of history (so late 1970s–80s).
  Competition format: **15 three-digit numbers** flashed in succession, total display
  time around 3 s and below; the sum is answered. [flash-anzan.jp], [ja-Wikipedia-フラッシュ暗算], [Waseda]
- Official practice software: **120 levels from 1 digit × 2 numbers in 3 s to 3 digits ×
  15 numbers in 1.2 s**. [AppStore-official]
- Official exam ladder (日本フラッシュ暗算検定協会, "タイム" = total display time in seconds
  for the whole sequence; unit not printed on the page but consistent with "3桁15口3秒"):

| 級/段 | digits | numbers | time (s) |
|---|---|---|---|
| 20級 / 19級 | 1 | 3 | 5 |
| 18級 | 1 | 5 | 7 |
| 17級 | 1 | 8 | 10 |
| 16級 | 1 | 10 | 12 |
| 15級 | 1 | 12 | 15 |
| 14級 | 1 | 15 | 15 |
| 13級 | 1 | 20 | 15 |
| 12級 | 2 | 10 | 10 |
| 11級 | 2 | 15 | 15 |
| 10級 | 2 | 2 | 4 |
| 9級 | 2 | 3 | 6 |
| 8級 | 2 | 4 | 7 |
| 7級 | 2 | 5 | 8 |
| 6級 | 2 | 6 | 9 |
| 5級 | 2 | 7 | 10 |
| 4級 | 2 | 8 | 11 |
| 3級 | 2 | 10 | 12 |
| 2級 | 2 | 12 | 12 |
| 1級 | 2 | 15 | 13 |
| 初段 | 2 | 15 | 10 |
| 2段 | 3 | 4 | 4 |
| 3段 | 3 | 6 | 5 |
| 4段 | 3 | 8 | 6 |
| 5段 | 3 | 10 | 7 |
| 6段 | 3 | 12 | 8 |
| 7段 | 3 | 15 | 8 |
| 8段 | 3 | 15 | 6 |
| 9段 | 3 | 15 | 4.5 |
| 10段 | 3 | 15 | 3 |
| 11段 … 20段 | 3 | 15 | 2.8, 2.6, 2.4, 2.2, 2.0, 1.9, 1.8, 1.7, 1.6, 1.5 |

  Note the ladder is **not monotone**: 13級 (1 digit × 20 in 15 s) → 12級 (2 digits ×
  10) → 10級 (2 digits × 2 in 4 s) resets number-count when digit-count rises. Per-number
  interval = time ÷ numbers (20級: 1.67 s/number; 1級: 0.87 s; 10段: 0.2 s; 20段: 0.1 s).
  [flash-anzan.jp-school]
- Federation flash exams: 日珠連 runs one since 2004 (renamed フラッシュ暗算検定試験 in
  2009; jointly with the association from April 2026); 全珠連 runs 20 kyu + 10 dan, 15
  problems × 10 points, pass ≥ 100; dan rises every 20 points at group sittings (十段 ≥
  280 of 300). [shuzan.jp-flash], [soroban.or.jp-flash]
- Third-party trainers (Nintendo Switch "Prof. Miyamoto's", iOS apps) expose 1–7 digits,
  2–30 numbers, 0.01–0.1 s steps; one unofficial app describes 20級 as "1 digit, 3
  numbers, ~5.0 s" and 18級 "5 numbers, ~7.0 s", matching the official table.
- Records (sources disagree on sanctioning): Takeo Sasano 15 × 3-digit in **1.70 s**
  (2012 All Japan Soroban Championship; Wikipedia); Yūichirō Takakura **1.60 s** (Soroban
  Christmas Cup 2015, Waseda); Guinness World Records: **1.61 s**, Rikako Miyamoto, 72nd
  Nakano Abacus Competition, 18 Feb 2024, beating 1.62 s. [Wikipedia-MA], [Waseda], [Guinness]

Suggested Abax ladder (derived from the official table; beginner → advanced):
1 digit × 3 @ 1.5 s/number → 1 digit × 5 @ 1.0 s → 1 digit × 10 @ 0.8 s →
2 digits × 5 @ 1.0 s → 2 digits × 10 @ 0.8 s → 2 digits × 15 @ 0.6 s →
3 digits × 5 @ 0.8 s → 3 digits × 10 @ 0.5 s → 3 digits × 15 @ 0.4 s → 0.2 s (10段).

### 6.3 What the research says

| study | design | finding |
|---|---|---|
| Hatano & Osawa 1983, *Cognition* 15 | grand experts, digit span | forward span ~16, backward ~14 digits; backward almost as fast as forward; disrupted more by concurrent visuo-spatial than verbal tasks → digits held as a visuo-spatial mental abacus. Hatano, Shimizu & Amaiwa 1987: memory becomes more visuo-spatial as expertise grows. |
| Stigler 1984, *Cognitive Psychology* 16, 145–176 | abacus-trained vs untrained Taiwanese children | trained children solve by picturing bead configurations; interim sums stored as abacus images; hand movements accompany mental work. (Stigler, Chalip & Miller 1986 book-length follow-up.) |
| Frank & Barner 2012, *JEP: General* 141(1), 134–149 | children in India, interference tasks | mental abacus is held in visual working memory "by splitting the abacus into a series of columns, each of which is independently stored as a unit"; practised users are relatively unaffected by verbal interference → a non-linguistic exact-number format. |
| Barner et al. 2016, *Child Development* 87(4), 1146–1158 | 3-year RCT, Vadodara, 204 children aged 5–7 at start (BPS digest says 183); 3 h/week abacus vs extra maths | MA students outperformed controls on arithmetic; MA did **not** alter basic cognitive abilities (spatial WM, mental rotation); benefit mediated by pre-existing spatial WM — weaker-WM children gained less. |
| Barner et al. 2018, *J. Numerical Cognition* 3(3), 540–558 | 1-year classroom RCT, US grades 1–2 | **no significant** arithmetic or WM advantage; 1st-graders struggled to reach expertise; hints of place-value gains in older children; authors urge caution about short-term adoption. |
| Lo & Andrews 2022, *J. Numerical Cognition* 8(1), 89–122 | 79 Japanese experts (mean age ~11) vs 102 Australian and 99 Japanese undergraduates | experts better only on arithmetic-based tasks (memory updating, number bisection, mental addition); no spatial-WM advantage; Japanese students beat experts on operation span → transfer is **domain-specific**. |
| Hanakawa et al. 2003 | 6 ten-dan experts, fMRI | null effects on a mental-updating task (as cited by Lo & Andrews). |
| Tanaka et al. 2012, *Frontiers in Psychology* 3:315 | single expert after right-hemisphere stroke, longitudinal fMRI | lost and regained the "mental abacus"; recovery shifted activity from language areas to left superior parietal lobule — "abacus-based acalculia"; backward digit span recovered to ~forward. |
| Du et al. 2013 (fMRI, Chinese children); Wang et al. 2015 (task switching); Dong et al. 2016 (n-back) | trained children vs controls | report neural modulation and some executive gains; Lo & Andrews note these transfer claims are contested. |
| Ku et al. 2012, *PLOS ONE* | EEG + fMRI case | sequential neural processes in abacus mental addition; cited by Wikipedia for heightened right-hemisphere activity in visualising users. |

Summary for Abax's claims copy: evidence is **strong** that abacus training produces
fast, accurate arithmetic and a visuo-spatial (column-wise, 3–4 column) representation
of number; evidence that it improves general working memory, IQ or "right-brain
activation" is **weak or negative** in the best-controlled trials. No peer-reviewed
study specifically of flash anzan (as opposed to mental abacus) was found.

---

## 7. Open items / disagreements noted

- Date of the 1:4 bead standard: 1891 (invention), ~1920 (Kojima), 1930/1940s
  (Wikipedia), 1935–44 (Japanese Wikipedia).
- Bead and rod dimensions: no standard found; estimate ~14 mm pitch for a 23-rod 33 cm
  instrument.
- 全珠連 per-level digit counts: not published on its exam pages (only subjects, time,
  pass marks).
- Flash anzan "タイム" unit on flash-anzan.jp is unlabelled; interpreted as total seconds.
- Fastest flash anzan: 1.60 s (2015, Christmas Cup) vs Guinness 1.61 s (2024) — different
  sanctioning bodies.
- Barner et al. 2016 sample: 204 (abstract) vs 183 (BPS digest).
- Heffelfinger's "Abacus: Mystery of the Bead" (webhome.idirect.com/~totton/abacus/) was
  unreachable (expired TLS certificate); archived copies of its pages are listed in the
  Wikibooks bibliography below.

---

## Sources

- [Wikipedia-Soroban] Soroban — https://en.wikipedia.org/wiki/Soroban
- [Wikipedia-MA] Mental abacus — https://en.wikipedia.org/wiki/Mental_abacus
- [ja-Wikipedia-そろばん] そろばん — https://ja.wikipedia.org/wiki/そろばん
- [ja-Wikipedia-珠算] 珠算 — https://ja.wikipedia.org/wiki/珠算
- [ja-Wikipedia-フラッシュ暗算] フラッシュ暗算 — https://ja.wikipedia.org/wiki/フラッシュ暗算
- [Kojima] Takashi Kojima, *The Japanese Abacus: Its Use and Theory*, Tuttle 1954 — full text: https://archive.org/stream/japaneseabacus00taka/japaneseabacus00taka_djvu.txt ; record: https://archive.org/details/japaneseabacus00taka
- Kojima, *Advanced Abacus: Japanese Theory and Practice*, Tuttle 1963 (not fetched).
- [Heffelfinger] Totton Heffelfinger, *Abacus: Mystery of the Bead* — https://webhome.idirect.com/~totton/abacus/ (unreachable at time of writing); archived sub-pages listed at https://en.wikibooks.org/wiki/Traditional_Abacus_and_Bead_Arithmetic/Bibliography
- Wikibooks, *Traditional Abacus and Bead Arithmetic / Division* — https://en.wikibooks.org/wiki/Traditional_Abacus_and_Bead_Arithmetic/Division
- [sorobanexam-basics] https://www.sorobanexam.org/basics/soroban.html
- [sorobanexam-add] https://www.sorobanexam.org/basics/add.html
- [sorobanexam-multiply] https://www.sorobanexam.org/basics/multiply.html
- [sorobanexam-divide] https://www.sorobanexam.org/basics/divide.html
- [sorobanexam-training] https://www.sorobanexam.org/training.html ; anzan trainer https://www.sorobanexam.org/anzan.html
- [shuzan.jp-kentei] 日本珠算連盟 珠算能力検定試験 — https://www.shuzan.jp/kentei/shuzan
- [shuzan.jp-anzan] 日本珠算連盟 暗算検定試験 — https://www.shuzan.jp/kentei/anzan
- [shuzan.jp-dani] 日本珠算連盟 段位認定試験 — https://www.shuzan.jp/kentei/dani
- [shuzan.jp-flash] 日本珠算連盟 フラッシュ暗算検定 — https://www.shuzan.jp/kentei/flash/index.html
- 日本珠算連盟 検定試験問題見本 (sample papers per level) — https://www.shuzan.jp/kentei/mihon
- 日本商工会議所 検定 portal — https://www.kentei.ne.jp/abacus
- [soroban.or.jp-exam] 全国珠算教育連盟 珠算検定試験 — https://www.soroban.or.jp/exam/shuzan/
- [soroban.or.jp-anzan] 全珠連 暗算検定試験 — https://www.soroban.or.jp/exam/anzan/
- [soroban.or.jp-flash] 全珠連 フラッシュ暗算検定 — https://www.soroban.or.jp/exam/flash/
- [flash-anzan.jp] 日本フラッシュ暗算検定協会 — https://www.flash-anzan.jp/ ; about: https://www.flash-anzan.jp/about/
- [flash-anzan.jp-school] exam ladder table — https://www.flash-anzan.jp/school/
- [AppStore-official] フラッシュ暗算 公式 (Japan Flash Anzan Association) — https://apps.apple.com/jp/app/id421826349
- [Guinness] Fastest flash mental arithmetic — https://www.guinnessworldrecords.com/world-records/fastest-flash-mental-arithmetic
- [Waseda] Waseda Weekly on flash anzan records (2016) — https://www.waseda.jp/inst/weekly/attention/2016/06/21/8369/
- [coeteco 12690] そろばんの基本・補数の説明 — https://coeteco.jp/articles/12690
- [coeteco 12692] 幼児へのそろばんの教え方 (finger rules, ages) — https://coeteco.jp/articles/12692
- [coeteco 12694] そろばんの選び方 (rod counts, materials) — https://coeteco.jp/articles/12694
- [coeteco 12704] 級・段位の体系 — https://coeteco.jp/articles/12704
- [coeteco 12711] かけ算 両落とし — https://coeteco.jp/articles/12711
- [coeteco 12712] フラッシュ暗算 — https://coeteco.jp/articles/12712
- [Miyagi] 宮城県教育委員会 小4算数 そろばん教材 — https://www.edu-c.pref.miyagi.jp/tangen/math/ele4_math/answer/17.els4.kaitou.05.5stp.pdf
- [MEXT] 文部科学省 点字教材 そろばん指導資料 — https://www.mext.go.jp/content/20240327-mxt_tokubetu01-000034750_19.pdf
- [hugkum] そろばんは何歳から — https://hugkum.sho.jp/146315
- [rurubu] 習い事アンケート (attendance durations) — https://kids.rurubu.jp/article/234643/
- Frank & Barner 2012, *J. Exp. Psychol. General* 141(1):134–149 — https://pubmed.ncbi.nlm.nih.gov/21767040/ ; open copy https://escholarship.org/uc/item/8r80c5j5
- Barner et al. 2016, *Child Development* 87(4):1146–1158, doi:10.1111/cdev.12515 — preprint https://dash.harvard.edu/bitstream/handle/1/41364831/Barneretal.inpress.pdf ; digest https://bps.org.uk/research-digest/teaching-children-ancient-mental-abacus-technique-boosted-their-maths-abilities
- Barner et al. 2018, *J. Numerical Cognition* 3(3):540–558 — https://www.psycharchives.org/handle/20.500.12034/1271 ; https://jnc.psychopen.eu/index.php/jnc/article/download/5761/5761.pdf/
- Lo & Andrews 2022, *J. Numerical Cognition* 8(1):89–122, doi:10.5964/jnc.8073 — https://jnc.psychopen.eu/index.php/jnc/article/download/8073/8073.pdf/
- Tanaka et al. 2012, *Frontiers in Psychology* 3:315 — https://pmc.ncbi.nlm.nih.gov/articles/PMC3428809
- Du et al. 2013, abacus training fMRI in Chinese children — https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3830782/
- Wang et al. 2015, abacus training and task switching — https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4596702/
- Hatano & Osawa 1983, "Digit memory of grand experts in abacus-derived mental calculation", *Cognition* 15:95–110 (cited via Lo & Andrews 2022).
- Stigler 1984, "Mental abacus: the effect of abacus training on Chinese children's mental calculation", *Cognitive Psychology* 16:145–176 (cited via Lo & Andrews 2022 and PMC3830782).
- Nintendo, *Prof. Miyamoto's Soroban & Flash Anzan* (third-party trainer parameters) — https://www.nintendo.com/store/products/prof-miyamotos-soroban-and-flash-anzan-switch
