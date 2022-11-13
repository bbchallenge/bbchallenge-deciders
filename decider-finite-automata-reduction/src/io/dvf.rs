//! A Decider Verification File -- as in https://github.com/TonyGuil/bbchallenge.

use super::MachineID;
use crate::core::{Side, DFA};
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// Magic value for specifying the "VerificationEntry" is from this program.
const DECIDER_TYPE: u32 = 6;

/// An output file listing decided machine indexes.
pub struct DeciderVerificationFile {
    out: BufWriter<File>,
}

impl DeciderVerificationFile {
    /// Open the given DVF file for appending. (The "nEntries" value is left as 0.)
    pub fn append<P: AsRef<Path>>(path: P) -> io::Result<DeciderVerificationFile> {
        if let Ok(mut new_file) = OpenOptions::new().write(true).create_new(true).open(&path) {
            new_file.write_all(&[0u8; 4])?;
        }
        let inner = OpenOptions::new().append(true).open(path)?;
        let out = BufWriter::new(inner);
        Ok(DeciderVerificationFile { out })
    }

    /// Mark the given machine as solved.
    pub fn insert(&mut self, id: MachineID, direction: Side, dfa: &DFA) -> io::Result<()> {
        for int in [id, DECIDER_TYPE, (1 + 2 * dfa.len()) as u32] {
            self.out.write_all(&int.to_be_bytes())?;
        }
        let dir = match direction {
            Side::R => 0u8,
            Side::L => 1u8,
        };
        self.out.write_all(&[dir])?;
        for pair in &dfa.t {
            self.out.write_all(pair)?;
        }
        self.out.flush()
    }
}
