# This is a work in progress.

Planned steps:

1. Add a "real" README.md explaining how to build and operate this. Finish up the explanation of this decider's power relative to other "CTL" techniques, as started here: https://github.com/uncombedCoconut/bbchallenge/#principles
2. Simplify the integer width configuration in `core/limits.rs`. The unedited version imposes a depth (1-sided DFA size) limit of 12. Using u128 instead of u64 enables up to 25, at the cost of a 25% slowdown or so.
3. Implement the SAT-solver approach here (placeholder: `provers/mitm_dfa.rs` and the dependency on `cadical`).
4. Implement index-file I/O (placeholder: `io/index.rs`) and save to `output/`.
5. Implement a CLI so provers and depth min/max are settable, maybe also proof data export.
6. Implement parallel/distributed processing (placeholder: `node_crunch` dependency) with status display (placeholder: `indicatif` dependency).
7. After learning from that, clean up the awkward Prover trait.
