//! An undecided index -- or output file -- as in https://bbchallenge.org/method.
//! Warning: this implementation is meant for use in the decider; it it'll assume an output path
//! and try to merge any checkpoint files left behind.

use super::MachineID;
use itertools::{EitherOrBoth, Itertools};
use std::fs::{read, read_dir, remove_file, rename, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use zerocopy::{BigEndian, LayoutVerified, U32};

type IndexFile<B> = LayoutVerified<B, [U32<BigEndian>]>;

/// A data structure for tracking which DB entries are previously unsolved machines.
pub struct Index {
    yes: Vec<MachineID>,
    no: Vec<MachineID>,
    size: usize,
}

impl Index {
    /// Initialize an index where everything is considered unsolved.
    pub fn new(size: usize) -> Index {
        let mut yes = vec![0; size];
        let no = vec![0; 0];
        for i in 0..size {
            yes[i] = i as MachineID;
        }
        Index { yes, no, size }
    }

    /// Initialize an index from an "undecided" file.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Index> {
        let mut index = Self::new(0);
        Self::extend_from(&mut index.yes, path)?;
        index.yes.sort();
        index.yes.dedup();
        index.size = index.yes.len();
        Ok(index)
    }

    /// Populate the negative entries from our own output directory.
    pub fn read_decided<P: AsRef<Path>>(&mut self, dir_path: P, merge: bool) -> io::Result<()> {
        let in_paths: Vec<PathBuf> = read_dir(&dir_path)?
            .flat_map(|e| e.ok())
            .filter_map(|e| match e.path() {
                p if p.extension().map(|x| x == "ind") == Some(true) => Some(p),
                _ => None,
            })
            .collect();
        for path in in_paths.iter() {
            Self::extend_from(&mut self.no, path)?;
        }
        self.clean();
        if !merge {
            Ok(())
        } else {
            self.save(dir_path.as_ref().join("verified.ind"))?;
            in_paths.into_iter().try_for_each(|path| remove_file(path))
        }
    }

    /// Yield all unsolved `MachineID`s.
    pub fn iter(&self) -> impl Iterator<Item = MachineID> + ExactSizeIterator + '_ {
        self.yes.iter().copied()
    }

    /// Count the machines in the initial undecided-index file.
    pub fn len_initial(&self) -> usize {
        self.size
    }

    /// Count the machines excluded from the undecided-index file.
    pub fn len_solved(&self) -> usize {
        self.size - self.yes.len()
    }

    /// Count the machines yet to be solved.
    pub fn len_unsolved(&self) -> usize {
        self.yes.len()
    }

    /// Internal function: extend `v` with the IDs saved to `path`, without trying to sort or dedup.
    fn extend_from<P: AsRef<Path>>(v: &mut Vec<MachineID>, path: P) -> io::Result<()> {
        let file = read(path)?;
        let slice = IndexFile::new_slice_unaligned(file.as_slice()).unwrap();
        v.extend(slice.into_iter().copied().map(U32::get));
        Ok(())
    }

    /// Restore the invariants after messing around with the individual vectors.
    fn clean(&mut self) {
        self.no.sort();
        self.no.dedup();
        self.yes = self
            .yes
            .iter()
            .copied()
            .merge_join_by(self.no.iter().copied(), MachineID::cmp)
            .filter_map(|lr| match lr {
                EitherOrBoth::Left(id) => Some(id),
                _ => None,
            })
            .collect();
    }

    /// Save the "no" list as a decided index at the given path.
    fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let backup_path = path.as_ref().with_extension("backup");
        let _ = rename(&path, &backup_path);
        let mut w = BufWriter::new(File::create(&path)?);
        for i in self.no.iter() {
            w.write(&i.to_be_bytes())?;
        }
        let _ = remove_file(backup_path);
        Ok(())
    }
}
