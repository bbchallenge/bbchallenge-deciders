# This is a work in progress.

Planned steps:

1. Add a "real" README.md explaining how to build and operate this. Finish up the explanation of this decider's power relative to other "CTL" techniques, as started here: https://github.com/uncombedCoconut/bbchallenge/#principles
2. Simplify the integer width configuration in `core/limits.rs`. The unedited version imposes a depth (1-sided DFA size) limit of 12. Using u128 instead of u64 enables up to 25, at the cost of a 25% slowdown or so.
3. Maybe extend CLI to support proof file (DVF) checking, and ad-hoc explanation of a seed-DB entry / DVF entry / TM defined by text.
