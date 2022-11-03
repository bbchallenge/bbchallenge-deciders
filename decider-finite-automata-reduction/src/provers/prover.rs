//! A common interface for the actual Proof-finding code.

use crate::core::{Machine, Proof, MAX_DFA};
use std::ops::Range;

/// At this level, `Prover`s are object-oriented. An instance provides:
pub trait Prover {
    /// An identifier for the proof strategy -- may be used in output file names or status displays.
    fn name(&self) -> String;

    /// Either return a `Proof` for `tm` -- should be valid, but caller must verify -- or give up.
    fn prove(&mut self, tm: &Machine) -> Option<Proof>;
}

pub trait ProverOptions {
    /// A new instance. Should be inexpensive, but first prove() call may be expensive.
    fn new(depth: usize) -> Self;

    /// The range of possible search-depth parameters.
    fn depths() -> Range<usize> {
        1..(MAX_DFA + 1)
    }
}
