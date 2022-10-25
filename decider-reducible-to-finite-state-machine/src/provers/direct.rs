//! This prover searches directly for a successful `TapeAutomaton`.
//! The depth parameter controls the DFA size.
//! As `../README.md` explains, we know an NFA only needs `(depth*TM_STATES+1)` states:
//! `nfa_start(q, f)` as defined in proof.rs, plus a special HALT state.
//! HALT is automatically an "accepted steady state", suggesting the following `Proof` search:
//! pick a direction, then pick a DFA, then construct the minimal NFA (in terms of added transitions
//! and accepted states) satisfying the closure conditions in proof.rs.
//! The search has succeeded iff  the NFA rejects `nfa_start(0, 0)`.
//! This is a powerful algorithm already, but now let's look closer:
//! When we "pick a DFA", we're building the transition table incrementally.
//! If we only know it through a fixed `(q, b)`, we can still compute the minimal NFA satisfying
//! the closure criteria we know about. This might already accept `nfa_start(0, 0)`, in which case
//! we needn't bother to complete the DFA; otherwise, we've at least made progress building the NFA.

use super::{DFAPrefixIterator, Prover};
use crate::core::{
    col, nfa_start, row, Machine, NFAState, RowVector, Rule, Side, TapeAutomaton, DFA, NFA,
    TM_STATES,
};

/// A prover which attempts a direct search for a `TapeAutomaton` meeting the proof criteria.
/// If one exists with at most `depth` DFA states, the prover will find it.
pub struct DirectProver {
    /// The DFA size to use.
    depth: usize,
}

impl Prover for DirectProver {
    fn name() -> &'static str {
        "direct"
    }

    fn steady_state(depth: usize) -> RowVector {
        row((TM_STATES * depth) as NFAState)
    }

    fn new(depth: usize) -> Self {
        DirectProver { depth }
    }

    fn prove(&mut self, tm: &Machine) -> Option<TapeAutomaton> {
        self.prove_side(tm, Side::R)
            .or_else(|| self.prove_side(tm, Side::L))
    }
}

impl DirectProver {
    fn nfa_halt(&self) -> NFAState {
        (TM_STATES * self.depth) as NFAState
    }

    /// Try to return a TapeAutomaton proving `tm` infinite, given the choice of scan direction.
    fn prove_side(&mut self, tm: &Machine, direction: Side) -> Option<TapeAutomaton> {
        let mut dfas = DFAPrefixIterator::new(self.depth);
        let mut nfas = vec![NFA::new(self.depth * TM_STATES + 1); 2 * self.depth];
        let halt = self.nfa_halt();
        loop {
            let (q_new, b_new) = dfas.next()?;
            let ply = (2 * q_new + b_new) as usize;
            if ply == 0 {
                Self::init(&dfas.dfa, &mut nfas[0], tm, halt);
            } else {
                nfas[ply] = nfas[ply - 1].clone();
            }
            Self::saturate(&dfas.dfa, &mut nfas[ply], tm, direction, q_new, b_new);
            if row(nfa_start(0, 0)) * nfas[ply].accepted {
                dfas.skip_current_subtree();
                continue;
            }
            if (q_new as usize, b_new) == (self.depth - 1, 1) {
                return Some(TapeAutomaton::new(direction, dfas.dfa, nfas[ply].clone()));
            }
        }
    }

    /// Initialize the NFA from the halt rules, which are independent of our DFA choices.
    fn init(dfa: &DFA, nfa: &mut NFA, tm: &Machine, halt: NFAState) {
        nfa.accepted = col(halt);
        for b in 0..2 {
            nfa.t[b][halt] = row(halt);
        }
        tm.rules().for_each(|rule| match rule {
            Rule::Halt { f, r } => {
                for q in 0..dfa.len() {
                    nfa.t[r as usize][nfa_start(q as NFAState, f)] |= row(halt);
                }
            }
            _ => {}
        })
    }

    /// Update `nfa` with all transitions and acceptances required by the closure conditions,
    /// given that `dfa` is known up to the `(q_new, b_new)` transition.
    /// The closure conditions for Move rules in the direction opposite our scan direction
    /// depend on the allowed NFA transitions, so this process repeats until there's nothing new.
    fn saturate(dfa: &DFA, nfa: &mut NFA, tm: &Machine, a_dir: Side, q_new: NFAState, b_new: u8) {
        tm.rules().for_each(|rule| match rule {
            Rule::Move { f, r, w, d, t } if d == a_dir && w == b_new => {
                nfa.t[r as usize][nfa_start(q_new, f)] |= row(nfa_start(dfa.step(q_new, w), t));
            }
            _ => {}
        });
        loop {
            let mut grew = false;
            tm.rules().for_each(|rule| match rule {
                Rule::Move { f, r, w, d, t } if d != a_dir => {
                    'qb: for q in 0.. {
                        for b in 0..2 {
                            if (q, b) > (q_new, b_new) {
                                break 'qb;
                            }
                            let q2 = dfa.step(q, b);
                            let t_r_q2 = nfa.t[r as usize][nfa_start(q2, f)];
                            let new = nfa.step_vec(nfa.step(nfa_start(q, t), b), w);
                            nfa.t[r as usize][nfa_start(q2, f)] |= new;
                            grew |= nfa.t[r as usize][nfa_start(q2, f)] != t_r_q2;
                        }
                    }
                }
                _ => {}
            });
            if !grew {
                break;
            }
        }
        loop {
            let old = nfa.accepted;
            nfa.accepted |= &nfa.t[0] * nfa.accepted;
            if nfa.accepted == old {
                break;
            }
        }
    }
}