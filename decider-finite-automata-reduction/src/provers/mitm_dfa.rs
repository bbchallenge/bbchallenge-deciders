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
    solver: Solver,
    ready: bool,
}

impl Prover for MitMDFAProver {
    fn name(&self) -> String {
        format!("mitm_dfa({})", self.n)
    }

    fn prove(&mut self, tm: &Machine) -> Option<Proof> {
        if !self.ready {
            self.init(self.n);
        }
        let assumptions = tm.rules().flat_map(|rule| Self::tm_clause(rule));
        if self.solver.solve_with(assumptions) == Some(true) {
            let mut dfa = DFA::new(self.n as usize);
            for q in 0..dfa.len() {
                for b in 0..2 {
                    dfa.t[q][b] = self.dfa_eval(FROM_LEFT, q as DFAState, b as u8);
                }
            }
            DirectProver::complete_unverified(&tm, Side::R, dfa)
        } else {
            None
        }
    }
}

impl ProverOptions for MitMDFAProver {
    fn new(depth: usize) -> Self {
        let (solver, n, ready) = (Solver::new(), depth as L, false);
        MitMDFAProver { n, solver, ready }
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
    fn _tm_write(        f: L, r: L)       -> L { f + T*r             + 2 }
    fn _tm_right(        f: L, r: L)       -> L { f + T*r             + T*2 + 2 }
    fn _tm_to_eq(        f: L, r: L, t: L) -> L { f + T*(r + 2*t)     + T*4 + 2 }
    fn _tm_to_le(        f: L, r: L, t: L) -> L { f + T*(r + 2*(t-1)) + T*(6+T*2) + 2 }
    fn _dfa_t_eq(n: L, lr: L, qb: L, t: L) -> L { lr + 2*(qb-1 + a(2*n  , t  )) + 4*T*(T+1) + 2 }
    fn _dfa_t_le(n: L, lr: L, qb: L, t: L) -> L { lr + 2*(qb-2 + a(2*n-2, t-1)) + 4*T*(T+1) + n*(1+3*n) }
    fn _accepted(n: L, ql: L, f: L, r: L, qr: L) -> L { ql + n*(f + T*(r + 2*qr)) + 4*T*(T+1) + 6*n*(n-1) + 1 }
    fn _aux_var0(n: L) -> L { n*T*2*n + 4*T*(T+1) + 6*n*(n-1) + 1 }
}

impl MitMDFAProver {
    fn negate_if_0(lit: L, w: u8) -> L {
        match w {
            0 => -lit,
            _ => lit,
        }
    }

    fn negate_if_l(lit: L, d: Side) -> L {
        match d {
            Side::L => -lit,
            Side::R => lit,
        }
    }

    fn tm_clause(rule: Rule) -> [L; 3] {
        match rule {
            Rule::Move { f, r, w, d, t } => [
                Self::negate_if_0(Self::_tm_write(f as L, r as L), w),
                Self::negate_if_l(Self::_tm_right(f as L, r as L), d),
                Self::_tm_to_eq(f as L, r as L, t as L),
            ],
            Rule::Halt { f, r } => [Self::_tm_to_eq(f as L, r as L, T), TRUE, TRUE],
        }
    }

    fn tm_to(&self, f: TMState, r: u8, t: L) -> L {
        if 0 <= t && t <= T {
            Self::_tm_to_eq(f as L, r as L, t)
        } else {
            FALSE
        }
    }

    fn tm_to_le(&self, f: TMState, r: u8, t: L) -> L {
        if t <= 0 {
            self.tm_to(f, r, t)
        } else if t < T {
            Self::_tm_to_le(f as L, r as L, t)
        } else {
            TRUE
        }
    }

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

    fn dfa_eval(&self, lr: i32, q: DFAState, b: u8) -> DFAState {
        // We have the less-or-equal variables solved, but a linear search is more than fast enough.
        for t in 0..(self.n as DFAState) {
            if self.value(self.dfa(lr, q, b, t as L)) {
                return t;
            }
        }
        unreachable!("The transition has a value in every model.");
    }

