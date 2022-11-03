//! An implementation of the DB and index formats from https://bbchallenge.org/method.

mod db;
mod index;
mod output;

pub use db::Database;
pub use index::Index;
pub use output::OutputFile;

use std::time::SystemTime;

pub type MachineID = u32;

/// Utility function returning a unix timestamp, used in filenames.
pub fn timestamp() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(dt) => dt.as_secs(),
        Err(_) => 0, // time travelers: do not use before 1970.
    }
}
