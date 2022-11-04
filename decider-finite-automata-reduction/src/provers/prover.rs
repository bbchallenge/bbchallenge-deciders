//! A common interface for the actual Proof-finding code.

use super::{DirectProver, MitMDFAProver};
use crate::core::{Machine, Proof, MAX_DFA};
use std::cmp::{max, min};
use std::ops::Range;

pub type ProverBox = Box<dyn Prover + Send>;

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

    /// Return a vector of Provers, for whichever of the given depths are valid.
    fn new_range(range: Range<usize>) -> Vec<ProverBox>
    where
        Self: Prover + Sized + Send + 'static,
    {
        let legal = Self::depths();
        ((max(range.start, legal.start))..min(range.end, legal.end))
            .map(|depth| Box::new(Self::new(depth)) as _)
            .collect()
    }
}

pub fn prover_names() -> impl Iterator<Item = String> {
    ["direct", "mitm_dfa"].into_iter().map(String::from)
}

/// Return a vector of Provers, for whichever of the given depths are valid.
pub fn prover_range_by_name<S: AsRef<str>>(name: S, range: Range<usize>) -> Vec<ProverBox> {
    match name.as_ref() {
        "direct" => DirectProver::new_range(range),
        "mitm_dfa" => MitMDFAProver::new_range(range),
        _ => vec![],
    }
}

pub fn prover_by_name<S: AsRef<str>>(name: S) -> Option<ProverBox> {
    let (class, depth_str) = name.as_ref().rsplit_once('-')?;
    let depth = depth_str.parse::<usize>().ok()?;
    prover_range_by_name(class, depth..depth + 1).pop()
}
