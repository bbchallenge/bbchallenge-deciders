//! This prover searches for a simpler form of recognizing automaton:
//! Consume the tape on both ends with DFAs. Exclude the bit `r` under the head (state `f`).
//! When we "meet in the middle", we have a tuple `(qL, f, r, qR)`.
//! We define a subset of these tuples as accepted, subject to the start/halt/closure rules.
//! Searching for useful MitM-DFA recognizers can take forever, so we make a SAT solver do it.
//! This gives a `TapeAutomaton` if we build an NFA from states `nfa_start(f, r)` plus each `qR`.
//! However, it's simpler to hand our DirectProver the left DFA and let it finish.
//!
//! The same DFA-pair/SAT technique was pioneered by others in the bbchallenge community:
//! - @djmati1111 (https://github.com/colette-b/bbchallenge)
//! - @Mateon1 (https://discuss.bbchallenge.org/u/mateon1)

use super::{DirectProver, Prover, ProverOptions};
use crate::core::{DFAState, Machine, Proof, Rule, Side, TMState, DFA, TM_STATES};
use cadical::Solver;
use std::cmp::min;

/// A prover which searches for "Meet-in-the-Middle DFA" recognizers.
pub struct MitMDFAProver {
    n: i32,
}

trait BetterAddClause {
    fn add<I: IntoIterator<Item = L>>(&mut self, clause: I);
}

impl BetterAddClause for Solver {
    fn add<I: IntoIterator<Item = L>>(&mut self, clause: I) {
        self.add_clause(clause.into_iter())
    }
}

impl Prover for MitMDFAProver {
    fn name(&self) -> String {
        format!("mitm_dfa-{}", self.n)
    }

    fn prove(&mut self, tm: &Machine) -> Option<Proof> {
        let mut solver = self.init(self.n, tm);
        if solver.solve() == Some(true) {
            let mut dfa = DFA::new(self.n as usize);
            for q in 0..dfa.len() {
                for b in 0..2 {
                    dfa.t[q][b] = self.dfa_eval(&solver, FROM_LEFT, q as DFAState, b as u8);
                }
            }
            DirectProver::complete_unverified(tm, Side::R, dfa)
        } else {
            None
        }
    }
}

impl ProverOptions for MitMDFAProver {
    fn new(depth: usize) -> Self {
        MitMDFAProver { n: depth as L }
    }
}

// SAT solvers speak CNF: *literals* are + `i32`s (variables) and their negations (-x means NOT x).
// Lists represent disjunctions (OR); the conjunction (AND) of all added clauses must be true.
// When it's not ludicrous, we pack the conditions of interest tightly into a sequence of variables.
// When I represent "the" result of a TM or DFA transition, I use sequential at-most-one conditions.
// That is, `eq`/`le` variables represent the outcome being `=`/`<=` each fixed value, with rules:
// `x = k` implies `x <= k` implies `x <= k+1` and `x != k+1`.
// See also: https://www.carstensinz.de/papers/CP-2005.pdf

type L = i32;
const TRUE: L = 1;
const FALSE: L = -TRUE;
const T: L = TM_STATES as L;
const FROM_LEFT: L = 0;
const FROM_RIGHT: L = 1;

/// Number of lattice points in the trapezoid `0 <= y < min(x, h)`, `0 <= x < b`.
fn a(b: L, h: L) -> L {
    let s = min(b, h);
    (s * (s - 1)) / 2 + (b - s) * h
}

#[rustfmt::skip]
impl MitMDFAProver {
    fn _dfa_t_eq(n: L, lr: L, qb: L, t: L) -> L { lr + 2*(qb-1 + a(2*n  , t  )) + 2 }
    fn _dfa_t_le(n: L, lr: L, qb: L, t: L) -> L { lr + 2*(qb-2 + a(2*n-2, t-1)) + n*(1+3*n) }
    fn _accepted(n: L, ql: L, f: L, r: L, qr: L) -> L { ql + n*(f + T*(r + 2*qr)) + 6*n*(n-1) + 1 }
    fn _aux_var0(n: L) -> L { n*T*2*n + 6*n*(n-1) + 1 }
}

impl MitMDFAProver {
    fn dfa(&self, lr: i32, q: DFAState, b: u8, t: L) -> L {
        let qb = 2 * (q as L) + (b as L);
        if (qb, t) == (0, 0) {
            TRUE
        } else if 0 <= t && t <= qb && t < self.n {
            Self::_dfa_t_eq(self.n, lr, qb, t)
        } else {
            FALSE
        }
    }

    fn dfa_le(&self, lr: i32, q: DFAState, b: u8, t: L) -> L {
        let qb = 2 * (q as L) + (b as L);
        if t <= 0 {
            self.dfa(lr, q, b, t)
        } else if t < qb && t < self.n - 1 {
            Self::_dfa_t_le(self.n, lr, qb, t)
        } else {
            TRUE
        }
    }

    fn accept(&self, ql: DFAState, f: TMState, r: u8, qr: DFAState) -> L {
        if (ql, f, r, qr) == (0, 0, 0, 0) {
            FALSE
        } else {
            Self::_accepted(self.n, ql as L, f as L, r as L, qr as L)
        }
    }

