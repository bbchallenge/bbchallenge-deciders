//! An implementation of the DB and index formats from https://bbchallenge.org/method.

mod db;
mod dvf;
mod index;
mod output;

pub use db::Database;
pub use dvf::DeciderVerificationFile;
pub use index::Index;
pub use output::OutputFile;

pub type MachineID = u32;
