use super::{BadProof, DFAState, ProofResult};
use crate::core::NFAState;
use serde::{Deserialize, Serialize};

/// A Deterministic Finite Automaton, with states indexed by ``DFAState`s and initial state 0.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct DFA {
    pub t: Vec<[DFAState; 2]>,
}

impl DFA {
    /// A DFA with n states (initialized with all transitions leading to the initial state).
    pub fn new(n: usize) -> DFA {
        DFA { t: vec![[0, 0]; n] }
    }

    /// The number of states.
    pub fn len(&self) -> usize {
        self.t.len()
    }

    /// The outcome of a single step.
    pub fn step(&self, q: DFAState, b: u8) -> NFAState {
        self.t[q as usize][b as usize]
    }

    /// Ensure the data define a valid DFA.
    pub fn validate(&self) -> ProofResult<()> {
        if self.t.len() == 0 {
            Err(BadProof::BadDFASize)
        } else if self.t.iter().flatten().any(|&v| (v as usize) >= self.len()) {
            Err(BadProof::BadDFATransition)
        } else {
            Ok(())
        }
    }

    /// Ensure the DFA furthermore reaches the same state regardless of any leading zeros.
    pub fn check_leading_zeros(&self) -> ProofResult<()> {
        if self.t[0][0] == 0 {
            Ok(())
        } else {
            Err(BadProof::LeadingZeroSensitivity)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let dfa = DFA::new(1);
        assert_eq!(dfa.validate(), Ok(()));
        let dfa: DFA = serde_json::from_str("[[0, 1], [0, 2], [0, 0]]").unwrap();
        assert_eq!(dfa.validate(), Ok(()));
        let dfa: DFA = serde_json::from_str("[]").unwrap();
        assert_eq!(dfa.validate(), Err(BadProof::BadDFASize));
        let dfa: DFA = serde_json::from_str("[[0, 42]]").unwrap();
        assert_eq!(dfa.validate(), Err(BadProof::BadDFATransition));
    }

    #[test]
    fn test_check_leading_zeros() {
        // all looping transitions: yes.
        let dfa: DFA = serde_json::from_str("[[0, 0], [1, 1]]").unwrap();
        assert!(dfa.check_leading_zeros().is_ok());
        // all looping transitions except 0->1 when reading '1': yes.
        let dfa: DFA = serde_json::from_str("[[0, 1], [1, 1]]").unwrap();
        assert!(dfa.check_leading_zeros().is_ok());
        // all looping transitions except 0->1 when reading '0': no!
        let dfa: DFA = serde_json::from_str("[[1, 0], [1, 1]]").unwrap();
        assert!(dfa.check_leading_zeros().is_err());
    }

    #[test]
    fn test_step() {
        // Again, all looping transitions except 0->1 when reading '0':
        let dfa: DFA = serde_json::from_str("[[1, 0], [1, 1]]").unwrap();
        assert_eq!(dfa.step(0, 0), 1);
        assert_eq!(dfa.step(0, 1), 0);
        assert_eq!(dfa.step(1, 0), 1);
        assert_eq!(dfa.step(1, 0), 1);
    }
}
