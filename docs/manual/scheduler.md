# The adaptive scheduler

The scheduler decides what to practise next from your answers and their
times. The numbers below are those in `crates/abax-core/src/scheduler.rs`.

| Piece | Question |
|-------|----------|
| Bayesian Knowledge Tracing | Do you know this rule? |
| Accuracy average | How are you doing lately? |
| Time average | How fast are you when you are right? |
| Elo ability | Is this particular step too hard for you? |
| Ladder rules | Should the next items be harder or easier? |
| Mastery gate | Are you done with this skill? |
| Forgetting and review | Have you started to lose it? |
| Session planner | Which drills, in which order, today? |

## Do you know the rule? Bayesian Knowledge Tracing

Bayesian Knowledge Tracing (Corbett and Anderson, 1995) keeps one number
per skill, *L*, the probability that you know it, allowing for guesses and
slips:

| Parameter | Value | Meaning |
|-----------|-------|---------|
| P(L₀) | 0.10 | before any answer, a 10 % chance you already know it |
| P(G) | 0.20 | chance of a right answer without knowing the rule (guess) |
| P(S) | 0.10 | chance of a wrong answer although you know it (slip) |
| P(T) | 0.10 | chance of learning it from one more item |

After a **right** answer, Bayes' rule gives

*post* = *L* (1 − P(S)) / ( *L* (1 − P(S)) + (1 − *L*) P(G) )

after a **wrong** answer

*post* = *L* P(S) / ( *L* P(S) + (1 − *L*) (1 − P(G)) )

and then the chance that this item taught you the rule is added:

*L*′ = *post* + (1 − *post*) P(T).

### By hand

Start at *L* = 0.10 and answer correctly.

- Numerator: 0.10 × 0.90 = 0.09. Denominator: 0.09 + 0.90 × 0.20 = 0.27.
  *post* = 0.09 / 0.27 = 0.333.
- Learning step: 0.333 + 0.667 × 0.10 = **0.400**.

Correct again from 0.400: 0.36 / (0.36 + 0.60 × 0.20) = 0.75, then
0.75 + 0.25 × 0.10 = **0.775**. Two more right answers give **0.9455** and
**0.9886**.

Now a wrong answer from 0.9886: 0.09886 / (0.09886 + 0.0114 × 0.80) =
0.9156, plus the learning step: **0.924**. The same slip after three right
answers, from 0.9455, gives 0.09455 / (0.09455 + 0.0545 × 0.80) = 0.684,
then **0.716**.

