# Finger rules

## The question

On paper it does not matter which hand writes a digit. On the soroban it
does: a fast operator makes every move with one of two fingers, in a fixed
order, so the hand never hesitates. Abax shows the finger with every move
and every move list follows the order below.

## Which finger

<svg class="abax-diagram" viewBox="0 0 394 202" role="img" aria-labelledby="t-fingers d-fingers" xmlns="http://www.w3.org/2000/svg"><title id="t-fingers">Which finger moves which bead</title><desc id="d-fingers">One rod. The index finger moves the heaven bead in both directions. The thumb pushes earth beads up to the beam. The index finger pushes earth beads down, away from the beam.</desc><rect class="sb-frame" x="10" y="10" width="70" height="182" rx="8"/><rect class="sb-well" x="18" y="18" width="54" height="166" rx="3"/><line class="sb-rod" x1="45" y1="18" x2="45" y2="184"/><rect class="sb-beam" x="18" y="56" width="54" height="14"/><path class="sb-bead" d="M26.0 31.0 Q45.0 9.0 64.0 31.0 Q45.0 53.0 26.0 31.0Z"/><path class="sb-bead" d="M26.0 99.0 Q45.0 77.0 64.0 99.0 Q45.0 121.0 26.0 99.0Z"/><path class="sb-bead" d="M26.0 123.0 Q45.0 101.0 64.0 123.0 Q45.0 145.0 26.0 123.0Z"/><path class="sb-bead" d="M26.0 147.0 Q45.0 125.0 64.0 147.0 Q45.0 169.0 26.0 147.0Z"/><path class="sb-bead" d="M26.0 171.0 Q45.0 149.0 64.0 171.0 Q45.0 193.0 26.0 171.0Z"/><line class="sb-arrow" x1="94" y1="25" x2="94" y2="50"/><path class="sb-arrowhead" d="M89 27 L94 19 L99 27Z"/><path class="sb-arrowhead" d="M89 48 L94 56 L99 48Z"/><text class="sb-label" x="110" y="41.5" text-anchor="start">heaven bead, both ways: index finger</text><line class="sb-arrow" x1="94" y1="171" x2="94" y2="88"/><path class="sb-arrowhead" d="M89 90 L94 82 L99 90Z"/><line class="sb-arrow" x1="118" y1="88" x2="118" y2="165"/><path class="sb-arrowhead" d="M113 163 L118 171 L123 163Z"/><text class="sb-label" x="134" y="116.5" text-anchor="start">earth beads up: thumb</text><text class="sb-label" x="134" y="142.5" text-anchor="start">earth beads down: index finger</text></svg>

The standard Japanese method uses two fingers of one hand:

| Move | Finger |
|------|--------|
| Earth beads up, towards the beam (adds) | thumb |
| Earth beads down, away from the beam (subtracts) | index |
| Heaven bead down, to the beam (adds 5) | index |
| Heaven bead up, away from the beam (subtracts 5) | index |

The thumb only ever pushes up. Everything else is the index finger. That is
all of it; the rest is about order.

## Kojima's six ordering rules

When one digit needs both a heaven-bead move and an earth-bead move, or
needs a second rod, the order is fixed. Kojima (1954, chapter III) gives six
rules; Japanese school materials teach the same.

1. **Adding 6 to 9 directly** (for example 1 + 6): bring the heaven bead down
   and push the earth beads up *at the same time*, as one pinch.
2. **Subtracting 6 to 9 directly** (for example 9 − 7): earth beads down
   *first*, then the heaven bead up.
3. **Five-complement addition** (for example 4 + 3, done as +5 −2): heaven
   bead down *first*, then the earth beads down, in quick succession.
4. **Five-complement subtraction** (for example 7 − 3, done as +2 −5): earth
   beads up *first*, then the heaven bead up.
5. **Ten-complement addition** (for example 4 + 8, done as −2 then +1 on the
   left): finish the unit rod *first*, then carry 1 onto the tens rod.
6. **Ten-complement subtraction** (for example 11 − 7, done as −1 on the
   left then +3): borrow 1 from the tens rod *first*, then work the unit
   rod.

Across rods the rule is simpler: numbers are entered from the highest place
to the lowest, left to right, one rod at a time, carrying and borrowing as
you go. A carry always lands on a rod you have already visited, never on one
still to come.

Some English tutorials narrate the carry first ("add 10, take away 3"). The
hand order taught in Japan, and used by Abax, is the one above: complement
first and carry second when adding, borrow first and complement second when
subtracting.

## The move notation

The manual writes moves on one rod like this:

| Notation | Meaning | Finger |
|----------|---------|--------|
| `+1` … `+4` | push that many earth beads up | thumb |
| `−1` … `−4` | push that many earth beads down | index |
| `+5` | heaven bead down | index |
| `−5` | heaven bead up | index |
| `+1 left`, `−1 left` | carry to, or borrow from, the next rod to the left | thumb, index |

So "4 + 3" on a rod showing 4 is `+5, −2`; "4 + 8" is `−2, +1 left`.
