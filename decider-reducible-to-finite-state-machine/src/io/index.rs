//! An undecided index -- or output file -- as in https://bbchallenge.org/method.
//! Warning: this implementation is meant for use in the decider; it it'll assume an output path
//! and try to merge any checkpoint files left behind.

use super::{timestamp, MachineID};
use itertools::{EitherOrBoth, Itertools};
use std::fs::{read, read_dir, remove_file, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use zerocopy::{BigEndian, LayoutVerified, U32};

type IndexFile<B> = LayoutVerified<B, [U32<BigEndian>]>;

/// A data structure for tracking which DB entries are previously unsolved machines.
pub struct Index {
    yes: Vec<MachineID>,
    no: Vec<MachineID>,
}

impl Index {
    /// Initialize an index where everything is considered unsolved.
    pub fn new(len: usize) -> Index {
        let mut yes = vec![0; len];
        let no = vec![0; 0];
        for i in 0..len {
            yes[i] = i as MachineID;
        }
        Index { yes, no }
    }

    /// Initialize an index from an "undecided" file.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Index> {
        let mut index = Self::new(0);
        Self::extend_from(&mut index.yes, path)?;
        index.clean();
        Ok(index)
    }

    /// Populate the negative entries from our own output directory.
    pub fn resume(&mut self) -> io::Result<()> {
        let dir_path = "output";
        let in_paths: Vec<PathBuf> = read_dir(dir_path)?
            .flat_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        for path in in_paths.iter() {
            Self::extend_from(&mut self.no, path)?;
        }
        self.clean();
        if in_paths.len() <= 1 {
            Ok(())
        } else {
            self.save(Path::new(dir_path).join(format!("merge.{}", timestamp())))?;
            in_paths.into_iter().try_for_each(|path| remove_file(path))
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = MachineID> + '_ {
        self.yes
            .iter()
            .copied()
            .merge_join_by(self.no.iter().copied(), MachineID::cmp)
            .filter_map(|lr| match lr {
                EitherOrBoth::Left(id) => Some(id),
                _ => None,
            })
    }

    /// Internal function: extend `v` with the IDs saved to `path`, without trying to sort or dedup.
    fn extend_from<P: AsRef<Path>>(v: &mut Vec<MachineID>, path: P) -> io::Result<()> {
        let file = read(path)?;
        let slice = IndexFile::new_slice_unaligned(file.as_slice()).unwrap();
        v.extend(slice.into_iter().copied().map(U32::get));
        Ok(())
    }

    fn clean(&mut self) {
        for v in [&mut self.no, &mut self.yes] {
            v.sort();
            v.dedup();
        }
        self.yes = self.iter().collect();
    }

    /// Save the "no" list as a decided index at the given path.
    fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut w = BufWriter::new(File::create(&path)?);
        for i in self.no.iter() {
            w.write(&i.to_be_bytes())?;
        }
        println!(
            "Saved {} - {} machines decided.",
            path.as_ref().display(),
            self.no.len()
        );
        Ok(())
    }
}