    /// Evaluate a variable or negation, making arbitrary choices if needed.
    fn value(&self, lit: L) -> bool {
        self.solver.value(lit).unwrap_or(lit > 0)
    }

    fn add<I: IntoIterator<Item = L>>(&mut self, clause: I) {
        self.solver.add_clause(clause.into_iter())
    }

    fn init(&mut self, n: L) {
        self.ready = true;
        self.add([TRUE]);
        // TM transition outcomes are mutually exclusive.
        for f in 0..(T as TMState) {
            for r in 0..2 {
                for t in 0..=T {
                    self.add([-self.tm_to(f, r, t), self.tm_to_le(f, r, t)]);
                    self.add([-self.tm_to_le(f, r, t), self.tm_to_le(f, r, t + 1)]);
                    self.add([-self.tm_to(f, r, t + 1), -self.tm_to_le(f, r, t)]);
                }
            }
        }
        // DFA transitions:
        for lr in 0..2 {
            for q in 0..(n as DFAState) {
                for b in 0..2 {
                    // Outcomes are mutually exclusive.
                    for t in 0..n {
                        self.add([-self.dfa(lr, q, b, t), self.dfa_le(lr, q, b, t)]);
                        self.add([-self.dfa_le(lr, q, b, t), self.dfa_le(lr, q, b, t + 1)]);
                        self.add([-self.dfa(lr, q, b, t + 1), -self.dfa_le(lr, q, b, t)]);
                    }
                    // An outcome occurs.
                    if (q, b) != (0, 0) {
                        let tmax = min(2 * (q as L) + (b as L) + 1, n);
                        let qb = 2 * (q as L) + (b as L);
                        self.add((0..tmax).map(|t| Self::_dfa_t_eq(n, lr, qb, t)));
                    }
                }
            }
        }
        // Closure conditions:
        for ql in 0..(n as DFAState) {
            for qr in 0..(n as DFAState) {
                for f in 0..(T as TMState) {
                    for r in 0..2 {
                        self.add([-self.tm_to(f, r, T), self.accept(ql, f, r, qr)]);
                        let cr = Self::_tm_right(f as L, r as L);
                        for w in 0..2 {
                            let cw = Self::negate_if_0(Self::_tm_write(f as L, r as L), w);
                            for t in 0..(T as TMState) {
                                let ct = self.tm_to(f, r, t as L);
                                for b in 0..2 {
                                    for qw in 0..(n as DFAState) {
                                        for qb in 0..(n as DFAState) {
                                            // Closure under left TM transitions, b f@r -> t@b w
                                            self.add([
                                                self.accept(qb, f, r, qr),
                                                -cw,
                                                cr,
                                                -ct,
                                                -self.dfa(FROM_LEFT, ql, b, qb as L),
                                                -self.dfa(FROM_RIGHT, qr, w, qw as L),
                                                -self.accept(ql, t, b, qw),
                                            ]);
                                            // Closure under right TM transitions, f@r b -> w t@b
                                            self.add([
                                                self.accept(ql, f, r, qb),
                                                -cw,
                                                -cr,
                                                -ct,
                                                -self.dfa(FROM_RIGHT, qr, b, qb as L),
                                                -self.dfa(FROM_LEFT, ql, w, qw as L),
                                                -self.accept(qw, t, b, qr),
                                            ]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
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
        fn tmax_eq(n: L, lr: L, qb: L, m: L, base: &Vec<L>) -> L {
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
                    self.add([-tmax_eq(n, lr, qb, m, &base), self.dfa_le(lr, q, b, m + 1)]);
                    self.add([
                        -tmax_eq(n, lr, qb, m, &base),
                        -self.dfa_le(lr, q, b, m),
                        tmax_eq(n, lr, qb + 1, m, &base),
                    ]);
                    self.add([
                        -tmax_eq(n, lr, qb, m, &base),
                        -self.dfa(lr, q, b, m + 1),
                        tmax_eq(n, lr, qb + 1, m + 1, &base),
                    ]);
                }
            }
        }
    }
}
