# Abax trainer design research

Research notes for a soroban (Japanese abacus) drill trainer: what existing apps do, what learning
science says a skill trainer should do, a concrete adaptive scheduler, bead-UI presentation numbers,
and accessibility/ergonomics. Compiled 2026-10. Claims are tied to the URLs in the Sources list;
where a claim is a design recommendation rather than a sourced fact it is marked *(design choice)*.

Caveat on review data: Google Play pages and Reddit could not be fetched by the research tooling, so
store-review quotes below come from the Apple App Store, aggregator snapshots (appfollow, mwm.ai), the
Art of Memory forum and blogs. Treat the Android side as under-sampled.

---

## 1. Survey of existing soroban / abacus trainers

### 1.1 Web

**sorobanexam.org** (free, browser, FR/EN/PT, light/dark toggle)
- Does well: step-by-step tutorials for the four operations on a "virtual soroban"; a *Training*
  page with exercises organised by bead-move type (plain moves, heaven-bead moves,
  complement-to-5, complement-to-10, subtraction with borrow) and an "Include previous exercises"
  option; an exam-style worksheet generator (PDF + HTML + interactive answer checking, with daily
  or weekly email delivery); an *Anzan* page with Count / Digits / Timeout / Flash settings, speech
  synthesis, Replay, full-screen and a "continuous" no-answer mode.
- Pedagogical stance worth copying: the Training page says the goal is to learn which beads to move
  "almost instinctively", not to compute mentally and then set the result.
- Interaction: beads are *clicked* (no drag). Answer checking is a manual "Check" button; the
  page does not describe what is shown on error (no "correct move" replay).
- Gaps: no adaptive difficulty, no per-skill progress tracking, no timer in training, kyu levels
  appear only in worksheet presets (e.g. "Moritomo 8th kyu", "Kojima 5th kyu").

**drillyourskill.com "Chain Math (Flash Anzan)"** (free, browser, optional paid Supporter badge)
- Settings: # Digits, Length (count of numbers), Time (sec) per number, Addition-only vs
  Addition/Subtraction, beep + voice toggles, a "Generate Optimal Settings" button, "Restart with
  the same sequence". Weekly leaderboard per digit setting (resets Sunday 23:59:59 UTC).
- Gaps: no abacus, no per-error correction, leaderboard-driven rather than mastery-driven.

**Totton Heffelfinger & Gary Flom, "Abacus: Mystery of the Bead"** (free tutorial site, long the
standard reference recommended on the Art of Memory forum; covers technique, +, −, ×, ÷, negative
numbers). Useful as a source for the canonical skill order; the site's certificate was expired at the
time of research so fetch by hand.

### 1.2 iOS

**Soroban Training** (Jumpei Adachi; free + $2.99 Remove Ads; iOS 18+; EN/JA; 4.7 from 179 ratings)
- Does well: on-screen abacus plus a "real soroban" mode that only generates problems; records
  times, best times and dates with line graphs; Game Center leaderboards; haptic feedback (added
  2023); dictation sounds even in silent mode; add/subtract two-number modes (Nov 2025); option to
  hide the stopwatch; a "Show Abacus Number" toggle.
- Complaints: "a bit laggy sometimes" at the end of a run and a request for more colour options
  (2025 paid user); a 2022 request to see the correct answer "for a few seconds before the next
  screen"; a fix for "rod highlights not disappearing"; a Jun 2026 fix for "delayed touch
  response". Banner and occasional pop-up ads in the free tier.
- Highlights rods but does not, per the listing, show the correct bead move after an error. The
  listing does not say whether input is tap or drag. This is the closest match to a "Soroban
  Trainer" app name.

**Abacus - Simple Soroban Abacus** (Switt Kongdachalert; free; iPad-designed; 2.6/5 from 10
ratings on the US store, 3.1/5 from 20 on appfollow)
- A bare abacus: "swipe up/down on the beads", "shake to reset". A later update added "Can now
  slide finger across to move beads; smoother operation", implying the original per-bead swipe was
  felt as clumsy. No lessons, drills, sound or feedback. Low rating is the cautionary tale for
  shipping a bare simulator.

**AbacusPro - Mental Maths** (Sree Sai Ventures; subscription tiers Level 1 / Level 2 / both;
released Jul 2026; 5.0 from 2 ratings)
- Notable: user-selectable **drag or tap** bead interaction styles; animated step-by-step lessons
  with voice in EN/HI/JA/KO; "Watch" mode animates each step with commentary; "Mix It Up"
  randomised practice across chosen topics; up to three learner profiles. No adaptive difficulty
  mentioned.

**Flash Mental Math (Anzan)** (SIC Company; free, $0.99/month removes ads and unlocks Custom Mode)
- 40 grades from 20 Kyu to 20 Dan, 10 questions per round, 7 correct passes; 2,100 stages across
  1-6 digits with "350 steps per digit" where count and speed increase gradually; Custom Mode:
  digits 1-7, count 2-30, seconds per number in 0.1 s steps; four text colours, two fonts, light
  and dark modes; audio in sync with each number; five rankings including a login streak.
- Does well: a fine-grained, pre-authored difficulty ladder. Gaps: not adaptive, no error
  diagnosis, streak-type engagement.

