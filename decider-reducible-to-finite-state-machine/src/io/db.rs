//! A seed database as in https://bbchallenge.org/method.

use crate::core::{Machine, TM_STATES};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use zerocopy::FromBytes;

const HEADER_SIZE: usize = 30;

pub struct Database {
    file: File,
}

/// A seed database file, as in https://bbchallenge.org/method.
impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Database> {
        let file = File::open(path)?;
        Ok(Database { file })
    }

    pub fn read<'a, I: Iterator<Item = usize> + 'a>(
        &'a self,
        ids: I,
    ) -> impl Iterator<Item = (usize, Machine)> + 'a {
        Reader {
            reader: BufReader::new(&self.file),
            ids,
            bytes: [0u8; 6 * TM_STATES],
            pos: 0,
        }
    }
}

/// An iterator of the machines in a database.
pub struct Reader<'a, I: Iterator<Item = usize> + 'a> {
    reader: BufReader<&'a File>,
    ids: I,
    bytes: [u8; 6 * TM_STATES],
    pos: usize,
}

impl<'a, I: Iterator<Item = usize> + 'a> Reader<'a, I> {
    fn try_seek(&mut self, i: usize) -> std::io::Result<()> {
        let pos = self.pos;
        self.pos = i + 2;
        if i + 1 != pos {
            self.reader
                .seek(SeekFrom::Start((HEADER_SIZE + 6 * TM_STATES * i) as u64))?;
        }
        Ok(())
    }
}

impl<'a, I: Iterator<Item = usize> + 'a> Iterator for Reader<'a, I> {
    type Item = (usize, Machine);
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
