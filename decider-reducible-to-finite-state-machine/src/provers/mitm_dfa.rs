//! This prover searches for a simpler form of recognizing automaton:
//! Consume the tape on both ends with DFAs. Exclude the bit `r` under the head (state `f`).
//! When we "meet in the middle", we have a tuple `(qL, f, r, qR)`.
//! We define a subset of these tuples as accepted, subject to the start/halt/closure rules.
//! Searching for useful MitM-DFA recognizers can take forever, so we make a SAT solver do it.
//! This gives a `TapeAutomaton` if we build an NFA from states `nfa_start(f, r)` plus each `qR`.
//! However, it's simpler to hand our DirectProver the left DFA and let it finish.

use super::{DirectProver, Prover};
use crate::core::{
    DFAState, Machine, RowVector, Rule, Side, TMState, TapeAutomaton, DFA, TM_STATES,
};
use cadical::Solver;
use std::cmp::min;

/// A prover which searches for "Meet-in-the-Middle DFA" recognizers.
pub struct MitMDFAProver {
    n: i32,
    solver: Solver,
    ready: bool,
}

impl Prover for MitMDFAProver {
    fn name() -> &'static str {
        "mitm_dfa"
    }

    fn steady_state(depth: usize) -> RowVector {
        DirectProver::steady_state(depth)
    }

    fn new(depth: usize) -> Self {
        let solver = Solver::new();
        let n = depth as V;
        let ready = false;
        MitMDFAProver { n, solver, ready }
    }

    fn prove(&mut self, tm: &Machine) -> Option<TapeAutomaton> {
        if !self.ready {
            self.init();
        }
        let assumptions = tm
            .rules()
            .flat_map(|rule| Self::tm_clause(rule))
            .filter(|&x| x != TRUE);
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

// The SAT solver speaks CNF: axioms are lists of variables (+ i32) and negations (- i32) to OR.
// Two i32s lack distinct negations. We use them as sentinels (internally only - solvers do too!)
// When it's not ludicrous, we pack the conditions of interest tightly into a sequence of variables.

type V = i32;
const FALSE: V = V::MIN;
const TRUE: V = 0;
const T: V = TM_STATES as V;
const FROM_LEFT: i32 = 0;
const FROM_RIGHT: i32 = 1;

/// Number of lattice points in the trapezoid `0 <= y < min(x, h)`, `0 <= x < b`.
fn a(b: V, h: V) -> V {
    let s = min(b, h);
    (s * (s - 1)) / 2 + (b - s) * h
}

#[rustfmt::skip]
impl MitMDFAProver {
    fn _tm_write(        f: V, r: V)       -> V { f + T*r             + 1 }
    fn _tm_right(        f: V, r: V)       -> V { f + T*r             + 1 + T*2 }
    fn _tm_to_eq(        f: V, r: V, t: V) -> V { f + T*(r + 2*t)     + 1 + T*4 }
    fn _tm_to_le(        f: V, r: V, t: V) -> V { f + T*(r + 2*(t-1)) + 1 + T*(6+T*2) }
    fn _dfa_t_eq(n: V, lr: V, qb: V, t: V) -> V { lr + 2*(qb-1 + a(2*n  , t  )) + 1 + 4*T*(T+1) }
    fn _dfa_t_le(n: V, lr: V, qb: V, t: V) -> V { lr + 2*(qb-2 + a(2*n-2, t-1)) - 1 + 4*T*(T+1) + n*(1+3*n) }
    fn _accepted(n: V, ql: V, f: V, r: V, qr: V) -> V { ql + n*(f + T*(r + 2*qr)) + 4*T*(T+1) + 6*n*(n-1) }
    fn _aux_var0(n: V) -> V { n*T*2*n + 4*T*(T+1) + 6*n*(n-1) }
}

fn not(var: V) -> V {
    match var {
        FALSE => TRUE,
        TRUE => FALSE,
        _ => -var,
    }
}

impl MitMDFAProver {
    fn negate_if_0(var: V, w: u8) -> V {
        match w {
            0 => -var,
            _ => var,
        }
    }

    fn negate_if_l(var: V, d: Side) -> V {
        match d {
            Side::L => -var,
            Side::R => var,
        }
    }

    fn tm_clause(rule: Rule) -> [V; 3] {
        match rule {
            Rule::Move { f, r, w, d, t } => [
                Self::negate_if_0(Self::_tm_write(f as V, r as V), w),
                Self::negate_if_l(Self::_tm_right(f as V, r as V), d),
                Self::_tm_to_eq(f as V, r as V, t as V),
            ],
            Rule::Halt { f, r } => [Self::_tm_to_eq(f as V, r as V, T), TRUE, TRUE],
        }
    }

    fn tm_to(&self, f: TMState, r: u8, t: V) -> V {
        if 0 <= t && t <= T {
            Self::_tm_to_eq(f as V, r as V, t)
        } else {
            FALSE
        }
    }

    fn tm_to_le(&self, f: TMState, r: u8, t: V) -> V {
        if t <= 0 {
            self.tm_to(f, r, t)
        } else if t < T {
            Self::_tm_to_le(f as V, r as V, t)
        } else {
            TRUE
        }
    }

    fn dfa(&self, lr: i32, q: DFAState, b: u8, t: V) -> V {
        let qb = 2 * (q as V) + (b as V);
        if (qb, t) == (0, 0) {
            TRUE
        } else if 0 <= t && t <= qb && t < self.n {
            Self::_dfa_t_eq(self.n, lr, qb, t)
        } else {
            FALSE
        }
    }

    fn dfa_le(&self, lr: i32, q: DFAState, b: u8, t: V) -> V {
        let qb = 2 * (q as V) + (b as V);
        if t <= 0 {
            self.dfa(lr, q, b, t)
        } else if t < qb && t < self.n - 1 {
            Self::_dfa_t_le(self.n, lr, qb, t)
        } else {
            TRUE
        }
    }

    fn accept(&self, ql: DFAState, f: TMState, r: u8, qr: DFAState) -> V {
        if (ql, f, r, qr) == (0, 0, 0, 0) {
            FALSE
        } else {
            Self::_accepted(self.n, ql as V, f as V, r as V, qr as V)
        }
    }

    fn dfa_eval(&self, lr: i32, q: DFAState, b: u8) -> DFAState {
        // We have the less-or-equal variables solved, but a linear search is more than fast enough.
        for t in 0..(self.n as DFAState) {
            if self.value(self.dfa(lr, q, b, t as V)) {
                return t;
            }
        }
        unreachable!("The transition has a value in every model.");
    }

    /// Evaluate a variable or negation, or TRUE/FALSE sentinel, making arbitrary choices if needed.
    fn value(&self, var: V) -> bool {
        match var {
            FALSE => false,
            TRUE => true,
            _ => match self.solver.value(var) {
                Some(value) => value,
                None => var > 0,
            },
        }
    }
    /// Add a disjunction, understanding the FALSE and TRUE sentinels above.
    pub fn add(&mut self, clause: &[V]) {
        if !clause.iter().any(|&var| var == TRUE) {
            self.solver
                .add_clause(clause.iter().copied().filter(|&var| var != FALSE))
        }
    }

    pub fn implies(&mut self, vl: V, vr: V) {
        self.nand(vl, not(vr))
    }

    pub fn nand(&mut self, v1: V, v2: V) {
        match (v1, v2) {
            (FALSE, _) | (_, FALSE) => {}
            (TRUE, v) | (v, TRUE) => self.solver.add_clause([-v].into_iter()),
            _ => self.solver.add_clause([-v1, -v2].into_iter()),
        }
    }

    fn init(&mut self) {
        self.ready = true;
        // TM transition outcomes are mutually exclusive.
        for f in 0..(T as TMState) {
            for r in 0..2 {
                for t in 0..=T {
                    self.implies(self.tm_to(f, r, t), self.tm_to_le(f, r, t));
                    self.implies(self.tm_to_le(f, r, t), self.tm_to_le(f, r, t + 1));
                    self.nand(self.tm_to(f, r, t + 1), self.tm_to_le(f, r, t));
                }
            }
        }
        // DFA transitions:
        for lr in 0..2 {
            for q in 0..(self.n as DFAState) {
                for b in 0..2 {
                    // Outcomes are mutually exclusive.
                    for t in 0..self.n {
                        self.implies(self.dfa(lr, q, b, t), self.dfa_le(lr, q, b, t));
                        self.implies(self.dfa_le(lr, q, b, t), self.dfa_le(lr, q, b, t + 1));
                        self.nand(self.dfa(lr, q, b, t + 1), self.dfa_le(lr, q, b, t));
                    }
                    // An outcome occurs.
                    if (q, b) != (0, 0) {
                        let tmax = min(2 * (q as V) + (b as V) + 1, self.n);
                        let qb = 2 * (q as V) + (b as V);
                        self.solver
                            .add_clause((0..tmax).map(|t| Self::_dfa_t_eq(self.n, lr, qb, t)));
                    }
                }
            }
        }
        // Closure conditions:
        for ql in 0..(self.n as DFAState) {
            for qr in 0..(self.n as DFAState) {
                for f in 0..(T as TMState) {
                    for r in 0..2 {
                        self.implies(self.tm_to(f, r, T), self.accept(ql, f, r, qr));
                        let cr = Self::_tm_right(f as V, r as V);
                        for w in 0..2 {
                            let cw = Self::negate_if_0(Self::_tm_write(f as V, r as V), w);
                            for t in 0..(T as TMState) {
                                let ct = self.tm_to(f, r, t as V);
                                for b in 0..2 {
                                    for qw in 0..(self.n as DFAState) {
                                        for qb in 0..(self.n as DFAState) {
                                            // Closure under left TM transitions, b f@r -> t@b w
                                            self.add(&[
                                                self.accept(qb, f, r, qr),
                                                not(cw),
                                                cr,
                                                not(ct),
                                                not(self.dfa(FROM_LEFT, ql, b, qb as V)),
                                                not(self.dfa(FROM_RIGHT, qr, w, qw as V)),
                                                not(self.accept(ql, t, b, qw)),
                                            ]);
                                            // Closure under right TM transitions, f@r b -> w t@b
                                            self.add(&[
                                                self.accept(ql, f, r, qb),
                                                not(cw),
                                                not(cr),
                                                not(ct),
                                                not(self.dfa(FROM_RIGHT, qr, b, qb as V)),
                                                not(self.dfa(FROM_LEFT, ql, w, qw as V)),
                                                not(self.accept(qw, t, b, qr)),
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
        // As in that file, for each qb>0, define `tmax[qb] = max{dfa.t[q][b] | 2*q+b < qb}`.
        // `tmax[qb]` is `>= qb/2` (this state must be reachable) and `< min(n, qb)`
        // (transition values can't go out of bounds or increase at a rate above 1).
        // Let's set up variables for the L/R `tmax[qb] == m`, with index base[qb]+lr+2*m.
        let mut base = vec![Self::_aux_var0(self.n); 2 * self.n as usize];
        for qb in 1..(2 * self.n as usize) {
            let choices = min(self.n as usize, qb) - (qb / 2) + 1;
            if choices > 1 && qb + 1 < 2 * (self.n as usize) {
                base[qb + 1] = base[qb] + 2 * (choices as V);
            }
            base[qb] -= 2 * (qb as V / 2);
        }
        fn tmax_eq(n: V, lr: V, qb: V, m: V, base: &Vec<V>) -> V {
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
        for qb in 1..2 * self.n {
            let (q, b) = ((qb / 2) as DFAState, (qb % 2) as u8);
            for m in qb / 2..min(self.n, qb) {
                for lr in 0..2 {
                    self.implies(
                        tmax_eq(self.n, lr, qb, m, &base),
                        self.dfa_le(lr, q, b, m + 1),
                    );
                    self.add(&[
                        not(tmax_eq(self.n, lr, qb, m, &base)),
                        not(self.dfa_le(lr, q, b, m)),
                        tmax_eq(self.n, lr, qb + 1, m, &base),
                    ]);
                    self.add(&[
                        not(tmax_eq(self.n, lr, qb, m, &base)),
                        not(self.dfa(lr, q, b, m + 1)),
                        tmax_eq(self.n, lr, qb + 1, m + 1, &base),
                    ]);
                }
            }
        }
    }
}
