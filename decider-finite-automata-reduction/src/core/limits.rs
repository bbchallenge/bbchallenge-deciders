//! Define up the hard, compile-time limits on our `BB(_)` problem and search space.

/// The exact number of states we expect from Turing Machines.
pub const TM_STATES: usize = 5;
/// The maximum number of states in a Proof's DFA.
pub const MAX_DFA: usize = if cfg!(feature = "u128") { 25 } else { 12 };
/// The maximum number of states in a Proof's NFA.
pub const MAX_NFA: usize = if cfg!(feature = "u128") { 128 } else { 64 };

/// A number indexing a TM state.
pub type TMState = u8;
/// A number indexing a DFA state.
pub type DFAState = u8;
/// A number indexing an NFA state.
pub type NFAState = u8;
/// A number indexing a set of NFA states: state i is included if bit `1<<i` is set.
#[cfg(feature = "u128")]
pub type NFAStateMask = u128;
/// A number indexing a set of NFA states: state i is included if bit `1<<i` is set.
#[cfg(not(feature = "u128"))]
pub type NFAStateMask = u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency() {
        assert!(TMState::MAX as usize + 1 >= TM_STATES);
        assert!(DFAState::MAX as usize + 1 >= MAX_DFA);
        assert!(NFAState::MAX as usize + 1 >= MAX_NFA);
        assert!(NFAStateMask::BITS as usize >= MAX_NFA);
        assert!(MAX_DFA * TM_STATES <= MAX_NFA, "nfa_start values won't fit");
    }
}
