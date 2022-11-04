//! The library of actual Proof-finding code. The interface is defined in prover.rs.
//! Other files implement individual search methods.
//! This code should enjoy more freedom than the core code, since the latter will verify all proofs
//! before marking machines as decided.

mod dfa_iterator;
mod direct;
mod mitm_dfa;
mod prover;

pub use dfa_iterator::{DFAIterator, DFAPrefixIterator};
pub use direct::DirectProver;
pub use mitm_dfa::MitMDFAProver;
pub use prover::{
    prover_by_name, prover_names, prover_range_by_name, Prover, ProverBox, ProverOptions,
};
