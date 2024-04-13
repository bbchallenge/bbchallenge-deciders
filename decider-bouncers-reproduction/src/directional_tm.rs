use std::{collections::VecDeque, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum TMError {
    MachineHasHalted,
    OutOfTapeError,
    InvalidTapeError,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Direction {
    LEFT,
    RIGHT,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TMTransition {
    pub write: u8,
    pub direction: Direction,
    pub state_goto: u8,
}

impl TMTransition {
    /// Transforms a standard transition triplet to a TMTransition.
    ///
    /// For instance, "1RA" gives `TMTransition { write: 1, direction: Direction::RIGHT, state_goto: 0 }`.
    ///
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{TMTransition, Direction};
    /// assert_eq!(TMTransition::from_std_str_triple("1RA"), Some(TMTransition { write: 1, direction: Direction::RIGHT, state_goto: 0 }));
    /// assert_eq!(TMTransition::from_std_str_triple("0LD"), Some(TMTransition { write: 0, direction: Direction::LEFT, state_goto: 3 }));
    /// assert_eq!(TMTransition::from_std_str_triple("---"), None);
    /// ```
    pub fn from_std_str_triple(triple: &str) -> Option<TMTransition> {
        if triple.len() != 3 || triple.chars().nth(0).unwrap() == '-' {
            return Option::None;
        }

        let write = triple.chars().nth(0).unwrap();
        let direction = triple.chars().nth(1).unwrap();
        let state_goto = triple.chars().nth(2).unwrap();

        let direction_enum = match direction {
            'L' => Direction::LEFT,
            'R' => Direction::RIGHT,
            _ => return Option::None,
        };

        Option::Some(TMTransition {
            write: (write as u8) - b'0',
            direction: direction_enum,
            state_goto: (state_goto as u8) - b'A',
        })
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TMTransitionTable {
    pub machine_std_format: String,
}

impl TMTransitionTable {
    pub fn new(machine_std_format: &str) -> TMTransitionTable {
        TMTransitionTable {
            machine_std_format: machine_std_format.to_string(),
        }
    }

    /// Returns the transition corresponding to a given state and read symbol.
    ///
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{TMTransition, Direction, TMTransitionTable};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let transition_table = TMTransitionTable::new(machine_str);
    /// assert_eq!(transition_table.get_transition(0, 0), Some(TMTransition { write: 1, direction: Direction::RIGHT, state_goto: 1 }));
    /// assert_eq!(transition_table.get_transition(0, 1), Some(TMTransition { write: 1, direction: Direction::LEFT, state_goto: 4 }));
    /// assert_eq!(transition_table.get_transition(4, 0), None);
    /// assert_eq!(transition_table.get_transition(4, 1), Some(TMTransition { write: 0, direction: Direction::LEFT, state_goto: 0 }));
    /// ```
    pub fn get_transition(&self, state: u8, read: u8) -> Option<TMTransition> {
        let machine_split: Vec<&str> = self.machine_std_format.split('_').collect();
        let read_usize = read as usize;
        TMTransition::from_std_str_triple(
            &machine_split[state as usize][3 * read_usize..3 * read_usize + 3],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TapeHead {
    pub state: u8,
    pub pointing_direction: Direction,
}

impl Default for TapeHead {
    fn default() -> TapeHead {
        TapeHead {
            state: 0,
            pointing_direction: Direction::RIGHT,
        }
    }
}
impl fmt::Display for TapeHead {
    /// Returns the string representation of a shift rule.
    ///
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{TapeHead, Direction};
    /// let head = TapeHead { state: 0, pointing_direction: Direction::RIGHT };
    /// assert_eq!(format!("{}", head), "A>");
    /// let head = TapeHead { state: 4, pointing_direction: Direction::LEFT };
    /// assert_eq!(format!("{}", head), "<E");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.pointing_direction {
            Direction::RIGHT => write!(f, "{}>", (self.state + b'A') as char),
            Direction::LEFT => write!(f, "<{}", (self.state + b'A') as char),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TapeContent {
    InfiniteZero,
    Symbol(u8),
    Head(TapeHead),
}

impl TapeContent {
    pub fn is_head(&self) -> bool {
        matches!(self, TapeContent::Head(_))
    }
}

/// Directional Turing machine (potentially partial) tape, with additional information stored for convenience.
/// Note that in this setup the tape also contain the head, hence completely represents a (partial) tape.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Tape {
    machine_transition: TMTransitionTable,
    pub tape_content: VecDeque<TapeContent>,
    pub head_pos: usize,
    step_count: i32,
}

impl fmt::Display for Tape {
    /// Returns the string representation of a tape.
    ///
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{TMTransition, Direction, TMTransitionTable, Tape};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let tape = Tape::new_initial(machine_str);
    /// assert_eq!(format!("{tape}"), "0∞A>0∞");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.tape_content.len() {
            match &self.tape_content[i] {
                TapeContent::InfiniteZero => write!(f, "0∞")?,
                TapeContent::Symbol(x) => write!(f, "{}", x)?,
                TapeContent::Head(head) => {
                    if i != self.head_pos {
                        panic!("Stored head position {} is not consistent with actual head position {} in tape.", self.head_pos, i);
                    }
                    write!(f, "{}", head)?;
                }
            }
        }
        write!(f, "")
    }
}

impl Tape {
    /// Constructs a tape enclosed with 0∞ symbols.
    pub fn new(
        machine_std_format: &str,
        before_head: &[u8],
        head: TapeHead,
        after_head: &[u8],
    ) -> Tape {
        Tape {
            machine_transition: TMTransitionTable::new(machine_std_format),
            tape_content: VecDeque::from(
                [
                    &[TapeContent::InfiniteZero],
                    before_head
                        .iter()
                        .map(|&x| TapeContent::Symbol(x))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &[TapeContent::Head(head)],
                    after_head
                        .iter()
                        .map(|&x| TapeContent::Symbol(x))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &[TapeContent::InfiniteZero],
                ]
                .concat(),
            ),
            head_pos: before_head.len() + 1,
            step_count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tape_content.len()
    }

    pub fn finite_words_left_right_of_head(&self) -> Result<(Vec<u8>, Vec<u8>), TMError> {
        let head = self.get_current_head()?;

        let left_word = self
            .tape_content
            .iter()
            .skip_while(|&x| *x == TapeContent::InfiniteZero)
            .take_while(|&x| !x.is_head())
            .map(|x| match x {
                TapeContent::Symbol(y) => *y,
                _ => panic!("Head should only point to 0/1 symbols."),
            })
            .collect::<Vec<_>>();

        let right_word = self
            .tape_content
            .iter()
            .skip(self.head_pos + 1)
            .take_while(|&x| *x != TapeContent::InfiniteZero)
            .map(|x| match x {
                TapeContent::Symbol(y) => *y,
                _ => panic!("Head should only point to 0/1 symbols."),
            })
            .collect::<Vec<_>>();

        Ok((left_word, right_word))
    }

    pub fn first_index_non_zero_infinite(&self) -> Option<usize> {
        self.tape_content
            .iter()
            .position(|&x| x != TapeContent::InfiniteZero)
    }

    pub fn last_index_non_zero_infinite(&self) -> Option<usize> {
        self.tape_content
            .iter()
            .rposition(|&x| x != TapeContent::InfiniteZero)
    }

    /// Returns a sub-tape from given start to end (excluded).
    pub fn sub_tape(&self, start: usize, end: usize) -> Option<Tape> {
        if end <= start || end > self.tape_content.len() {
            return None;
        }

        Some(Tape {
            machine_transition: self.machine_transition.clone(),
            tape_content: self
                .tape_content
                .iter()
                .cloned()
                .skip(start)
                .take(end - start)
                .collect(),
            head_pos: self.head_pos - start,
            step_count: self.step_count,
        })
    }

    /// Constructs an initial tape in bbchallenge sense: 0∞A>0∞.
    pub fn new_initial(machine_std_format: &str) -> Tape {
        Tape::new(machine_std_format, &[], TapeHead::default(), &[])
    }

    /// Constructs a partial tape (not enclosed with 0∞ symbols)
    pub fn new_partial(
        machine_std_format: &str,
        before_head: &[u8],
        head: TapeHead,
        after_head: &[u8],
    ) -> Tape {
        Tape {
            machine_transition: TMTransitionTable::new(machine_std_format),
            tape_content: VecDeque::from(
                [
                    before_head
                        .iter()
                        .map(|&x| TapeContent::Symbol(x))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &[TapeContent::Head(head)],
                    after_head
                        .iter()
                        .map(|&x| TapeContent::Symbol(x))
                        .collect::<Vec<_>>()
                        .as_slice(),
                ]
                .concat(),
            ),
            head_pos: before_head.len(),
            step_count: 0,
        }
    }

    fn valid_tape_after_direction(
        &self,
        pos: usize,
        direction: Direction,
    ) -> Result<usize, TMError> {
        let new_pos = match direction {
            Direction::RIGHT => (pos as i32) + 1,
            Direction::LEFT => (pos as i32) - 1,
        };
        if new_pos < 0 || new_pos >= self.tape_content.len().try_into().unwrap() {
            return Err(TMError::OutOfTapeError);
        }

        Ok(new_pos as usize)
    }

    fn get_tape_content(&self, pos: usize) -> Result<TapeContent, TMError> {
        if pos >= self.tape_content.len() {
            return Err(TMError::OutOfTapeError);
        }

        match self.tape_content[pos] {
            TapeContent::InfiniteZero => {
                if pos == 0 || pos == self.tape_content.len() - 1 {
                    return Ok(TapeContent::InfiniteZero);
                }
                Err(TMError::InvalidTapeError)
            }
            TapeContent::Symbol(x) => Ok(TapeContent::Symbol(x)),
            TapeContent::Head(head) => {
                if pos == self.head_pos {
                    return Ok(TapeContent::Head(head));
                }
                Err(TMError::InvalidTapeError)
            }
        }
    }

    pub fn get_current_head(&self) -> Result<TapeHead, TMError> {
        match self.get_tape_content(self.head_pos)? {
            TapeContent::Head(head) => Ok(head),
            _ => Err(TMError::InvalidTapeError),
        }
    }

    pub fn get_current_read_pos(&self) -> Result<usize, TMError> {
        self.valid_tape_after_direction(self.head_pos, self.get_current_head()?.pointing_direction)
    }

    fn get_current_read_content(&self) -> Result<TapeContent, TMError> {
        self.get_tape_content(self.get_current_read_pos()?)
    }

    fn get_current_read_symbol(&self) -> Result<u8, TMError> {
        match self.get_current_read_content()? {
            TapeContent::Symbol(x) => Ok(x),
            TapeContent::InfiniteZero => Ok(0),
            TapeContent::Head(_) => Err(TMError::InvalidTapeError),
        }
    }

    fn get_current_transition(&self) -> Result<TMTransition, TMError> {
        let curr_transition = self.machine_transition.get_transition(
            self.get_current_head()?.state,
            self.get_current_read_symbol()?,
        );
        match curr_transition {
            Option::None => Err(TMError::MachineHasHalted),
            Option::Some(transition) => Ok(transition),
        }
    }

    /// Implements a directional Turing machine step, inplace.
    ///
    /// Testing the step function on partial tapes:
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{Tape,TapeContent,TapeHead, Direction, TMError};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let mut tape = Tape::new_partial(machine_str, &[1,0,0,1], TapeHead::default(),&[0,1]);
    /// assert_eq!(format!("{tape}"), "1001A>01");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "10011B>1");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "100111D>");
    /// assert_eq!(tape.step(), Err(TMError::OutOfTapeError));
    /// ```
    ///
    /// Testing the step function with expansion on the right 0∞ extremity:
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{Tape};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let mut tape = Tape::new_initial(machine_str);
    /// assert_eq!(format!("{tape}"), "0∞A>0∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞1B>0∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞1<C10∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞1C>10∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞11C>0∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞11<B10∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞11D>10∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞110D>0∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞110<A10∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞111B>10∞");
    /// ```
    ///
    /// Testing the step function with expansion on the left 0∞ extremity:
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{Tape};
    /// let machine_str = "1LB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let mut tape = Tape::new_initial(machine_str);
    /// assert_eq!(format!("{tape}"), "0∞A>0∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞<B10∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞<C110∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞<B1110∞");
    /// tape.step();
    /// assert_eq!(format!("{tape}"), "0∞<C11110∞");
    /// tape.step();
    /// ```
    pub fn step(&mut self) -> Result<(), TMError> {
        let curr_head = self.get_current_head()?;
        let mut curr_read_pos = self.get_current_read_pos()?;
        let curr_read_content = self.get_current_read_content()?;
        let curr_transition = self.get_current_transition()?;

        let new_head = TapeHead {
            state: curr_transition.state_goto,
            pointing_direction: curr_transition.direction,
        };

        // Extend tape if head pointing at 0^\infty extremity
        match curr_read_content {
            TapeContent::InfiniteZero => {
                if curr_read_pos != 0 {
                    self.tape_content.push_back(TapeContent::InfiniteZero);
                } else {
                    self.tape_content.push_front(TapeContent::InfiniteZero);
                    self.head_pos += 1;
                    curr_read_pos += 1;
                }
            }
            TapeContent::Symbol(_) => {}
            TapeContent::Head(_) => {
                return Err(TMError::InvalidTapeError);
            }
        }

        self.tape_content[self.head_pos] = TapeContent::Head(new_head);
        self.tape_content[curr_read_pos] = TapeContent::Symbol(curr_transition.write);

        if curr_head.pointing_direction == new_head.pointing_direction {
            self.tape_content.swap(self.head_pos, curr_read_pos);
            self.head_pos =
                self.valid_tape_after_direction(self.head_pos, curr_transition.direction)?;
        }

        self.step_count += 1;
        Ok(())
    }

    /// Implements n directional Turing machine steps, inplace.
    ///
    /// ```
    /// use decider_bouncers_reproduction::directional_tm::{Tape};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let mut tape = Tape::new_initial(machine_str);
    /// assert_eq!(format!("{tape}"), "0∞A>0∞");
    /// tape.steps(64);
    /// assert_eq!(format!("{tape}"), "0∞11111101100D>0∞");
    /// tape.steps(25);
    /// assert_eq!(format!("{tape}"), "0∞111111011<A01010110∞");
    /// tape.steps(2);
    /// assert_eq!(format!("{tape}"), "0∞1111110<A0101010110∞");
    /// tape.steps(13);
    /// assert_eq!(format!("{tape}"), "0∞1111111110110D>0110∞");
    /// tape.steps(4);
    /// assert_eq!(format!("{tape}"), "0∞111111111011110D>10∞");
    /// tape.steps(1);
    /// assert_eq!(format!("{tape}"), "0∞1111111110111100D>0∞");
    /// ```
    pub fn steps(&mut self, n: u32) -> Result<(), TMError> {
        for _ in 0..n {
            self.step()?;
        }
        Ok(())
    }
}
