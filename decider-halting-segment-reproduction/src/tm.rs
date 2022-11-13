use std::fmt;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;

use crate::utils::u8_to_bool;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum HeadMove {
    Right,
    Left,
}

impl fmt::Display for HeadMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeadMove::Right => write!(f, "R"),
            HeadMove::Left => write!(f, "L"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum HaltOrGoto {
    Halt,
    Goto(u8),
}

#[derive(Copy, Clone)]
pub struct Transition {
    pub write: bool,
    pub hmove: HeadMove,
    pub goto: HaltOrGoto,
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.goto {
            HaltOrGoto::Halt => write!(f, "---"),
            HaltOrGoto::Goto(state) => {
                write!(f, "{}{}{}", self.write, self.hmove, (b'A' + state) as char)
            }
        }
    }
}

#[derive(Clone)]
pub struct TM {
    pub n_states: u8,
    pub n_symbols: u8,
    pub transitions: Vec<Vec<Transition>>,
}

impl fmt::Display for TM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Cf. https://discuss.bbchallenge.org/t/standard-tm-text-format
        for i in 0..self.n_states {
            for j in 0..self.n_symbols {
                write!(f, "{}", self.transitions[i as usize][j as usize])?;
            }
            if i != self.n_states - 1 {
                write!(f, "_")?
            }
        }
        write!(f, "")
    }
}

impl TM {
    pub fn from_bbchallenge_id(machine_id: u32, path_to_bbchallenge_db: &str) -> io::Result<TM> {
        let mut file = File::open(path_to_bbchallenge_db)?;
        let mut buf = vec![0u8; 30];

        file.seek(SeekFrom::Start(((machine_id + 1) * 30) as u64))?;
        file.read_exact(&mut buf)?;

        let mut transitions: Vec<Vec<Transition>> = vec![];

        let mut write: u8 = 0;
        let mut hmove: HeadMove = HeadMove::Right;
        let mut goto: HaltOrGoto;
        let mut i_state;

        for (i, &byte) in buf.iter().enumerate() {
            i_state = i / 6;

            if i % 6 == 0 {
                transitions.push(vec![]);
            }

            if i % 3 == 0 {
                write = byte;
            } else if i % 3 == 1 {
                hmove = if byte == 0 {
                    HeadMove::Right
                } else {
                    HeadMove::Left
                };
            } else {
                if byte == 0 {
                    goto = HaltOrGoto::Halt;
                } else {
                    goto = HaltOrGoto::Goto(byte - 1);
                }
                transitions[i_state].push(Transition {
                    write: u8_to_bool(write),
                    hmove,
                    goto,
                })
            }
        }

        Ok(TM {
            n_states: 5,
            n_symbols: 2,
            transitions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";

    #[test]
    fn bbchallenge_format() {
        let ids_and_format = [
            (234, "1RB---_0RC---_0RD---_1LE---_0RE1LE"),
            (2847516, "1RB---_1LC0RD_1LE1RD_0RB0RD_0LC---"),
            (14156519, "1RB1LE_1LC0RA_1LA1LD_0RE---_1RA0LB"),
            (9881807, "1RB1RD_1LC---_0RD0LC_0RE1RA_1LB1LB"),
        ];

        for (id, format) in ids_and_format {
            assert_eq!(
                format,
                TM::from_bbchallenge_id(id, PATH_TO_BBCHALLENGE_DB)
                    .unwrap()
                    .to_string()
            )
        }
    }
}
