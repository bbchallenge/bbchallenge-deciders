//! Definitions of the ingredients and validation of a proof that a given Turing Machine can't halt.
//! The Reproducibility and Verifiability Statement `[1]` requires a particular rigor here.
//! The core code strives to be simple and clearly correct (aside background terms and knowledge).
//! Then it's documented and unit tested anyway.
//! This eases the burden on advanced provers and SAT solvers: they can't have mistakes that matter.
//! `[1]` https://bbchallenge.org/method#reproducibility-and-verifiability-statement

mod algebra;
mod dfa;
mod error;
mod limits;
mod machine;
mod nfa;
mod proof;

pub use algebra::{col, row, ColVector, Matrix, RowVector};
pub use dfa::DFA;
pub use error::{BadProof, ProofResult};
pub use limits::{DFAState, NFAState, NFAStateMask, TMState, MAX_DFA, MAX_NFA, TM_STATES};
pub use machine::{Machine, Rule, Side};
pub use nfa::NFA;
pub use proof::{nfa_start, Proof, TapeAutomaton};