**SimplySoroban**, **Soroban66** ($3.99 iPad, flash anzan drills), **Mental Math Speed Trainer**
(rank-based flash anzan), **MiSoroban** (€1.99/month; levels by age range, video tutorials, daily
goal, rankings, background music toggle), **SumzUp Soroban Abacus Lessons** (dyscalculia focus),
**Know Abacus** (Kittipong Mungnirun/Lookkid; iOS, Android, Chromebook; counting, 1-2 digit +/−,
×, ÷, mental math, and a **left-handed mode**; a visual-impairment-education blog called it "the
best abacus app I've found so far" but "most helpful for teachers"; ads avoidable in airplane mode).

**Cotter AL Abacus** ($1.99; 3.5 from 6 ratings; a different abacus type but the reviews are the
clearest touch-UI complaints found): "my daughter had trouble touching the beads in just the right
spot to move them. I ended up moving them for her"; "takes for! Ever! For it to recognize your
finger"; "The beads only go to the far left and far right". Lessons: hit areas must be generous,
recognition must be immediate, and bead positions must match what the book/teacher shows.

### 1.3 Android

- **Soroban: Abacus & Mental Math** (SIC Company, jp.sicorp.abacusga; free; 1,000+ downloads;
  video lessons "supervised by a Japan soroban dojo"). **Simple Abacus Speed Practice** and
  **Anzan - Abacus Flash Cards** (Rainbow Creators; adjustable speed, digits, question count).
  Review text could not be retrieved.

### 1.4 Console

- **Prof. Miyamoto's Soroban and Flash Anzan** (Nintendo Switch; free download, "Period Ticket"
  needed for full use). Flash anzan practice: up to 4 digits, up to 30 numbers, time settable in
  0.01 s units, minimum 1.35 s per question or 0.045 s per number; exam mode follows the Nihon
  Flash Anzan Examination Association standards, Grade 20 to Level 20. Touch input supported.

### 1.5 Feature matrix (from listings; "?" = not stated)

| App | Platform | Price | Adaptive | Shows correct move on error | Drag vs tap | Sound/haptics |
|---|---|---|---|---|---|---|
| sorobanexam.org | web | free | no | no (Check button only) | click | speech (anzan) |
| Chain Math (drillyourskill) | web | free (+badge) | "optimal settings" button only | no | n/a | beep + voice |
| Soroban Training | iOS | free + $2.99 | no (user picks mode) | no; rod highlight only | ? | sound + haptics |
| Simple Soroban Abacus | iOS | free | no | no | swipe / slide | ? |
| AbacusPro | iOS | subscription | no | animated lesson steps | drag **or** tap (user choice) | voice |
| Flash Mental Math | iOS | free + $0.99/mo | staged ladder, not adaptive | no | n/a | synced audio |
| Know Abacus | iOS/Android/Chromebook | free + ads | no | ? | ? | ? |
| MiSoroban | iOS | subscription | levels by age | ? | ? | background music |
| Switch Soroban & Flash Anzan | Switch | free + ticket | no | ? | touch | ? |

### 1.6 Recurring complaints and praise, distilled

