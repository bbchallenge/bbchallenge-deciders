
use std::fmt;
enum HeadMove {
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

enum HaltOrGoto {
    Halt,
    Goto(u8),
}

struct Transition {
    write: u8,
    hmove: HeadMove,
    goto: HaltOrGoto,
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

pub struct TM {
    n_states: u8,
    n_symbol: u8,
    transitions: Vec<Vec<Transition>>,
}

impl fmt::Display for TM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Cf. https://discuss.bbchallenge.org/t/standard-tm-text-format
        for i in 0..self.n_states {
            for j in 0..self.n_symbol {
                write!(f, "{}", self.transitions[i as usize][j as usize])?;
            }
            if i != self.n_states - 1 {
                write!(f, "_")?
            }
        }
        write!(f, "")
    }
}
