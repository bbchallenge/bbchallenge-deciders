//! Definitions of Boolean matrices and their operations, implemented using fast bitwise operations.

use super::{BadProof, NFAState, NFAStateMask, ProofResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{BitOr, BitOrAssign, Index, IndexMut, Mul};

/// A Boolean column vector, representing a set of NFAStates (to test against).
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, Eq, PartialEq)]
#[serde(transparent)]
pub struct ColVector(NFAStateMask);

/// A Boolean row vector, representing a set of NFAStates (which the NFA has reached).
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, Eq, PartialEq)]
#[serde(transparent)]
pub struct RowVector(NFAStateMask);

/// A Boolean square matrix, representing a transition.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(transparent)]
pub struct Matrix {
    rows: Vec<RowVector>,
}

/// Iterator for the indices of set bits (i.e., the NFAStates in the set being represented).
pub struct IterBits(NFAStateMask);

fn bits_from_iter<I: IntoIterator<Item = NFAState>>(iter: I) -> NFAStateMask {
    iter.into_iter()
        .fold(0 as NFAStateMask, |v, i| v | (1 as NFAStateMask) << i)
}

/// Ensure mask uses only the first n bits.
fn validate_mask(mask: NFAStateMask, n: usize) -> ProofResult<()> {
    if (mask >> n) == 0 {
        Ok(())
    } else {
        Err(BadProof::BadVector)
    }
}

/// Implement the partial order given by bitwise inclusion.
fn bitwise_partial_cmp(l: NFAStateMask, r: NFAStateMask) -> Option<Ordering> {
    let intersection = l & r;
    if l == r {
        Some(Ordering::Equal)
    } else if l == intersection {
        Some(Ordering::Less)
    } else if r == intersection {
        Some(Ordering::Greater)
    } else {
        None
    }
}

impl ColVector {
    /// The zero vector.
    pub fn new() -> ColVector {
        ColVector(0)
    }

    /// The standard basis vectors, representing sets of the form {i}.
    pub fn e(i: NFAState) -> ColVector {
        ColVector((1 as NFAStateMask) << i)
    }

    /// Ensure self is valid as an n-dimensional vector.
    pub fn validate(self, n: usize) -> ProofResult<()> {
        validate_mask(self.0, n)
    }
}

impl BitOr for ColVector {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ColVector {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl PartialOrd for ColVector {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        bitwise_partial_cmp(self.0, rhs.0)
    }
}

impl RowVector {
    /// The zero vector.
    pub fn new() -> RowVector {
        RowVector(0)
    }

    /// The standard basis vectors, representing sets of the form {i}.
    pub fn e(i: NFAState) -> RowVector {
        RowVector((1 as NFAStateMask) << i)
    }

    /// Ensure self is valid as an n-dimensional vector.
    pub fn validate(self, n: usize) -> ProofResult<()> {
        validate_mask(self.0, n)
    }
}

impl BitOr for RowVector {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for RowVector {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl PartialOrd for RowVector {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        bitwise_partial_cmp(self.0, rhs.0)
    }
}

impl Mul<ColVector> for RowVector {
    type Output = bool;
    fn mul(self, rhs: ColVector) -> bool {
        (self.0 & rhs.0) != 0
    }
}

impl Iterator for IterBits {
    type Item = NFAState;
    fn next(&mut self) -> Option<NFAState> {
        if self.0 == 0 {
            None
        } else {
            let lsb = self.0.trailing_zeros() as NFAState;
            self.0 &= self.0.wrapping_sub(1);
            Some(lsb)
        }
    }
}

impl IntoIterator for ColVector {
    type Item = NFAState;
    type IntoIter = IterBits;
    fn into_iter(self) -> IterBits {
        IterBits(self.0)
    }
}

impl IntoIterator for RowVector {
    type Item = NFAState;
    type IntoIter = IterBits;
    fn into_iter(self) -> IterBits {
        IterBits(self.0)
    }
}

impl FromIterator<NFAState> for ColVector {
    fn from_iter<I: IntoIterator<Item = NFAState>>(iter: I) -> Self {
        Self(bits_from_iter(iter))
    }
}

impl FromIterator<NFAState> for RowVector {
    fn from_iter<I: IntoIterator<Item = NFAState>>(iter: I) -> Self {
        Self(bits_from_iter(iter))
    }
}

impl Matrix {
    /// An n x n matrix of zeros.
    pub fn new(n: usize) -> Matrix {
        Matrix {
            rows: vec![RowVector(0); n],
        }
    }

    /// Ensure self is valid as an n x n matrix.
    pub fn validate(&self) -> ProofResult<()> {
        self.into_iter().try_for_each(|v| v.validate(self.len()))
    }

