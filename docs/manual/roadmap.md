# Roadmap

These are intentions, not promises, roughly in order.

- **Sound on Android.** The desktop click uses cpal, which needs the C++
  library oboe on Android. The plan is the platform's AAudio through the
  `ndk` crate, without C++.
- **Signed desktop builds.** Code-signing and notarizing the macOS app and
  signing the Windows binary, so the first launch needs no extra
  confirmation.
- **Deeper multiplication and division.** More lessons and steps: decimals
  and Kojima's rules for placing the point, longer multipliers and divisors,
  and the head-first methods many Japanese schools use today.
- **More languages.** The interface and lessons are English only. The text
  lives in a few places (`curriculum.rs` and the screens), which makes
  translation possible; right-to-left scripts need layout work.
- **Dictated numbers.** Reading numbers aloud (*yomiage-zan*) is a standard
  soroban exercise alongside flash anzan.
- **A placement probe.** The scheduler already supports placing an
  experienced learner from a short mixed test; the screen for it does not
  exist yet.
- **Faster flash steps.** The flash ladders stop at 0.4 s per number for
  one digit and 0.5 s for three; the exam ladders go much further.

Suggestions are welcome as GitHub issues.
