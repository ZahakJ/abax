# The soroban

## The question it answers

How do you write a number with beads so that a hand can add to it without
thinking about how it is written? The soroban's answer is to give every
decimal place its own rod, and to split each digit into "how many fives"
and "how many ones".

## Parts and values

<svg class="abax-diagram" viewBox="0 0 480 222" role="img" aria-labelledby="t-parts d-parts" xmlns="http://www.w3.org/2000/svg"><title id="t-parts">The parts of a soroban</title><desc id="d-parts">Five rods of a soroban showing 20579. The heaven bead above the beam counts 5, each earth bead below counts 1, and a bead counts only when it touches the beam.</desc><rect class="sb-frame" x="10" y="10" width="254" height="182" rx="8"/><rect class="sb-well" x="18" y="18" width="238" height="166" rx="3"/><line class="sb-rod" x1="45" y1="18" x2="45" y2="184"/><line class="sb-rod" x1="91" y1="18" x2="91" y2="184"/><line class="sb-rod" x1="137" y1="18" x2="137" y2="184"/><line class="sb-rod" x1="183" y1="18" x2="183" y2="184"/><line class="sb-rod" x1="229" y1="18" x2="229" y2="184"/><rect class="sb-beam" x="18" y="56" width="238" height="14"/><circle class="sb-dot" cx="91" cy="63" r="2.6"/><circle class="sb-dot" cx="229" cy="63" r="2.6"/><path class="sb-bead" d="M26.0 31.0 Q45.0 9.0 64.0 31.0 Q45.0 53.0 26.0 31.0Z"/><path class="sb-bead" d="M26.0 82.0 Q45.0 60.0 64.0 82.0 Q45.0 104.0 26.0 82.0Z"/><path class="sb-bead" d="M26.0 106.0 Q45.0 84.0 64.0 106.0 Q45.0 128.0 26.0 106.0Z"/><path class="sb-bead" d="M26.0 147.0 Q45.0 125.0 64.0 147.0 Q45.0 169.0 26.0 147.0Z"/><path class="sb-bead" d="M26.0 171.0 Q45.0 149.0 64.0 171.0 Q45.0 193.0 26.0 171.0Z"/><path class="sb-bead" d="M72.0 31.0 Q91.0 9.0 110.0 31.0 Q91.0 53.0 72.0 31.0Z"/><path class="sb-bead" d="M72.0 99.0 Q91.0 77.0 110.0 99.0 Q91.0 121.0 72.0 99.0Z"/><path class="sb-bead" d="M72.0 123.0 Q91.0 101.0 110.0 123.0 Q91.0 145.0 72.0 123.0Z"/><path class="sb-bead" d="M72.0 147.0 Q91.0 125.0 110.0 147.0 Q91.0 169.0 72.0 147.0Z"/><path class="sb-bead" d="M72.0 171.0 Q91.0 149.0 110.0 171.0 Q91.0 193.0 72.0 171.0Z"/><path class="sb-bead" d="M118.0 44.0 Q137.0 22.0 156.0 44.0 Q137.0 66.0 118.0 44.0Z"/><path class="sb-bead" d="M118.0 99.0 Q137.0 77.0 156.0 99.0 Q137.0 121.0 118.0 99.0Z"/><path class="sb-bead" d="M118.0 123.0 Q137.0 101.0 156.0 123.0 Q137.0 145.0 118.0 123.0Z"/><path class="sb-bead" d="M118.0 147.0 Q137.0 125.0 156.0 147.0 Q137.0 169.0 118.0 147.0Z"/><path class="sb-bead" d="M118.0 171.0 Q137.0 149.0 156.0 171.0 Q137.0 193.0 118.0 171.0Z"/><path class="sb-bead" d="M164.0 44.0 Q183.0 22.0 202.0 44.0 Q183.0 66.0 164.0 44.0Z"/><path class="sb-bead" d="M164.0 82.0 Q183.0 60.0 202.0 82.0 Q183.0 104.0 164.0 82.0Z"/><path class="sb-bead" d="M164.0 106.0 Q183.0 84.0 202.0 106.0 Q183.0 128.0 164.0 106.0Z"/><path class="sb-bead" d="M164.0 147.0 Q183.0 125.0 202.0 147.0 Q183.0 169.0 164.0 147.0Z"/><path class="sb-bead" d="M164.0 171.0 Q183.0 149.0 202.0 171.0 Q183.0 193.0 164.0 171.0Z"/><path class="sb-bead" d="M210.0 44.0 Q229.0 22.0 248.0 44.0 Q229.0 66.0 210.0 44.0Z"/><path class="sb-bead" d="M210.0 82.0 Q229.0 60.0 248.0 82.0 Q229.0 104.0 210.0 82.0Z"/><path class="sb-bead" d="M210.0 106.0 Q229.0 84.0 248.0 106.0 Q229.0 128.0 210.0 106.0Z"/><path class="sb-bead" d="M210.0 130.0 Q229.0 108.0 248.0 130.0 Q229.0 152.0 210.0 130.0Z"/><path class="sb-bead" d="M210.0 154.0 Q229.0 132.0 248.0 154.0 Q229.0 176.0 210.0 154.0Z"/><text class="sb-num" x="45" y="210" text-anchor="middle">2</text><text class="sb-num" x="91" y="210" text-anchor="middle">0</text><text class="sb-num" x="137" y="210" text-anchor="middle">5</text><text class="sb-num" x="183" y="210" text-anchor="middle">7</text><text class="sb-num" x="229" y="210" text-anchor="middle">9</text><line class="sb-leader" x1="252" y1="44" x2="284" y2="44"/><text class="sb-label" x="290" y="48" text-anchor="start">heaven bead: 5</text><line class="sb-leader" x1="252" y1="63" x2="284" y2="63"/><text class="sb-label" x="290" y="67" text-anchor="start">beam (reckoning bar)</text><line class="sb-leader" x1="252" y1="82" x2="284" y2="82"/><text class="sb-label" x="290" y="86" text-anchor="start">earth beads: 1 each</text><line class="sb-leader" x1="229" y1="66" x2="229" y2="63"/><line class="sb-leader" x1="260" y1="192" x2="284" y2="192"/><text class="sb-label" x="290" y="196" text-anchor="start">frame</text><line class="sb-leader" x1="91" y1="66" x2="91" y2="63"/><text class="sb-label sb-soft" x="290" y="127" text-anchor="start">unit dots: every third rod</text><line class="sb-leader" x1="232" y1="65" x2="284" y2="123"/></svg>

