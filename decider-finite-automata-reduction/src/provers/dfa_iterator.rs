//! Iterators for DFAs (on alphabet {0,1}), and prefixes (incomplete transition tables) thereof.

use crate::core::{DFAState, DFA};

/// Iterates over possible n-state DFAs, subject to two restrictions:
/// 1. `DFA::check_leading_zeros()` passes.
/// 2. The DFA states are ordered the same as a breadth-first search from the initial state
///    (taking 0-transitions before 1-transitions). In particular, it's connected and the
///    `DFAState` IDs `0..n` make their first appearances in order in the transition table
///    (which in Rust that would be: in `dfa.t.iter().flatten()`.)
///
/// Condition 2 stops us from emitting two isomorphic DFAs.
///
/// The iterator has two flavors. `DFAPrefixIterator` yields once per (nonempty) partially-filled
/// transition table, so long as it's a prefix of a total DFA following rules 1-2.
/// `DFAIterator` yields once per completed DFA.
/// `DFAPrefixIterator` supports an additional method, `skip_current_subtree()`, if the caller
/// is uninterested in DFAs starting with the most recently yielded prefix.
///
/// We don't yield references to the DFA. (Rust only recently added "LendingIterator" support.)
/// Instead: the dfa is accessible as a field, DFAPrefixIterator yields the changed `(q, b)` index,
/// and DFAIterator yields `()`.
pub struct DFAPrefixIterator {
    /// The DFA under construction.
    pub dfa: DFA,
    /// The DFA has elements `[q][b]` filled in for `2*q+b < qb`.
    qb: usize,
    /// For each qb, `max{dfa.t[q][b] | 2*q+b < qb}`.
    tmax: Vec<DFAState>,
    /// Whether we've been asked to skip everything starting with the current prefix.
    skip_current: bool,
}

/// See `DFAPrefixIterator`.
pub struct DFAIterator(pub DFAPrefixIterator);

impl DFAPrefixIterator {
    pub fn new(n: usize) -> Self {
        Self {
            dfa: DFA::new(n),
            qb: 0,
            tmax: vec![0; 2 * n + 1],
            skip_current: false,
        }
    }

    pub fn skip_current_subtree(&mut self) {
        self.skip_current = true;
    }

    fn qb_pair(&self) -> (usize, usize) {
        (self.qb / 2, self.qb % 2)
    }
}

impl DFAIterator {
    pub fn new(n: usize) -> Self {
        Self(DFAPrefixIterator::new(n))
    }
}

impl Iterator for DFAPrefixIterator {
    type Item = (DFAState, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let m = (self.dfa.len() - 1) as DFAState;
        // If the table wasn't full yet, but we've promised it can be filled, the next prefix
        // is the first extension of the current one.
        if self.qb < 2 * self.dfa.len() && !self.skip_current {
            let (q, b) = self.qb_pair();
            // Case 1: the next entry must be an unvisited state (thus the first one).
            // That's the case if doing otherwise would close the transition graph, early -- i.e.,
            // states `> tmax[qb]` exist and would become unreachable from ones `<= tmax[qb]`.
            // Case 2: no such restriction, so the lex-next table fills in a 0.
            if self.tmax[self.qb] < m && self.qb == 2 * (self.tmax[self.qb] as usize) + 1 {
                self.dfa.t[q][b] = self.tmax[self.qb] + 1;
            } else {
                self.dfa.t[q][b] = 0 as DFAState;
            }
            self.qb += 1;
            self.tmax[self.qb] = std::cmp::max(self.tmax[self.qb - 1], self.dfa.t[q][b]);
            return Some((q as DFAState, b as u8));
        }
        self.skip_current = false;
        // If instead the table was full, we have to backtrack (or "carry" as we count?)
        // After we backtrack, either incrementing the current transition is the lex-first option,
        // or raising it any further would violate rules 1-2 or exit 0..n - backtrack further if so.
        while self.qb > 1 {
            self.qb -= 1;
            let (q, b) = self.qb_pair();
            if self.dfa.t[q][b] <= self.tmax[self.qb] && self.dfa.t[q][b] < m {
                self.dfa.t[q][b] += 1;
                self.qb += 1;
                self.tmax[self.qb] = std::cmp::max(self.tmax[self.qb - 1], self.dfa.t[q][b]);
                return Some((q as DFAState, b as u8));
            }
        }
        None
    }
}

impl Iterator for DFAIterator {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let q_max = (self.0.dfa.len() - 1) as DFAState;
            if self.0.next()? == (q_max, 1) {
                return Some(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_state() {
        // It's actually pretty easy to screw this up. :-)
        let mut it = DFAIterator::new(1);
        assert_eq!(it.next(), Some(()));
        assert_eq!(it.0.dfa.t, [[0, 0]]);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_2_states() {
        let mut it = DFAIterator::new(2);
        assert_eq!(it.next(), Some(()));
        assert_eq!(it.0.dfa.t, [[0, 1], [0, 0]]);
        assert_eq!(it.next(), Some(()));
        assert_eq!(it.0.dfa.t, [[0, 1], [0, 1]]);
        assert_eq!(it.next(), Some(()));
        assert_eq!(it.0.dfa.t, [[0, 1], [1, 0]]);
        assert_eq!(it.next(), Some(()));
        assert_eq!(it.0.dfa.t, [[0, 1], [1, 1]]);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_skips() {
        let mut it = DFAPrefixIterator::new(3);
        assert_eq!(it.next(), Some((0, 0)));
        assert_eq!(it.dfa.t, [[0, 0], [0, 0], [0, 0]]);
        assert_eq!(it.next(), Some((0, 1)));
        assert_eq!(it.dfa.t, [[0, 1], [0, 0], [0, 0]]);
        assert_eq!(it.next(), Some((1, 0)));
        assert_eq!(it.dfa.t, [[0, 1], [0, 0], [0, 0]]);
        it.skip_current_subtree(); // Done with 0, 1, 0, _
        assert_eq!(it.next(), Some((1, 0)));
        assert_eq!(it.dfa.t, [[0, 1], [1, 0], [0, 0]]);
        assert_eq!(it.next(), Some((1, 1)));
        assert_eq!(it.dfa.t, [[0, 1], [1, 2], [0, 0]]);
        it.skip_current_subtree(); // Done with 0, 1, 1, 2, _
        assert_eq!(it.next(), Some((1, 0)));
        assert_eq!(it.dfa.t, [[0, 1], [2, 2], [0, 0]]);
        // This entry here is irrelevant: ^  -- caller may not rely on it being zeroed.
        it.skip_current_subtree(); // Done with 0, 1, 2, _
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_counts() {
        // Counts of complete 0-insensitive binary DFAs follow https://oeis.org/A107668
        assert_eq!(DFAIterator::new(1).count(), 1);
        assert_eq!(DFAIterator::new(2).count(), 4);
        assert_eq!(DFAIterator::new(3).count(), 45);
        assert_eq!(DFAIterator::new(4).count(), 816);
    }
}
