//! Error/Result definitions for the outcomes of checking a proof.

use super::{DFAState, Rule};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A reason for rejecting a proof.
#[derive(Error, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BadProof {
    #[error("array lengths did not match")]
    BadDimensions,
    #[error("indices on a vector (reresented as a binary number) went out of bounds")]
    BadVector,
    #[error("DFA too small to address the initial state")]
    BadDFASize,
    #[error("NFA too small to address the states nfa_start(q, f)")]
    BadNFASize,
    #[error("DFA transitions went out of bounds")]
    BadDFATransition,
    #[error("DFA failed to ignore leading zeros")]
    LeadingZeroSensitivity,
    #[error("NFA failed to ignore trailing zeros")]
    TrailingZeroSensitivity,
    #[error("tape automaton accepted the start configuration")]
    BadStart,
    #[error("closure under {rule} unmet at q={q} (DFA)")]
    NotClosed { q: DFAState, rule: Rule },
    #[error("NFA transitions didn't preserve 'steady_state'")]
    BadSteadyState,
    #[error("NFA didn't accept 'steady_state")]
    RejectedSteadyState,
}

pub type ProofResult<T> = Result<T, BadProof>;
