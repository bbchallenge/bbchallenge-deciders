//! A seed database as in https://bbchallenge.org/method.

use super::MachineID;
use crate::core::{Machine, TM_STATES};
use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;
use zerocopy::FromBytes;

const HEADER_SIZE: i64 = 30;
const RECORD_SIZE: i64 = 6 * TM_STATES as i64;

pub struct Database {
    file: File,
}

/// A seed database file, as in https://bbchallenge.org/method.
impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Database> {
        let file = File::open(path)?;
        Ok(Database { file })
    }

    pub fn read<'a, I: Iterator<Item = MachineID> + 'a>(
        &'a self,
        ids: I,
    ) -> impl Iterator<Item = (MachineID, Machine)> + 'a {
        let mut reader = BufReader::new(&self.file);
        let pos = reader.stream_position().unwrap() as i64;
        Reader {
            reader,
            ids,
            bytes: [0u8; 6 * TM_STATES],
            pos,
        }
    }

    pub fn len(&self) -> usize {
        ((self.file.metadata().unwrap().len() as i64 - HEADER_SIZE) / RECORD_SIZE) as usize
    }
}

/// An iterator of the machines in a database.
pub struct Reader<'a, I: Iterator<Item = MachineID> + 'a> {
    reader: BufReader<&'a File>,
    ids: I,
    bytes: [u8; 6 * TM_STATES],
    pos: i64,
}

impl<'a, I: Iterator<Item = MachineID> + 'a> Reader<'a, I> {
    fn try_seek(&mut self, i: MachineID) -> io::Result<()> {
        let old = self.pos;
        let new = HEADER_SIZE + (i as i64) * RECORD_SIZE;
        self.pos = new + RECORD_SIZE;
        self.reader.seek_relative(new - old)
    }
}

impl<'a, I: Iterator<Item = MachineID> + 'a> Iterator for Reader<'a, I> {
    type Item = (MachineID, Machine);
    fn next(&mut self) -> Option<Self::Item> {
        self.ids.next().and_then(|i| {
            self.try_seek(i)
                .ok()
                .and_then(|_| self.reader.read_exact(&mut self.bytes).ok())
                .and_then(|_| Machine::read_from(&self.bytes as &[u8]))
                .map(|tm| (i, tm))
        })
    }
}
