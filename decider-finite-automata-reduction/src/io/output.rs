//! An undecided index -- or output file -- as in https://bbchallenge.org/method.
//! Warning: this implementation is meant for use in the decider; it it'll assume an output path
//! and try to merge any checkpoint files left behind.

use super::{timestamp, MachineID};
use std::fs::{create_dir_all, File};
use std::io::{self, BufWriter, Write};

pub struct OutputFile {
    out: Option<BufWriter<File>>,
}

impl OutputFile {
    pub fn new() -> OutputFile {
        OutputFile { out: None }
    }

    /// Mark the given machine as solved.
    pub fn insert(&mut self, id: MachineID) -> io::Result<()> {
        match &mut self.out {
            Some(w) => w,
            None => {
                let path = format!("output/decided.{}", timestamp());
                create_dir_all("output")?;
                self.out
                    .insert(BufWriter::with_capacity(512, File::create(path)?))
            }
        }
        .write(&id.to_be_bytes())
        .and(Ok(()))
    }
}