- Touch precision and latency dominate negative reviews ("right spot", "takes forever to
  recognize", "laggy", "delayed touch response").
- Users want to *see the answer* before the next item (Soroban Training request) and to see
  *how* beads should have moved (no surveyed app does this in drill mode; only lesson animations).
- Ads and subscription gating of basic settings (Custom Mode behind a monthly fee) irritate.
- Praised: recorded times with graphs, customisable drills, "real soroban" problem-only mode,
  haptics, choice of drag/tap, multiple profiles, left-handed mode, calm/no-ads presentation.
- Nobody surveyed does adaptive difficulty from measured performance; ladders are hand-authored
  (350 steps per digit) or user-chosen. This is the open gap for Abax.

---

## 2. Learning-science principles for a skill trainer

### 2.1 Immediate, informative feedback (show the move, not just "wrong")
- Pashler, Cepeda, Wixted & Rohrer (2005): for cued recall, giving the **correct answer** after an
  error increased final retention (the abstract reports "494%" relative to no feedback, i.e. the
  no-feedback baseline was very low), while mere right/wrong feedback was "no more effective than
  receiving no feedback". Fazio et al. (2010) replicated: right/wrong feedback "did not aid error
  correction". For Abax: on a wrong answer, replay the correct bead sequence on the rods, not a
  red X.
- Butler et al. (2013): explanation feedback beats correct-answer feedback for transfer. For Abax:
  name the rule ("add 7: take 3 from the ones, add 1 to the tens") alongside the replayed move.
- Timing: Kulik & Kulik (1988; 53 effects, d = 0.28 in Hattie & Timperley's table) found delayed
  feedback can win in laboratory verbal-learning tests at 5-7 day retention, but "applied
  classroom studies have usually favored immediate feedback". For a motor-procedural skill drilled
  item by item, immediate is the right default; a delayed end-of-set summary is a cheap addition
  (and what a Soroban Training reviewer asked for).
- Wisniewski, Zierer & Hattie (2020): feedback effect d = 0.48 across 435 studies, "substantially
  influenced by the information content conveyed".

### 2.2 Deliberate practice (Ericsson, Krampe & Tesch-Römer 1993)
- Deliberate practice = "highly structured activities ... created specifically to improve
  performance ... through immediate feedback", described as effortful and "not inherently
  enjoyable". Expert soroban use is in this tradition (dojo drills, kyu/dan exams). Implications:
  target the weakest sub-skill, keep items just beyond current ability, keep sessions short enough
  to sustain full concentration. (Hambrick et al. 2014 and a 2019 replication qualify the *size*
  of the effect, not the direction.)

### 2.3 Desirable difficulties and interleaving
- Bjork & Bjork (2011): difficulties are desirable only when the learner "has the background
  knowledge or skills to respond to them successfully"; examples: spacing, varying conditions of
  practice, testing instead of re-presenting. Translate as: never raise difficulty on a skill whose
  prerequisites are unmastered; prefer *doing* over *watching* once the lesson is understood.
- Rohrer & Taylor (2007), *Instructional Science* 35: shuffled (interleaved) maths practice was
  "vastly superior" one week later versus blocked practice, even though blocked practice looks
  better during the session. Taylor (2008): 77% vs 38% on novel problems. For Abax: a focused
  drill should still mix in ~30% items from other unlocked skills, and the set after a new skill
  should be mixed.
- Cepeda et al. (2006; 317 experiments): spacing helps, and the optimal gap grows with the desired
  retention interval. Short daily sessions beat one long weekly one.

### 2.4 Mastery-based progression
- Bloom (1984): tutored + mastery-learning students outperformed conventional classes by about two
  standard deviations; "70% of mastery learning students reached the level only the top 20% of
  conventional students reached". The operational rule is "do not advance until the current unit
  is demonstrably mastered, then remediate exactly what failed". This is the backbone of the
  scheduler in section 3.

### 2.5 Flow channel and target success rate
- Csikszentmihalyi: flow lies in the channel between boredom (skill > challenge) and anxiety
  (challenge > skill). Engeser & Rheinberg (2008) found the challenge-skill balance "only partially
  supported" as a predictor; a 2014 meta-analysis found a moderate relationship. So use the
  channel as a heuristic, not a law.
- Empirical targets: Math Garden samples items at a mean success probability of **.75**
  (Klinkenberg et al. 2011). Jansen et al. (2013; 207 children, six weeks) compared pre-set success
  rates and found "the higher the pre-set success rate, the more problems were attempted" and that
  practice volume mediated gains. Lomas et al. (2013; CHI; ~10K and ~70K players) found *no*
  inverted-U: easier was more engaging, yet "the most engaging conditions produced the slowest
  learning". Pelánek et al. (geography facts) found harder items better for learning facts.
- Design resolution *(design choice)*: aim for ~80% success on the focus skill and ~90% on
  warm-up/review; let advanced users opt into a 70% "hard mode", because Lomas also found moderate
  difficulty most motivating "only when players chose their own level".

### 2.6 Accuracy first, then speed
- Newell & Rosenbloom (1981): time per item falls as a power (or per-individual exponential,
  Heathcote et al.) function of practice. Speed comes from repetition after the procedure is
  correct; drilling speed on a wrong procedure entrenches the error. Pelánek et al. report that
  after a *correct* answer the probability of the next answer being correct runs from ~95% for very
  fast answers to ~80% for slow ones, i.e. response time is a usable signal of fluency. For Abax:
  gate timers behind accuracy, then tighten.

### 2.7 Short sessions, no shaming, momentum instead of streaks
- Duolingo's retention team has run "over 600 experiments" on streaks and regards them as its most
  important lever, but Silverman et al. (2023, *Journal of Consumer Research*, via The Decision
  Lab) found "users often view extending their streak as more important than engaging in the
  underlying activity" and that users who break streaks "are more likely to stop using the platform
  entirely"; paid repair lowers motivation to keep the streak. Guilt copy ("You made Duo sad") is
  singled out as the failure mode. Duolingo's own learning: requiring "one completed lesson a day"
  was clearer than XP-based streaks; letting users choose their own goal duration "significantly
  increased retention"; the first seven days are where retention is most fragile.
- The dev.to write-up of a SwiftUI soroban app frames progress as "private encouragement rather
  than a public leaderboard" and avoids "making every interaction feel like a test".
- Abax policy *(design choice)*: report **momentum** (items solved this week, skills advanced,
  best-ever speed per skill, days practised out of the last 14 as dots, never a counter that
  resets to zero); no loss-framed notifications; sessions of 5-8 minutes; end each session on a
  success (a review item at the 90% tier).

---

## 3. Adaptive scheduler for a short-session drill trainer

### 3.1 Skill graph (prerequisites)

Order follows the sorobanexam.org training ladder and the Totton tutorial. Each node is a
*skill*; each drill item carries exactly one *focus skill* plus difficulty parameters.

```
S0  set/read a digit on one rod              (prereq: none)
S1  + simple (no complement, one rod)        (S0)
S2  − simple                                 (S0)
S3  + five-complement ("small friend")       (S1)
S4  − five-complement                        (S2, S3)
S5  + ten-complement ("big friend", carry)   (S3)
S6  − ten-complement (borrow)                (S4, S5)
S7  + / − mixed complements (e.g. +6..+9 across 5 and 10)   (S5, S6)
S8  multi-digit addition chains              (S7)
S9  multi-digit mixed +/− chains             (S8)
S10 multiplication (1x1 → 2x1 → 2x2)         (S8)
S11 division                                 (S10)
A1  anzan: visualised 1-digit chains, untimed (S7)
A2  flash anzan 1-digit, timed               (A1, S9)
A3  flash anzan 2+ digits                    (A2)
```

A skill is **unlocked** when every prerequisite has `mastery ≥ 0.80` (not necessarily the 0.95
"mastered" flag), so learners meet the next idea while the previous one is still being polished
(Bjork's "background knowledge" condition).

### 3.2 Per-skill state

```rust
struct SkillState {
    p_known: f64,      // BKT P(L), init 0.10
    acc_ewma: f64,     // exponentially weighted accuracy, init 0.50
    log_t_ewma: f64,   // EWMA of ln(seconds per item), init ln(benchmark*3)
    theta: f64,        // Elo skill, init 0.0
    n: u32,            // attempts on this skill
    level: u8,         // current difficulty step on the skill's ladder, init 0
    last_practised: Timestamp,
    mastered_at: Option<Timestamp>,
    review_interval_days: f64,  // init 1.0
}
```

### 3.3 Update rules (per answered item, correct ∈ {0,1}, t = seconds)

**Bayesian Knowledge Tracing** (Corbett & Anderson 1995), parameters for Abax *(design choice,
in the range used in tutoring systems)*: `p_L0 = 0.10`, `p_T = 0.10`, `p_G = 0.20`, `p_S = 0.10`.
Guess is low because the learner types a full number (not multiple choice); slip covers mis-taps.

```
if correct:  post = L*(1-p_S) / (L*(1-p_S) + (1-L)*p_G)
else:        post = L*p_S     / (L*p_S     + (1-L)*(1-p_G))
L_next = post + (1-post)*p_T
```

Worked values to unit-test against (start L = 0.10): four consecutive corrects give
0.400, 0.775, 0.9455, 0.9886; one error after that drops to 0.924; one more correct returns to
0.984 (values recomputed during implementation; an earlier draft said 0.907/0.978). Mastery flag requires `L ≥ 0.95` (the conventional threshold) **and** `n ≥ 10` **and** the
speed gate below, so four lucky answers cannot "master" a skill.

**Accuracy EWMA**: `acc ← acc + 0.2*(correct − acc)`. Ten consecutive corrects from 0.5 give
0.946; effective window ≈ 10 items. Use `acc` for fast up/down-shifts within a session.

**Time EWMA on log seconds** (robust to one slow outlier): `lt ← lt + 0.3*(ln t − lt)`. Only
update on correct answers (errors are not fluency evidence). Report `exp(lt)` as the user's typical
time.

**Elo per skill with item difficulty** (Pelánek et al.; Klinkenberg et al. 2011):

```
P = σ(theta − d)                         σ(x) = 1/(1+e^-x)
theta ← theta + U(n_skill) * (correct − P)
d     ← d     + U(n_item)  * (P − correct)      // only if items are stored and shared
U(n) = α / (1 + β n),  α = 1.0, β = 0.06       // Pelánek et al. grid-search optimum
```

For a generated-item trainer, derive `d` from the generator instead of learning it:

```
d(item) = base[skill] + 0.40*(digits−1) + 0.25*(operands−2) + 0.50*[timed] + 0.30*[flash]
base: S1,S2 = −1.0; S3,S4 = 0.0; S5,S6 = 0.5; S7 = 1.0; S8 = 1.5; S9 = 2.0; A1 = 1.5; A2 = 2.5
```

Unit test: `theta = 0, d = 0, n = 0` → `P = 0.5`; a correct answer moves `theta` to `+0.5`; at
`n = 10`, `U = 0.625`.

**Response-time-aware scoring (optional, for timed modes)**: Math Garden's "high speed, high
stakes" rule (Klinkenberg et al. 2011; Maris & van der Maas) replaces `correct` with
`s = (2*correct − 1) * (T_limit − t) / T_limit`, so a fast correct is ≈ +1, a slow correct ≈ 0,
a fast wrong ≈ −1, a slow wrong ≈ 0. Use `P_s = 2P − 1` in the Elo update when scoring this way.
It discourages guessing fast.

### 3.4 Difficulty ladder per skill

Each skill has an ordered list of `(digits, operands, time_mode, presentation)` steps. Raise **one
dimension at a time**, in the order: operands → digits → time pressure → presentation.

```
S1 ladder:  (1,2,untimed,beads) → (1,3,untimed,beads) → (1,5,untimed,beads)
            → (1,5,gentle,beads) → (1,8,gentle,beads) → (1,8,tight,beads)
S8 ladder:  (2,2,untimed) → (2,3) → (2,5) → (3,3) → (3,5) → (2,5,gentle) → (3,5,gentle) → (3,10,tight)
A2 ladder:  per-number display 1.5 s → 1.2 → 1.0 → 0.8 → 0.6 → 0.5 → 0.4 (then digits ++)
```

Time budgets *(design choice; tune with telemetry)*:
`gentle = 2.0 × user's exp(lt)`, `tight = 1.3 × exp(lt)`, floor 1.2 s per operand for bead
entry. Never introduce a timer while `acc_ewma < 0.90` at the current untimed step.

**Step up** when all hold: `acc_ewma ≥ 0.85`, `P(correct | theta, d_current) ≥ 0.85`, at least 8
items at this step, and (if timed) `exp(lt) ≤ 0.8 × budget`.
**Step down** when: `acc_ewma < 0.60` over the last 6, or two consecutive errors on the same rule
within one drill (fast fallback; also triggers a one-item guided replay).

### 3.5 Readiness to advance (mastery gate)

```
mastered(skill) :=
    p_known ≥ 0.95
 && n ≥ 10
 && acc_ewma ≥ 0.90
 && level ≥ ladder_len(skill) − 2            // reached at least the first timed step
 && exp(log_t_ewma) ≤ 1.5 × benchmark[skill] // fluency gate
```

Benchmarks *(initial design values, seconds per operand on touch)*: S1-S2 2.5, S3-S4 3.0, S5-S7
3.5, S8 (2-digit) 5.0, A1 2.0 per number, A2 = current display time. Replace with population medians
once telemetry exists.

### 3.6 Choosing the next drill (3 drills per 5-8 min session, 8-15 items each)

Decay first (forgetting), then score candidates.

```
days = (now − last_practised).days
p_eff = p_known × exp(−days / τ)       τ = 14 × review_interval_days
```

Review scheduling is expanding: when a *mastered* skill is reviewed with `acc ≥ 0.9`,
`review_interval_days ← min(review_interval_days × 1.6, 60)`; on `acc < 0.7` reset to 1.0 and
clear `mastered_at`. A mastered skill is **due** when `p_eff < 0.80` or `days ≥ review_interval_days`.

Per-session plan:
1. **Warm-up** (6-8 items): the due review skill with the highest `days`; if none due, the most
   recently mastered skill at its top ladder step. Target success ≈ 0.90. Purpose: an easy start.
2. **Focus** (10-15 items): the unlocked, unmastered skill with the lowest `p_eff`, chosen with
   probability 0.7; with probability 0.3 the *second* lowest (variety; avoids hammering one skill
   for days). 70% of items are from the focus skill at its current step; 30% are interleaved from
   other unlocked skills at their current step (Rohrer & Taylor). Target success ≈ 0.80.
3. **Cool-down** (5-8 items): mixed set from all skills with `p_eff ≥ 0.8`, untimed, ending on an
   item whose `P ≥ 0.9`.

Item-level selection inside a drill, adapted from Pelánek et al.'s weighted score:

```
score(item) = 10 * S_prob + 10 * S_count + 120 * S_time
S_prob  = P_est/P_target            if P_target ≥ P_est
        = (1−P_est)/(1−P_target)    otherwise
S_count = 1 / sqrt(1 + times_seen(item_class))
S_time  = −1 / seconds_since_last_seen(item_class)   // avoid short-term repeats
```

where `item_class` is e.g. "+7 on a rod showing 8". Pick the top-scoring item; break ties randomly.

### 3.7 First-run and cold start
- Placement: offer "I'm new" (start at S0 with lesson), or a 12-item mixed probe over S1-S7. For
  each skill probed with ≥ 2 items, set `p_known = clamp(fraction_correct, 0.1, 0.9)` and
  `theta = ln(p/(1−p))`. Everything else stays at defaults.
- Pelánek et al.'s finding that Elo with `U(n)` gives "fast, rough estimates after a few answers
  and stability in the long run" is the reason not to need a longer placement test.

### 3.8 Unit-test checklist
- BKT sequence above (0.400 / 0.775 / 0.9455 / 0.9886 / 0.924 / 0.984) to 3 d.p.
- EWMA: 10 corrects from 0.5 → 0.9463; 3 errors from 0.9 → 0.4608.
- `U(0) = 1.0`, `U(10) = 0.625`, `U(100) = 0.142857`.
- Elo monotonicity: a correct answer never lowers `theta`; an incorrect one never raises it.
- Mastery gate rejects 4-for-4 with `n = 4`; accepts 10-for-10 at `level ≥ len−2` with time OK;
  rejects 10-for-10 when `exp(lt) > 1.5 × benchmark`.
- Step-down fires on two consecutive same-rule errors; step-up does not fire while timed step has
  `exp(lt) > 0.8 × budget`.
- Forgetting: `p_known = 0.96`, 14 days, `review_interval_days = 1` → `p_eff = 0.353` (due);
  `review_interval_days = 10` → `τ = 140`, `p_eff = 0.869` (not due by `p_eff`, due by interval).
- Session planner never picks a locked skill; never picks the same focus skill as yesterday when
  another unlocked skill has `p_eff` within 0.05.

---

## 4. Presentation notes for the bead UI

### 4.1 Animation durations
- Material (M1 motion spec): mobile transitions "typically" 300 ms, entering 225 ms, leaving
  195 ms, "transitions that exceed 400ms may feel too slow", desktop 150-200 ms, tablet ~30%
  longer, wearables ~30% shorter; shorter durations "when objects travel short distances". Material
  component tokens run 50-500 ms in 50 ms steps; standard easing `cubic-bezier(0.2, 0, 0, 1)`,
  decelerate `(0, 0, 0, 1)`, accelerate `(0.3, 0, 1, 1)`.
- A bead travels a short distance (one bead height), so *(design choice)*:
  - user-initiated bead move: **80-120 ms**, decelerate easing, or a critically damped spring
    (stiffness ~600-900, damping ratio ~1.0) so the bead never overshoots the frame;
  - beads pushed as a group (e.g. clearing a rod): stagger 15-25 ms per bead, total ≤ 300 ms;
  - "show the correct move" replay: **350-450 ms per move** with a 250 ms pause between moves,
    then hold the final state ≥ 1.5 s or until tap (a Soroban Training reviewer asked for "a few
    seconds to view the answer");
  - reset/clear: ≤ 250 ms total; respect reduced-motion (section 5).
- Honour `prefers-reduced-motion` / platform reduce-motion: collapse to ≤ 80 ms or instant.

### 4.2 Hit areas and touch recognition
- Apple HIG: minimum tappable area 44 × 44 pt. Material: 48 × 48 dp with ≥ 8 dp between
  targets ("about 9 mm"). WCAG 2.2 2.5.8 (AA): 24 × 24 CSS px minimum; 2.5.5 (AAA): 44 × 44.
- Soroban beads on a phone are necessarily smaller than 44 pt, so the *hit region* must exceed the
  *drawn* bead: make each rod a full-height touch column at least 44 pt wide, and resolve a tap to
  a bead by vertical position within the column (nearest gap between beads), never by pixel-perfect
  bead bounds. On phones in portrait, show 5-7 rods; 9-13 rods only on tablet or landscape.
- Respond on `touch-down`, not `touch-up`, and never wait for a long-press or disambiguation
  timer; the "takes forever to recognize your finger" and "delayed touch response" complaints are
  recognition latency.

### 4.3 Tap vs drag semantics
- Observed practice: sorobanexam.org uses click-to-toggle; Simple Soroban Abacus started with
  per-bead swipe and added "slide finger across to move beads"; AbacusPro lets the user choose
  "drag or tap". The low-rated bare simulator is the one that only supported swipe.
- Recommended *(design choice)*: support both, with tap as default.
  - **Tap** on an earth bead: move that bead and every bead between it and the beam toward the
    beam (if it is away) or away (if it is at the beam), i.e. tap sets the digit implied by that
    bead. Tap the heaven bead toggles it. This matches how physical users flick several beads with
    one stroke.
  - **Drag**: beads follow the finger with the group constraint; commit on release past 50% of the
    travel; a flick (velocity > ~600 pt/s) commits regardless of distance. Horizontal slide across
    columns while touching should sweep (the "slide finger across" fix).
  - Expose a setting: `Tap only` / `Drag only` / `Both`, because advanced users complain when a
    tap intended as a flick is interpreted as a drag start.
- Provide a "clear" gesture (two-finger swipe or dedicated button); do not rely on shake.

### 4.4 Haptics and sound
- Android: "less is more"; use **clear** haptics ("crisp and clean sensations associated with a
  discrete event, such as button presses") via `HapticFeedbackConstants`; a keyclick-type signal
  should last **10-20 ms**; avoid one-shot `vibrate(long)` because it feels "buzzy"; given the
  choice between buzzy and none, "choose no haptics"; for drag/snap "gradually increase the
  amplitude of a sequence of ticks" toward the target; keep haptics in sync with visuals and audio.
- Apple: use system patterns for their documented meanings (selection tick for bead-to-bead,
  light/medium impact for a bead hitting the beam, notification success/error for answer results);
  keep a "clear, causal link" between haptic and action; make haptics optional.
- Sound *(design choice)*: one short wooden "tick" (≤ 40 ms) per bead settle, pitch slightly
  varied per rod to avoid machine-gun monotony; a soft two-note chime for correct, a low single
  note (not a buzzer) for incorrect; number dictation voice for anzan (sorobanexam.org, Soroban
  Training and Flash Mental Math all offer audio). Mute switch + separate toggles for ticks,
  results, and voice.

### 4.5 Highlighting "move this bead"
- Show the **target state**, not only the bead: outline the destination gap with a dashed ring,
  draw a short arrow in the direction of travel, and pulse the bead itself (scale 1.0 → 1.08,
  600 ms, 2 cycles). Pulsing alone fails for colour-blind users and for users who have reduced
  motion on; the dashed ring and arrow carry the meaning without colour or motion.
- For rule explanation, highlight the two rods involved in a complement (ones and tens) with a
  shared label ("+7 = +10 − 3") above the beam.
- After an error, replay the correct sequence at 350-450 ms per move (section 4.1), then let the
  user redo the item (do not auto-advance).

### 4.6 Colour-blind-safe highlight choices
- Okabe & Ito's Color Universal Design palette (popularised by Wong, *Nature Methods* 2011):
  orange `#E69F00`, sky blue `#56B4E9`, bluish green `#009E73`, yellow `#F0E442`, blue `#0072B2`,
  vermilion `#D55E00`, reddish purple `#CC79A7`, black `#000000`.
- Their rules: never red vs green, avoid green/brown, red/brown, blue/violet, pink/sky-blue, and
  thin blue-on-yellow; prefer vermilion over pure red (pure dark red reads as near-black to
  protanopes); alternate warm and cool colours and make brightness clearly different when two
  colours are both warm or both cool; "do not convey information by colour alone" - add shape,
  position or label.
- Abax palette *(design choice)*: "move this bead" = sky blue `#56B4E9` ring + arrow; "correct"
  = bluish green `#009E73` with a check glyph; "incorrect / this bead is wrong" = vermilion
  `#D55E00` with an x glyph and a dashed outline. Keep bead body colour neutral (wood tones) and
  reserve the three hues for state. Verify 3:1 non-text contrast of ring vs background (WCAG 1.4.11).

---

## 5. Accessibility and ergonomics

### 5.1 Keyboard entry (desktop / external keyboard)
- The calculator layout (7-8-9 on top) is the computer numeric-keypad standard and dates to a 1914
  Sundstrand patent; the telephone layout (1-2-3 on top) came from Bell Labs human-factors research
  (Deininger 1960 found the calculator layout about 3% slower for phone dialing). On screen, the
  **phone layout** is what every mobile number pad shows, so use 1-2-3 on top for the on-screen
  pad and accept both `Digit0-9` and `Numpad0-9` from hardware keyboards.
- Conventions to implement *(design choice, matches calculator/spreadsheet habits)*:
  `Enter` / `NumpadEnter` = submit; `Backspace` = delete last digit; `Escape` or `Delete` = clear
  entry; `-` / `NumpadSubtract` = toggle sign where negatives are allowed; `Space` = next item /
  replay the demonstration; `R` = reset the abacus; `H` = show hint (costs the item's "clean"
  flag). Rod focus with `Left/Right`; `Up/Down` moves the focused earth group; `PageUp/PageDown`
  toggles the heaven bead. Never require NumLock state the app cannot see; treat `Numpad*` keys
  by their digit meaning when the browser reports a digit.
- Show a visible focus ring (≥ 3:1 contrast, WCAG 1.4.11) on the focused rod.

### 5.2 Left-handed and one-handed layouts
- Hoober's 1,333-observation street study (UXmatters 2013): one-handed grip ~49%, cradled ~36%,
  two-handed ~15% (figures via a secondary summary). Thumb-reach heat maps mirror for left-handed
  users; for right-handers the bottom-right and top-right corners are the least accurate zones.
- Implement a **Handedness** setting (Right / Left / Auto by last-touch side) that mirrors the
  layout of the number pad, the Submit/Clear buttons and the hint button, but **does not** mirror
  the abacus or digits (the Know Abacus app ships a "left-handed mode"; patents on handed keypads
  describe mirroring target positions "except that the characters and graphics are not reversed").
- Put Submit under the thumb; put destructive Clear away from Submit by at least one target width
  (8 dp Material spacing minimum, more for these two).

### 5.3 High contrast and large text
- WCAG 2.2: text 4.5:1 (3:1 for large text, ≥ 24 px regular or ≥ 19 px bold); non-text UI
  components and graphical objects 3:1 against adjacent colours (1.4.11); text must survive 200%
  resize without loss of function (1.4.4); AAA text contrast 7:1 (1.4.6).
- Apple: honour Dynamic Type (`preferredFont(forTextStyle:)`), Increase Contrast, Button Shapes
  and Reduce Motion. Material: components pad to 48 dp automatically; the icon may be 24 dp inside
  a 48 dp target.
- Abax *(design choice)*: numerals in the problem and answer field at ≥ 32 pt by default with a
  scalable setting up to 200%; a "high-contrast beads" theme (black beads, white rods, 7:1); bead
  count labels optional under each rod (Soroban Training's "Show Abacus Number" toggle was removed
  and then re-added after user demand, so keep it).

### 5.4 Motion, sound and timing accommodations
- Reduce-motion: instant bead placement, no pulse; keep the dashed ring + arrow.
- Every timed mode must have an untimed twin and a "+50% time" accessibility multiplier (WCAG
  2.2.1 Timing Adjustable).
- Voice dictation of problems (already standard in sorobanexam.org, Soroban Training, Flash Mental
  Math) doubles as an accommodation for low vision; expose per-number speech rate.
- All feedback is triple-coded: colour + glyph + (optional) sound/haptic.

### 5.5 Children and shared devices
- Multiple profiles (AbacusPro has three; MiSoroban has teacher classes) because a household
  typically shares one tablet; keep per-profile scheduler state and handedness.
- No ads, no interstitials, no streak counters, no guilt notifications (sections 2.7).

---

## Sources

Apps and sites
- sorobanexam.org home: https://www.sorobanexam.org/
- sorobanexam.org training exercises: https://www.sorobanexam.org/training.html
- sorobanexam.org anzan: https://www.sorobanexam.org/anzan.html
- Chain Math (Flash Anzan), drillyourskill: https://drillyourskill.com/chain-math
- Soroban Training (iOS, Jumpei Adachi): https://apps.apple.com/us/app/soroban-training/id1519658576
- Abacus - Simple Soroban Abacus (iOS): https://apps.apple.com/app/id852935077
- Abacus - Simple Soroban Abacus ratings snapshot: https://apps.appfollow.io/ios/abacus-simple-soroban-abacus/852935077?country=us
- AbacusPro - Mental Maths: https://mwm.ai/apps/abacuspro-mental-maths/6783415011
- Flash Mental Math (Anzan): https://apps.apple.com/app/id6749636736
- SimplySoroban: https://apps.apple.com/pg/app/simplysoroban/id6449087338
- Mental Math Speed Trainer: https://apps.apple.com/app/id6476421167
- MiSoroban: https://apps.apple.com/de/app/misoroban/id6484265986
- SumzUp Soroban Abacus Lessons: https://apps.apple.com/app/id6743635881
- Know Abacus (iOS listing): https://apps.apple.com/ru/app/641867438
- Know Abacus review (Paths to Literacy): https://www.pathstoliteracy.org/fast-math-abacus/
- Cotter AL Abacus reviews (touch complaints): https://apps.apple.com/app/id406078581
- Soroban: Abacus & Mental Math (Android, AppBrain): https://www.appbrain.com/app/soroban-abacus-mental-math/jp.sicorp.abacusga
- Simple Abacus Speed Practice (Android): https://chrome-stats.com/d/com.rainbowcreators.abacuspeed
- Prof. Miyamoto's Soroban and Flash Anzan (Switch): https://www.nintendo.com/store/products/prof-miyamotos-soroban-and-flash-anzan-switch/
- Building a SwiftUI soroban app (dev.to): https://dev.to/soroban_app_team/building-a-swiftui-soroban-app-for-short-daily-mental-math-practice-410
- Art of Memory forum, soroban resources: https://forum.artofmemory.com/t/soroban-resource/40678
- Art of Memory forum, learning materials: https://forum.artofmemory.com/t/any-good-learning-materials-on-the-abacus-method/30038
- Totton Heffelfinger & Gary Flom, Abacus: Mystery of the Bead: http://webhome.idirect.com/~totton/abacus/

Learning science
- Ericsson, Krampe & Tesch-Römer (1993), Psychological Review 100:363-406: https://stafforini.com/works/ericsson-1993-role-deliberate-practice/
- Hambrick et al. (2014), Intelligence (qualification): https://www.gwern.net/doc/psychology/chess/2014-hambrick.pdf
- Bjork & Bjork (2011), Making things hard on yourself, but in a good way: https://bjorklab.psych.ucla.edu/wp-content/uploads/sites/13/2016/04/EBjork_RBjork_2011.pdf
- Rohrer & Taylor (2007), Instructional Science 35:481-498: https://www.gwern.net/doc/psychology/spaced-repetition/2007-rohrer.pdf
- Rohrer & Taylor (2007) repository record: https://digitalcommons.usf.edu/psy_facpub/1767/
- Cepeda et al. (2006), Psychological Bulletin, distributed practice meta-analysis: https://www.evullab.org/pdf/CepedaPashlerVulWixtedRohrer-PB-2006.pdf
- Bloom (1984), The 2 Sigma Problem, Educational Researcher 13(6): https://www.gwern.net/doc/psychology/1984-bloom.pdf
- Bloom's 2 sigma problem (overview): https://en.wikipedia.org/wiki/Bloom%27s_2_sigma_problem
- Engeser & Rheinberg (2008), Flow, performance and moderators of challenge-skill balance: https://www.uni-trier.de/fileadmin/fb1/prof/PSY/PGA/bilder/Engeser___Rheinberg_2008.pdf
- Challenge-skill balance meta-analysis (2014): https://www.instituteofcoaching.org/resources/challenge%E2%80%93skill-balance-and-antecedents-flow-meta-analytic-investigation
- Pashler, Cepeda, Wixted & Rohrer (2005), When does feedback facilitate learning of words?: https://www.yorku.ca/ncepeda/publications/PCWR2005.pdf
- Fazio et al. (2010), right/wrong vs correct-answer feedback: https://pmc.ncbi.nlm.nih.gov/articles/PMC4073309
- Butler et al. (2013), explanation feedback vs correct-answer feedback: https://bpb-us-e2.wpmucdn.com/sites.wustl.edu/dist/8/805/files/2026/06/Butler-et-al.-2013-Explanation-feedback-is-better-than-correct-answer-feedback-for-promoting-transfer-of-learning.pdf
- Hattie & Timperley (2007), The power of feedback: https://www.uky.edu/~gmswan3/575/Hattie_Timperly_2007.pdf
- Smith & Kimball (2010), on Kulik & Kulik (1988) feedback timing: https://www.ou.edu/memorylab/pdfs/SmithKimball_2010_LearningFromFeedback_ms.pdf
- Wisniewski, Zierer & Hattie (2020), The power of feedback revisited: https://pmc.ncbi.nlm.nih.gov/articles/PMC6987456
- Newell & Rosenbloom (1981) / power law of practice overview: https://en.wikipedia.org/wiki/Power_law_of_practice
- Lomas et al. (2013), Optimizing challenge in an educational game, CHI: https://www.stat.cmu.edu/~brian/463-663/project-part-one/Optimizing%20Challenge.pdf
- Jansen et al. (2013), experiencing success in math (Math Garden success rates): https://dare.uva.nl/id/789fc422-4935-49ca-bcba-803239522951

Streaks and motivation
- The Decision Lab, Streak Creep (summarising Silverman et al. 2023, JCR): https://thedecisionlab.com/insights/consumer-insights/streak-creep-the-perils-of-too-much-gamification
- Lenny's Newsletter, Behind the product: Duolingo streaks: https://www.lennysnewsletter.com/p/behind-the-product-duolingo-streaks
- Oulu thesis, Keeping the streak alive: https://oulurepo.oulu.fi/handle/10024/54117

Adaptive scheduling
- Klinkenberg, Straatemeier & van der Maas (2011), Computer adaptive practice of maths ability using a new item response model (Math Garden): https://dare.uva.nl/id/de65bb65-d198-4dbc-ac7b-bc25ca23e627
- Pelánek et al., Elo-based learner modeling for the adaptive practice of facts (UMUAI): https://www.fi.muni.cz/~xpelanek/publications/umuai-adaptive-practice.pdf
- Pelánek (2016), Applications of the Elo rating system in adaptive educational systems: https://www.fi.muni.cz/~xpelanek/publications/CAE-elo.pdf
- Vermeiren et al., dynamic K in Elo-based learning systems: https://research.tilburguniversity.edu/en/publications/balancing-stability-and-flexibility-investigating-a-dynamic-k-val/
- Corbett & Anderson (1995) BKT, parameter table in: https://arxiv.org/pdf/1803.05926
- BKT worked example and 0.95 threshold convention: https://www.mindomax.com/the-math-behind-knowledge-tracing
- BKT overview (Williams College): https://www.cs.williams.edu/~iris/res/bkt/
- Baker, Corbett & Aleven (2008), contextual guess and slip: https://www.springerprofessional.de/doi/10.1007/978-3-540-69132-7_44

UI, motion, haptics, colour
- Material motion: duration and easing: https://m1.material.io/motion/duration-easing.html
- Material motion tokens (50-500 ms, easing curves): https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md
- Material applying density (48 dp targets, 8 dp spacing): https://material.io/design/layout/applying-density.html
- Android accessibility, touch target size: https://support.google.com/accessibility/android/answer/7101858
- Apple HIG, Accessibility (44 pt targets, Dynamic Type, Reduce Motion): https://developer.apple.com/design/human-interface-guidelines/accessibility
- Apple HIG, Layout: https://developer.apple.com/design/human-interface-guidelines/layout
- Apple HIG, Playing haptics: https://developer.apple.com/design/human-interface-guidelines/playing-haptics
- Android haptics design principles: https://developer.android.com/develop/ui/views/haptics/haptics-principles
- Okabe & Ito, Color Universal Design: https://jfly.uni-koeln.de/color/
- Okabe-Ito palette hex reference: https://conceptviz.app/blog/okabe-ito-palette-hex-codes-complete-reference
- Wong (2011), Points of view: Color blindness, Nature Methods 8:441, DOI 10.1038/nmeth.1618

Accessibility and ergonomics
- WCAG 2.2 Understanding 1.4.3 Contrast (Minimum): https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
- WCAG 2.2 Understanding 1.4.11 Non-text Contrast: https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html
- WCAG 2.2 Understanding 1.4.4 Resize Text: https://www.w3.org/WAI/WCAG22/Understanding/resize-text.html
- WCAG 2.2 Understanding 2.5.8 Target Size (Minimum): https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- WCAG target size discussion quoting Apple 44 pt / Material 48 dp: https://github.com/w3c/wcag/issues/1831
- Hoober (2013), How do users really hold mobile devices?: https://www.uxmatters.com/mt/archives/2013/02/how-do-users-really-hold-mobile-devices.php
- Thumb zone and left-handed mirroring (secondary): https://parachutedesign.ca/blog/thumb-zone-ux/
- Numeric keypad layouts (calculator vs telephone, Deininger 1960): https://en.wikipedia.org/wiki/Keypad
- A brief history of the numeric keypad: https://uxdesign.cc/a-brief-history-of-the-numeric-keypad-59112cbf4c49
- Handed keypad mirroring patent (targets mirrored, glyphs not): https://patents.justia.com/patent/10048860