    fn dfa_eval(&self, solver: &Solver, lr: i32, q: DFAState, b: u8) -> DFAState {
        // We have the less-or-equal variables solved, but a linear search is more than fast enough.
        let max_val = (self.n - 1) as DFAState;
        for t in 0..max_val {
            match solver.value(self.dfa(lr, q, b, t as L)) {
                Some(true) | None => return t,
                Some(false) => {}
            }
        }
        max_val
    }

    fn init(&mut self, n: L, tm: &Machine) -> Solver {
        let mut solver = Solver::new();
        solver.add([TRUE]);
        // DFA transitions:
        for lr in 0..2 {
            for q in 0..(n as DFAState) {
                for b in 0..2 {
                    // Outcomes are mutually exclusive.
                    for t in 0..n {
                        solver.add([-self.dfa(lr, q, b, t), self.dfa_le(lr, q, b, t)]);
                        solver.add([-self.dfa_le(lr, q, b, t), self.dfa_le(lr, q, b, t + 1)]);
                        solver.add([-self.dfa(lr, q, b, t + 1), -self.dfa_le(lr, q, b, t)]);
                    }
                    // An outcome occurs.
                    if (q, b) != (0, 0) {
                        let tmax = min(2 * (q as L) + (b as L) + 1, n);
                        let qb = 2 * (q as L) + (b as L);
                        solver.add((0..tmax).map(|t| Self::_dfa_t_eq(n, lr, qb, t)));
                    }
                }
            }
        }
        // Closure conditions:
        for ql in 0..(n as DFAState) {
            for qr in 0..(n as DFAState) {
                tm.rules().for_each(|rule| match rule {
                    Rule::Halt { f, r } => solver.add([self.accept(ql, f, r, qr)]),
                    Rule::Move { f, r, w, d, t } => {
                        for b in 0..2 {
                            for qw in 0..(n as DFAState) {
                                for qb in 0..(n as DFAState) {
                                    if d == Side::L {
                                        // Transition: b f@r -> t@b w
                                        solver.add([
                                            -self.dfa(FROM_LEFT, ql, b, qb as L),
                                            -self.dfa(FROM_RIGHT, qr, w, qw as L),
                                            -self.accept(ql, t, b, qw),
                                            self.accept(qb, f, r, qr),
                                        ]);
                                    } else {
                                        // Transition: f@r b -> w t@b
                                        solver.add([
                                            -self.dfa(FROM_RIGHT, qr, b, qb as L),
                                            -self.dfa(FROM_LEFT, ql, w, qw as L),
                                            -self.accept(qw, t, b, qr),
                                            self.accept(ql, f, r, qb),
                                        ]);
                                    }
                                }
                            }
                        }
                    }
                });
            }
        }
        // DFA ordering criteria: as in dfa_iterator.rs, we impose an ordering criterion, which
        // forces the states to appear in order in each DFA's transition table.
        // This saves the solver from considering DFAs with unused states or relabels of prior DFAs.
        // (See also a simpler version in z3py by colette-b/@djmati1111:
        // https://github.com/colette-b/bbchallenge/blob/main/sat2_cfl.py#L64 plus chat explanation
        // https://discord.com/channels/960643023006490684/1028746861395316776/1030907938249912431.)
        // As in dfa_iterator.rs, for each qb>0, define `tmax[qb] = max{dfa.t[q][b] | 2*q+b < qb}`.
        // `tmax[qb]` is `>= qb/2` (this state must be reachable) and `< min(n, qb)`
        // (transition values can't go out of bounds or increase at a rate above 1).

        // Let's set up variables for the L/R `tmax[qb] == m`, with index base[qb]+lr+2*m.
        let mut base = vec![Self::_aux_var0(n); 2 * n as usize];
        for qb in 1..(2 * n as usize) {
            let choices = min(n as usize, qb) - (qb / 2) + 1;
            if choices > 1 && qb + 1 < 2 * (n as usize) {
                base[qb + 1] = base[qb] + 2 * (choices as L);
            }
            base[qb] -= 2 * (qb as L / 2);
        }
        fn tmax_eq(n: L, lr: L, qb: L, m: L, base: &[L]) -> L {
            if (qb, m) == (2 * n, n - 1) {
                TRUE
            } else if m < (qb / 2) || m >= min(n, qb) {
                FALSE
            } else if min(n, qb) - (qb / 2) <= 1 {
                TRUE
            } else {
                base[qb as usize] + lr + 2 * m
            }
        }
        for qb in 1..2 * n {
            let (q, b) = ((qb / 2) as DFAState, (qb % 2) as u8);
            for m in qb / 2..min(n, qb) {
                for lr in 0..2 {
                    solver.add([-tmax_eq(n, lr, qb, m, &base), self.dfa_le(lr, q, b, m + 1)]);
                    solver.add([
                        -tmax_eq(n, lr, qb, m, &base),
                        -self.dfa_le(lr, q, b, m),
                        tmax_eq(n, lr, qb + 1, m, &base),
                    ]);
                    solver.add([
                        -tmax_eq(n, lr, qb, m, &base),
                        -self.dfa(lr, q, b, m + 1),
                        tmax_eq(n, lr, qb + 1, m + 1, &base),
                    ]);
                }
            }
        }
        solver
    }
}
