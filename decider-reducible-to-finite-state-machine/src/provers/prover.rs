//! A common interface for the actual Proof-finding code.

use crate::core::{Machine, RowVector, TapeAutomaton, MAX_DFA};
use std::ops::Range;

/// The interface we expect for `Proof` search code.  
pub trait Prover {
    /// An identifier for the proof strategy -- may be used in output file names or status displays.
    fn name() -> &'static str;

    /// Provers are expected to take a search parameter to keep their runtimes under control.
    /// Override this is the allowable ranges isn't 1 to `MAX_DFA` inclusive.
    fn depths() -> Range<usize> {
        1..(MAX_DFA + 1)
    }

    /// Return an "accepted steady state" as in prover.rs.
    fn steady_state(depth: usize) -> RowVector;

    /// A new instance. May be expensive!
    fn new(depth: usize) -> Self;

    /// Either return a `TapeAutomaton` which (with the `steady_state()` value) proves `tm`
    /// infinite, or give up. The main program will check the proof before updating the index
    /// of decided machines.
    fn prove(&mut self, tm: &Machine) -> Option<TapeAutomaton>;
}
