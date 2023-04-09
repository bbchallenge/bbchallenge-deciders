//! An output index file -- as in https://bbchallenge.org/method.

use super::MachineID;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// An output file listing decided machine indexes.
pub struct OutputFile {
    out: File,
}

impl OutputFile {
    /// Open the given (output) index file for appending.
    pub fn append<P: AsRef<Path>>(path: P) -> io::Result<OutputFile> {
        let out = OpenOptions::new().append(true).create(true).open(path)?;
        Ok(OutputFile { out })
    }

    /// Mark the given machine as solved.
    pub fn insert(&mut self, id: MachineID) -> io::Result<()> {
        self.out.write(&id.to_be_bytes()).and(Ok(()))
    }
}
