use std::cmp::{max, min};

const TAPE_SIZE: usize = 200_000;
const MACHINE: &str = "1RB1RE_1LC1LB_0LD1LA_0RE---_1LD0RA";

#[derive(Debug)]
enum TMError {
    OutOfTapeError,
    MachineHasHalted,
}

struct Configuration {
    machine: String,
    tape: [u8; TAPE_SIZE],
    state: u8,
    head_pos: i32,
    step_count: i32,
    min_head_pos: i32,
    max_head_pos: i32,
    last_move_was_left: bool,
}

impl Configuration {
    /* Turing machine configuration.
    Implementation specific to 5-state 2-symbol.*/

    fn new(machine: &str) -> Configuration {
        Configuration {
            machine: machine.to_string(),
            tape: [0; TAPE_SIZE],
            state: 0,
            head_pos: 0,
            step_count: 0,
            min_head_pos: 0,
            max_head_pos: 0,
            last_move_was_left: false,
        }
    }

    fn readTape(&self, i: i32) -> Result<u8, TMError> {
        let j: i32 = i + ((TAPE_SIZE / 2) as i32);
        if j < 0 || j >= (TAPE_SIZE as i32) {
            return Err(TMError::OutOfTapeError);
        }
        Ok(self.tape[j as usize])
    }

    fn setTape(&mut self, i: i32, value: u8) -> Result<(), TMError> {
        let j: i32 = i + ((TAPE_SIZE / 2) as i32);
        if j < 0 || j >= (TAPE_SIZE as i32) {
            return Err(TMError::OutOfTapeError);
        }
        self.tape[j as usize] = value;
        Ok(())
    }

    fn step(&mut self) -> Result<(), TMError> {
        let curr_read: u8 = self.readTape(self.head_pos)?;
        let curr_trans_i: usize = (6 * self.state + 3 * curr_read + self.state) as usize;
        let curr_transition: &str = &self.machine[curr_trans_i..curr_trans_i + 3];

        let write: char = curr_transition.chars().nth(0).unwrap();
        let head_move: char = curr_transition.chars().nth(1).unwrap();
        let state_goto: char = curr_transition.chars().nth(2).unwrap();

        if write == '-' {
            return Err(TMError::MachineHasHalted);
        }

        self.setTape(self.head_pos, write as u8 - '0' as u8);

        if head_move == 'R' {
            self.head_pos += 1;
            self.last_move_was_left = false;
        } else {
            self.head_pos -= 1;
            self.last_move_was_left = true;
        }

        self.min_head_pos = min(self.head_pos, self.min_head_pos);
        self.max_head_pos = max(self.head_pos, self.max_head_pos);
        self.step_count += 1;

        self.state = state_goto as u8 - 'A' as u8;

        if self.state >= 5 {
            return Err(TMError::MachineHasHalted);
        }

        Ok(())
    }
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in self.min_head_pos..self.max_head_pos + 1 {
            if i == self.head_pos && !self.last_move_was_left {
                write!(f, "{}>", (self.state + 'A' as u8) as char);
            }

            write!(f, "{}", self.readTape(i).unwrap());

            if i == self.head_pos && self.last_move_was_left {
                write!(f, "<{}", (self.state + 'A' as u8) as char);
            }
        }
        write!(f, "")
    }
}

fn main() {
    let mut conf: Configuration = Configuration::new(MACHINE);
    for _ in 0..100 {
        println!("{}", conf);
        conf.step().unwrap();
    }
}