- **Rods.** Each rod is one decimal place. The rightmost rod in use is the
  units, the next one to the left the tens, and so on, exactly as you write
  numbers.
- **Beam.** The horizontal bar (the reckoning bar) divides every rod into
  two parts.
- **Heaven bead.** The single bead above the beam is worth 5.
- **Earth beads.** The four beads below the beam are worth 1 each.
- **Frame.** Holds it all together; it plays no part in the arithmetic.

A bead counts only when it touches the beam. So each rod shows one digit:

| Digit | Heaven bead | Earth beads at the beam |
|-------|-------------|-------------------------|
| 0 | up (away) | none |
| 1 to 4 | up (away) | 1 to 4 |
| 5 | down (touching) | none |
| 6 to 9 | down (touching) | 1 to 4 |

Why only four earth beads and one heaven bead? Because 4 ones and 1 five are
exactly enough for 0 to 9, and nothing more. Older soroban had 5 earth beads
and 2 heaven beads; the 1:4 form became the Japanese standard in the first
half of the twentieth century. With no spare beads, every digit has exactly
one picture, which is what makes reading fast and, later, picturing the
soroban in your head possible.

## Unit dots

The beam carries a small dot on every third rod. You choose one dotted rod
as the units rod; the dots to its left then fall on the thousands, the
millions, and so on, the way commas group digits on paper. In the diagram
above the rightmost rod is the units rod and the dot on the second rod marks
the thousands, so the soroban shows 20579.

Abax counts rods from the right: the rightmost rod is always the units rod
and carries a dot, as does every third rod to its left.

## Reading a number

Read left to right, one rod at a time, the way you read written digits. For
each rod: is the heaven bead down (5) or not (0); how many earth beads touch
the beam (0 to 4); add the two. Leading rods with nothing at the beam are
leading zeros.

## Setting a number

Start from a clear soroban: every heaven bead up, every earth bead down.
Then set the digits from left to right, with the finger the
[finger rules](fingers.md) give:

- Set 3: the thumb pushes three earth beads up.
- Set 7: the index finger brings the heaven bead down and the thumb pushes
  two earth beads up, in one pinch.
- Set 406: four earth beads up on the hundreds rod, nothing on the tens,
  heaven bead down and one earth bead up on the units.

These are the first demonstrations of lesson S0 in Abax (3, 7, 50, 406).

## Clearing

On a physical soroban you clear by tilting it towards you so every bead
drops away from the beam, laying it flat and running the index finger along
the top of the beam to push the heaven beads up. In Abax the **Clear**
button (or Escape) does the same with a short sweep from the right.

## How many rods

Abax offers 9, 13, 17, 21, 23 and 27 rods. A real soroban always has an
odd number of rods; 13 is the usual beginner size and 23 the usual school
and exam size. Drills size the soroban to the problem on phones, so the
beads stay large enough to tap.
