//! Error/Result definitions for the outcomes of checking a proof.
use super::{DFAState, Rule};
use custom_error::custom_error;

custom_error! {
    #[derive(Eq, PartialEq)]
    pub BadProof
    /// A reason for rejecting a proof.

    BadDimensions = "array lengths did not match",
    BadVector = "indices on a vector (represented as a binary number) went out of bounds",
    BadDFASize = "DFA too small to address the initial state",
    BadNFASize = "NFA too small to address the states nfa_start(q, f)",
    BadDFATransition = "DFA transitions went out of bounds",
    LeadingZeroSensitivity = "DFA failed to ignore leading zeros",
    TrailingZeroSensitivity = "NFA failed to ignore trailing zeros",
    BadStart = "tape automaton accepted the start configuration",
    NotClosed {q: DFAState, rule: Rule} = "closure under {rule} unmet at q={q} (DFA)",
    BadSteadyState = "NFA transitions didn't preserve 'steady_state'",
    RejectedSteadyState = "NFA didn't accept 'steady_state",
}

pub type ProofResult<T> = Result<T, BadProof>;