The [curriculum graph](curriculum.md#the-graph) uses *L* directly: a
skill's prerequisites count as met when every one has *L* ≥ 0.80. Three
right answers in a row on S3 (0.9455) are already enough to make S5 the
suggested next step.

**I know this already** sets *L* to exactly 0.80 (and θ to the matching
ln(0.8 / 0.2) = 1.386) without adding any answers. One right answer from
there: 0.72 / (0.72 + 0.20 × 0.20) = 0.947, then **0.953**. One wrong
answer: 0.08 / (0.08 + 0.20 × 0.80) = 0.333, then **0.400**, back below the
threshold.

## How are you doing lately? The accuracy average

An exponentially weighted moving average (EWMA) of accuracy tracks recent
performance:

acc′ = acc + 0.2 × (correct − acc), with correct = 1 or 0, starting at 0.5.

Each answer moves the average a fifth of the way towards itself, so it
mostly reflects the last ten or so items. From 0.5, right answers give 0.6,
0.68, 0.744; a wrong one then drops it to 0.744 − 0.2 × 0.744 = 0.595. Ten
right answers in a row from 0.5 reach 0.946.

## How fast? The time average

An EWMA of the logarithm of **seconds per operand**, on right answers
only:

log_t′ = log_t + 0.3 × (ln(seconds per operand) − log_t).

Your "typical time" is exp(log_t). It starts at three times the skill's
benchmark. Benchmarks, in seconds per operand: S0 3.0; S1 and S2
2.5; S3 and S4 3.0; S5, S6 and S7 3.5; S8 and S9 5.0; S10 12; S11 15; A1 2.0;
A2 and A3 1.0.

## Is this step too hard? Elo ability

You and the item are rated on one scale, as in chess. Each item has a
difficulty *d* computed from its step:

*d* = base(skill) + 0.40 (digits − 1) + 0.25 (operands − 2) + 0.50 [timed] + 0.30 [flash]

with base −1.5 for S0; −1.0 for S1 and S2; 0.0 for S3 and S4; 0.5 for S5
and S6; 1.0 for S7; 1.5 for S8 and A1; 2.0 for S9 and S10; 2.5 for S11 and
A2; 3.0 for A3. Your ability θ on the skill starts at 0. The predicted
chance of a right answer is the logistic curve

P = 1 / (1 + e^−(θ − *d*)),

and after each answer

θ′ = θ + U(n) × (correct − P), with U(n) = 1 / (1 + 0.06 n),

where *n* is how many items of this skill you have answered. A surprise
moves θ a lot, an expected result very little, and U shrinks as evidence
accumulates: U(0) = 1, U(10) = 0.625, U(100) = 0.14.

## Harder or easier? The ladder rules

Every skill has a [ladder of steps](curriculum.md#ladders). After each
answer:

**Step down** (if not already on the first step) when you have made two
errors in a row on this step, or when accuracy has fallen below 0.60 after
at least 6 items on the step.

**Step up** when all of these hold:

- accuracy ≥ 0.85;
- at least 8 items on this step;
- the Elo prediction for this step, P, is at least 0.85;
- on a timed step: typical time ≤ 0.8 × the time budget that was fixed when
  you entered the step;
- when the next step is the first timed one: accuracy ≥ 0.90.

## A worked session

A learner starts S3 (adding with the five-complement) at step 1: two
one-digit operands, untimed, so *d* = 0. Each item takes 4.0 s, which is
2.0 s per operand. They answer right, right, right, wrong, then right eight
times. These are the scheduler's actual numbers:

| Item | Answer | *L* | acc | typical s/operand | θ | Step |
|------|--------|-----|-----|-------------------|---|------|
| 1 | right | 0.400 | 0.600 | 5.73 | 0.500 | 1 |
| 2 | right | 0.775 | 0.680 | 4.18 | 0.856 | 1 |
| 3 | right | 0.946 | 0.744 | 3.35 | 1.122 | 1 |
| 4 | wrong | 0.716 | 0.595 | 3.35 | 0.483 | 1 |
| 5 | right | 0.927 | 0.676 | 2.87 | 0.791 | 1 |
| 6 | right | 0.985 | 0.741 | 2.58 | 1.031 | 1 |
| 7 | right | 0.997 | 0.793 | 2.39 | 1.224 | 1 |
| 8 | right | 0.999 | 0.834 | 2.26 | 1.384 | 1 |
| 9 | right | 1.000 | 0.867 | 2.18 | 1.520 | 1 |
| 10 | right | 1.000 | 0.894 | 2.13 | 1.636 | 1 |
| 11 | right | 1.000 | 0.915 | 2.09 | 1.738 | **2** |
| 12 | right | 1.000 | 0.932 | 1.82 | 1.849 | 2 |

Follow item 1 through every piece:

- *L*: 0.10 → 0.400, as computed above.
- acc: 0.5 + 0.2 × (1 − 0.5) = 0.6.
- time: the average starts at ln(3 × 3.0) = ln 9 = 2.197. The sample is
  ln 2.0 = 0.693. 2.197 + 0.3 × (0.693 − 2.197) = 1.746, and e^1.746 = 5.73 s.
- Elo: P = 1 / (1 + e^0) = 0.5 and U(0) = 1, so θ = 0 + 1 × (1 − 0.5) = 0.5.

Item 2's Elo update: P = 1 / (1 + e^−0.5) = 0.622, U(1) = 1 / 1.06 = 0.943,
θ = 0.5 + 0.943 × 0.378 = 0.856.

Why no step up before item 11? At item 9 the learner already had 9 items on
the step, *L* near 1 and accuracy 0.867, but the Elo prediction was
1 / (1 + e^−1.520) = 0.82, under 0.85. At item 10 it was 0.837. At item 11,
θ = 1.738 gives 0.850, accuracy is 0.915, and the next step is still
untimed: up to three operands. The wrong answer at item 4 cost about three
items.

Readiness follows the live estimate. S3's *L* passed 0.80 at item 3, which
made S5 a suggested next step (and S4, once S2 is there too); the error at
item 4 took that back until item 5.

## Are you done? The mastery gate

A skill is marked **mastered** when all five hold:

- *L* ≥ 0.95;
- at least 10 answers on the skill;
- accuracy ≥ 0.90;
- you have reached at least the second-to-last step of its ladder, which for
  most skills is the first timed one;
- typical time ≤ 1.5 × the skill's benchmark.

Four lucky answers reach *L* = 0.989 but fail the count. The learner
above, after twelve items, is on step 2 of 6: S3 counts as met, but is not
mastered.

## Have you started to lose it? Forgetting and review

*L* decays with the days since the skill was last practised:

*L*_eff = *L* × e^(−days / τ), with τ = 14 × review interval (in days).

A mastered skill starts with a review interval of 1 day. It is **due for
review** when *L*_eff falls below 0.80 or when the interval has passed.
Example: *L* = 0.96, 14 days without practice, interval 1 day: τ = 14 and
*L*_eff = 0.96 × e^−1 = 0.353, clearly due. Had the interval grown to 10
days, τ = 140 and *L*_eff = 0.869: not due by decay, but due because 14 days
exceeds 10.

When you answer a due review with accuracy ≥ 0.90, the interval grows by a
factor of 1.6, up to 60 days. If accuracy on a mastered skill drops below 0.70, the mastered mark
is removed and the interval resets to 1 day.

## What today? The session planner

A planned session is three short drills. They pick from the skills on the
suggested path (prerequisites met) plus every skill you have already met,
practised or marked as known. A gap in the prerequisites never stops it.

1. **Warm-up, 7 items.** The due review that has waited longest, at its
   current step. If nothing is due and the focus skill below has a
   prerequisite under 0.80 (you went ahead of the suggested path), a
   **light review** of that prerequisite instead: 5 untimed items at its
   current step, the weakest prerequisite first. Otherwise the most recently
   mastered skill at its top step. Target success about 90 % (85 % for the
   light review).
2. **Focus, 12 items.** Among those skills that are not yet mastered, sort
   by *L*_eff and take the weakest, with probability 0.7, or the second
   weakest, with probability 0.3. About 30 % of its items come from other skills you have
   already met, each at its own step. Target success about 80 %. The focus
   runs whatever the state of its prerequisites.
3. **Cool-down, 6 items.** Untimed items from the skills you know well
   (*L*_eff ≥ 0.80), strongest first, mixed together. A skill marked as
   known starts here at exactly 0.80.

An example with a gap. A new learner opens S10 straight away and answers
six multiplication items, half of them right, and has also done some S0.
S10 is now a skill they have met, so the planner can choose it; with the
lowest *L* it is the usual focus. Its prerequisite S8 has never been
touched (*L* = 0.10), nothing is due, so the warm-up becomes five untimed
S8 items at step 1, and then the twelve S10 items run as normal.

On a brand-new profile the only skill on the path is S0, so the plan is a
single 12-item S0 drill. And before any of this, the home screen's
**Continue** sends you to the suggested next lesson, the first skill on the
path you have neither tried nor marked as known.

## Momentum

The stats screen shows the last 14 days: items, minutes, days active, and a
"momentum" figure that halves every three days without practice.

## Sources

- Corbett, A. T. and Anderson, J. R. (1995), "Knowledge tracing: Modeling
  the acquisition of procedural knowledge", *User Modeling and User-Adapted
  Interaction* 4: 253–278.
- Pelánek, R. et al., "Elo-based learner modeling for the adaptive practice
  of facts", *User Modeling and User-Adapted Interaction* (the uncertainty
  function U(n) = α / (1 + βn), with α = 1 and β = 0.06).
- Klinkenberg, Straatemeier and van der Maas (2011), Math Garden: item
  difficulty and ability on one logistic scale.
- Rohrer and Taylor (2007), on interleaved practice.