    /// The dimension of the vectors self operates on.
    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

impl Index<NFAState> for Matrix {
    type Output = RowVector;
    fn index(&self, i: NFAState) -> &Self::Output {
        &self.rows[i as usize]
    }
}

impl IndexMut<NFAState> for Matrix {
    fn index_mut(&mut self, i: NFAState) -> &mut Self::Output {
        &mut (self.rows[i as usize])
    }
}

impl<'a> IntoIterator for &'a Matrix {
    type Item = &'a RowVector;
    type IntoIter = std::slice::Iter<'a, RowVector>;
    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

impl Mul<&Matrix> for RowVector {
    type Output = RowVector;
    fn mul(self, rhs: &Matrix) -> RowVector {
        self.into_iter().fold(RowVector(0), |v, pos| v | rhs[pos])
    }
}

impl Mul<ColVector> for &Matrix {
    type Output = ColVector;
    fn mul(self, rhs: ColVector) -> ColVector {
        self.into_iter()
            .enumerate()
            .filter_map(|(i, &v)| if v * rhs { Some(i as NFAState) } else { None })
            .collect::<ColVector>()
    }
}

/// Synonym for `RowVector::e(i)`.
pub fn row(i: NFAState) -> RowVector {
    RowVector::e(i)
}

/// Synonym for `ColVector::e(i)`.
pub fn col(i: NFAState) -> ColVector {
    ColVector::e(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        assert_eq!(ColVector::new().validate(0), Ok(()));
        assert_eq!(RowVector::new().validate(0), Ok(()));
        for n in 0..3 {
            for i in 0..3 {
                assert_eq!(row(i).validate(n).is_ok(), (i as usize) < n);
                assert_eq!(col(i).validate(n).is_ok(), (i as usize) < n);
            }
        }
    }

    #[test]
    fn test_or() {
        let (bra_0, bra_1, ket_0, ket_1) = (row(0), row(1), col(0), col(1));
        // Commutative "|" operation, compatible "|=":
        let or_a = bra_0 | bra_1;
        let mut or_b = bra_1;
        or_b |= bra_0;
        assert_eq!(or_a, or_b);
        let or_a = ket_0 | ket_1;
        let mut or_b = ket_1;
        or_b |= ket_0;
        assert_eq!(or_a, or_b);
    }

    #[test]
    fn test_iteration() {
        let states_set: Vec<NFAState> = (row(0) | row(2) | row(5)).into_iter().collect();
        assert_eq!(states_set, vec![0, 2, 5]);
        let states_set: Vec<NFAState> = (col(1) | col(3) | col(5)).into_iter().collect();
        assert_eq!(states_set, vec![1, 3, 5]);
    }

    #[test]
    fn test_comparison() {
        let (lt, gt) = (Some(Ordering::Less), Some(Ordering::Greater));
        let eq = Some(Ordering::Equal);
        assert_eq!(RowVector(0b00).partial_cmp(&RowVector(0b01)), lt);
        assert_eq!(ColVector(0b01).partial_cmp(&ColVector(0b01)), eq);
        assert_eq!(RowVector(0b11).partial_cmp(&RowVector(0b10)), gt);
        assert_eq!(ColVector(0b01).partial_cmp(&ColVector(0b10)), None);
    }

    #[test]
    fn test_inner_product() {
        for i in 0 as NFAState..2 as NFAState {
            for j in 0 as NFAState..2 as NFAState {
                assert_eq!(row(i) * col(j), (i == j));
                // distributive law
                for i2 in 0 as NFAState..2 as NFAState {
                    for j2 in 0 as NFAState..2 as NFAState {
                        assert_eq!(
                            (row(i) | row(i2)) * col(j),
                            row(i) * col(j) | row(i2) * col(j)
                        );
                        assert_eq!(
                            row(i) * (col(j) | col(j2)),
                            row(i) * col(j) | row(i) * col(j2)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_matrix_ops() {
        // Construct the identity matrix [1, 0; 0, 1], bit by bit:
        let mut id = Matrix::new(2);
        id[0] |= row(0);
        id[1] |= row(1);
        // Construct the upper-triangular matrix [1, 1; 0, 1], bit by bit:
        let mut upper_tri = Matrix::new(2);
        upper_tri[0] |= id[0];
        upper_tri[1] |= id[1];
        upper_tri[0] |= row(1);
        // Try to extract the elements using multiplications with the standard basis (co)vectors.
        for i in 0 as NFAState..2 as NFAState {
            for j in 0 as NFAState..2 as NFAState {
                assert_eq!((row(i) * &upper_tri) * col(j), (i <= j));
                // The associative property should make this the same:
                assert_eq!(row(i) * (&upper_tri * col(j)), (i <= j));
            }
        }
    }

    #[test]
    fn test_serialization() {
        let ones: Matrix = serde_json::from_str("[3, 3]").unwrap();
        assert!(ones.validate().is_ok());
        assert_eq!(ones[0], row(0) | row(1));
        assert_eq!(ones[1], row(0) | row(1));
        let oops_2_by_1: Matrix = serde_json::from_str("[3]").unwrap();
        assert!(oops_2_by_1.validate().is_err());
    }
}
