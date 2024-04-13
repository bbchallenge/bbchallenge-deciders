use crate::directional_tm;
use crate::directional_tm::{Direction, Tape, TapeContent, TapeHead};
use std::fmt;

mod shift_rule_detection;

/// Represents a bouncer shift rule (c.f. bouncer writeup).
pub struct ShiftRule {
    pub head: TapeHead,
    pub tail: Vec<u8>,
    pub lhs_repeater: Vec<u8>,
    pub rhs_repeater: Vec<u8>,
    pub num_steps: usize,
}

/// Returns the string representation of a vector of u8.
///
/// ```
/// use decider_bouncers_reproduction::formula_tape::vec_u8_to_string;
/// let v = vec![0,0,1,1,1,0];
/// assert_eq!(vec_u8_to_string(&v), "001110");
/// ```
pub fn v2s(v: &[u8]) -> String {
    v.iter().map(|i| i.to_string()).collect::<String>()
}

impl fmt::Display for ShiftRule {
    /// Returns the string representation of a shift rule. We store additional num_steps information to be able to display it.
    ///
    /// ```
    /// use decider_bouncers_reproduction::formula_tape::{ShiftRule};
    /// use decider_bouncers_reproduction::directional_tm::{TapeHead, Direction};
    /// let shift_rule = ShiftRule { head: TapeHead::default(), tail: vec![1,1,0], lhs_repeater: vec![1,1], rhs_repeater: vec![0,0], num_steps: 2};
    /// assert_eq!(format!("{shift_rule}"), "110A>(11) → (00)110A>");
    /// let shift_rule = ShiftRule { head: TapeHead { state: 3, pointing_direction: Direction::LEFT }, tail: vec![1,1,0], lhs_repeater: vec![1,1], rhs_repeater: vec![0,0], num_steps: 2 };
    /// assert_eq!(format!("{shift_rule}"), "(11)<D110 → <D110(00)");
    /// ````
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.head.pointing_direction {
            Direction::RIGHT => write!(
                f,
                "{}{}({}) → ({}){}{}",
                v2s(&self.tail),
                self.head,
                v2s(&self.lhs_repeater),
                v2s(&self.rhs_repeater),
                v2s(&self.tail),
                self.head
            ),
            Direction::LEFT => write!(
                f,
                "({}){}{} → {}{}({})",
                v2s(&self.lhs_repeater),
                self.head,
                v2s(&self.tail),
                self.head,
                v2s(&self.tail),
                v2s(&self.rhs_repeater),
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Begin and end indexes of a repeater in a formula tape.
pub struct RepeaterPos {
    pub beg: usize,
    /// end is exclusive, and repeater cannot be empty beg < end
    pub end: usize,
}

impl RepeaterPos {
    fn len(&self) -> usize {
        self.end - self.beg
    }
}

/// Formula tape (wall-repeater formula tape) as defined in the bouncers writeup.
///
/// ```
/// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos};
/// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
/// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
/// let formula_tape = FormulaTape { tape: Tape::new_partial(machine_str, &[1,1,0,1,1,0,1], TapeHead::default(), &[0,0]), repeaters_pos: vec![RepeaterPos { beg: 0, end: 4 }] };
/// assert_eq!(format!("{formula_tape}"), "(1101)101A>00");
/// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0], TapeHead::default(), &[1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
/// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
/// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1], TapeHead::default(), &[1,1,0,1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 14, end: 16 }] };
/// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011A>(11)010∞");
/// ```
pub struct FormulaTape {
    pub tape: Tape,
    pub repeaters_pos: Vec<RepeaterPos>, // sorted by beg *and* end (if unwraped the array is a sorted array of positions)
}

#[derive(Debug, PartialEq, Eq)]
pub enum FormulaTapeError {
    TMError(directional_tm::TMError),
    InvalidFormulaTapeError,
    NoShiftRule,
    ShiftRuleNotApplicable,
}

impl FormulaTapeError {
    fn result_from_tm_error<T>(
        res: Result<T, directional_tm::TMError>,
    ) -> Result<T, FormulaTapeError> {
        match res {
            Ok(x) => Ok(x),
            Err(e) => Err(FormulaTapeError::TMError(e)),
        }
    }
}

impl FormulaTape {
    fn pos_is_repeater_beg(&self, pos: usize) -> bool {
        self.repeaters_pos
            .binary_search_by_key(&pos, |repeater_pos| repeater_pos.beg)
            .is_ok()
    }

    fn pos_is_repeater_end(&self, pos: usize) -> bool {
        self.repeaters_pos
            .binary_search_by_key(&pos, |repeater_pos| repeater_pos.end)
            .is_ok()
    }

    /// Returns the position of the repeater (if any) whose beginning is the closest to the right of the given pos. If the given pos is the beginning of a repeater, this repeater is returned.
    ///
    ///  ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0], TapeHead::default(), &[1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
    /// let repeater_right = formula_tape.repeater_right(0);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_right, Some(RepeaterPos { beg: 1, end: 4 }));
    /// let repeater_right = formula_tape.repeater_right(1);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_right, Some(RepeaterPos { beg: 1, end: 4 }));
    /// let repeater_right = formula_tape.repeater_right(5);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_right, Some(RepeaterPos { beg: 13, end: 15 }));
    /// let repeater_right = formula_tape.repeater_right(13);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_right, Some(RepeaterPos { beg: 13, end: 15 }));
    /// let repeater_right = formula_tape.repeater_right(14);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_right, None);
    /// ```
    pub fn repeater_right(&self, pos: usize) -> Option<RepeaterPos> {
        let repeater_index = self
            .repeaters_pos
            .binary_search_by_key(&pos, |repeater_pos| repeater_pos.beg);

        match repeater_index {
            Ok(repeater_index) => Some(self.repeaters_pos[repeater_index]),
            Err(repeater_index) => {
                if repeater_index == self.repeaters_pos.len() {
                    return None;
                }
                Some(self.repeaters_pos[repeater_index])
            }
        }
    }

    /// Returns the position of the repeater (if any) whose beginning is the closest to the left of the given pos. If the given pos is the beginning of a repeater, this repeater is returned.
    ///
    /// ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0], TapeHead::default(), &[1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
    /// let repeater_left = formula_tape.repeater_left(0);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_left, None);
    /// let repeater_left = formula_tape.repeater_left(1);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_left, Some(RepeaterPos { beg: 1, end: 4 }));
    /// let repeater_left = formula_tape.repeater_left(5);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_left, Some(RepeaterPos { beg: 1, end: 4 }));
    /// let repeater_left = formula_tape.repeater_left(13);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_left, Some(RepeaterPos { beg: 13, end: 15 }));
    /// let repeater_left = formula_tape.repeater_left(14);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_left, Some(RepeaterPos { beg: 13, end: 15 }));
    /// let repeater_left = formula_tape.repeater_left(17);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(repeater_left, Some(RepeaterPos { beg: 13, end: 15 }));
    /// ```
    pub fn repeater_left(&self, pos: usize) -> Option<RepeaterPos> {
        let repeater_index = self
            .repeaters_pos
            .binary_search_by_key(&pos, |repeater_pos| repeater_pos.beg);

        match repeater_index {
            Ok(repeater_index) => Some(self.repeaters_pos[repeater_index]),
            Err(repeater_index) => {
                if repeater_index == 0 {
                    return None;
                }
                Some(self.repeaters_pos[repeater_index - 1])
            }
        }
    }

    fn head_is_pointing_at_repeater(&self) -> bool {
        if self.tape.head_pos == 0 {
            return self.pos_is_repeater_beg(self.tape.head_pos + 1);
        }
        self.pos_is_repeater_beg(self.tape.head_pos + 1)
            || self.pos_is_repeater_end(self.tape.head_pos)
    }

    /// Returns the sub-tape corresponding to the shift rule the head is potentially pointing at.
    ///
    /// ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1], TapeHead::default(), &[1,1,0,1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 14, end: 16 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011A>(11)010∞");
    /// let shift_rule_tape = formula_tape.shift_rule_tape().unwrap();
    /// assert_eq!(format!("{shift_rule_tape}"), "111111011A>11");
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[], TapeHead::default(), &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0,1]), repeaters_pos: vec![RepeaterPos { beg: 2, end: 5 },RepeaterPos { beg: 14, end: 16 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞A>(111)111111011(11)010∞");
    /// let shift_rule_tape = formula_tape.shift_rule_tape().unwrap();
    /// assert_eq!(format!("{shift_rule_tape}"), "A>111");
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str,  &[1,1,1,1,1,1,1,1,1,0,1,1,1,1], TapeHead {state: 0, pointing_direction: Direction::LEFT}, &[0,1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)<A010∞");
    /// let shift_rule_tape = formula_tape.shift_rule_tape().unwrap();
    /// assert_eq!(format!("{shift_rule_tape}"), "11<A01");
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str,  &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0,1], TapeHead {state: 0, pointing_direction: Direction::LEFT}, &[]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 15, end: 17 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)11111101111(01)<A0∞");
    /// let shift_rule_tape = formula_tape.shift_rule_tape().unwrap();
    /// assert_eq!(format!("{shift_rule_tape}"), "01<A");
    /// ````
    pub fn shift_rule_tape(&self) -> Result<Tape, FormulaTapeError> {
        if !self.head_is_pointing_at_repeater() {
            return Err(FormulaTapeError::NoShiftRule);
        }

        let head = match &self.tape.tape_content[self.tape.head_pos] {
            TapeContent::Head(head) => head,
            _ => {
                return Err(FormulaTapeError::TMError(
                    directional_tm::TMError::InvalidTapeError,
                ))
            }
        };

        let shift_rule_beg = match head.pointing_direction {
            Direction::RIGHT => match self.repeater_left(self.tape.head_pos) {
                Some(repeater_pos) => repeater_pos.end,
                None => self.tape.first_index_non_zero_infinite().unwrap(), // unwrap is safe because tape non empty and contains at least the head
            },
            Direction::LEFT => self.repeater_left(self.tape.head_pos).unwrap().beg, // unwrap is safe because head is pointing at a repeater
        };

        let shift_rule_end = match head.pointing_direction {
            Direction::RIGHT => self.repeater_right(self.tape.head_pos).unwrap().end, // unwrap is safe because head is pointing at a repeater ,
            Direction::LEFT => match self.repeater_right(self.tape.head_pos) {
                Some(repeater_pos) => repeater_pos.beg,
                None => self.tape.last_index_non_zero_infinite().unwrap() + 1, // unwrap is safe because tape non empty and contains at least the head
            },
        };

        Ok(self.tape.sub_tape(shift_rule_beg, shift_rule_end).unwrap()) // shift_rule_beg and shift_rule_end should be valid
    }

    fn apply_shift_rule(&mut self, shift_rule: &ShiftRule) -> Result<(), FormulaTapeError> {
        if !self.head_is_pointing_at_repeater() {
            return Err(FormulaTapeError::ShiftRuleNotApplicable);
        }

        let head = FormulaTapeError::result_from_tm_error(self.tape.get_current_head())?;

        let lhs_repeater_pos = match head.pointing_direction {
            Direction::RIGHT => self.repeater_right(self.tape.head_pos).unwrap(), // unwrap is safe because head is pointing at a repeater
            Direction::LEFT => self.repeater_left(self.tape.head_pos).unwrap(), // unwrap is safe because head is pointing at a repeater
        };

        let lhs_repeater_index = self
            .repeaters_pos
            .binary_search_by_key(&lhs_repeater_pos.beg, |repeater_pos| repeater_pos.beg)
            .unwrap(); // unwrap is safe because repeater_pos is in repeaters_pos

        let new_repeater_pos = match head.pointing_direction {
            Direction::RIGHT => RepeaterPos {
                beg: lhs_repeater_pos.beg - shift_rule.tail.len() - 1,
                end: lhs_repeater_pos.beg - shift_rule.tail.len() - 1 + lhs_repeater_pos.len(),
            },
            Direction::LEFT => RepeaterPos {
                beg: lhs_repeater_pos.beg + shift_rule.lhs_repeater.len() + 1,
                end: lhs_repeater_pos.beg
                    + shift_rule.lhs_repeater.len()
                    + 1
                    + lhs_repeater_pos.len(),
            },
        };

        self.repeaters_pos[lhs_repeater_index] = new_repeater_pos;

        self.tape.tape_content.make_contiguous()[lhs_repeater_pos.beg..lhs_repeater_pos.end]
            .iter_mut()
            .zip(&shift_rule.rhs_repeater)
            .for_each(|(slot, x)| *slot = TapeContent::Symbol(*x));

        match head.pointing_direction {
            Direction::RIGHT => {
                self.tape.tape_content.make_contiguous()
                    [new_repeater_pos.beg..lhs_repeater_pos.end]
                    .rotate_right(lhs_repeater_pos.len());
            }
            Direction::LEFT => {
                self.tape.tape_content.make_contiguous()
                    [lhs_repeater_pos.beg..new_repeater_pos.end]
                    .rotate_left(lhs_repeater_pos.len());
            }
        }

        Ok(())
    }

    /// Implements a formula tape step: it corresponds to a standard TM step when the head in not pointing at a repeater and corresponds to running a shift rule (if any exists) otherwise.
    fn step(&mut self) -> Result<(), FormulaTapeError> {
        // Usual step: perform a TM step if head not pointing at a repeater
        if !self.head_is_pointing_at_repeater() {
            return FormulaTapeError::result_from_tm_error(self.tape.step());
        }

        // Shift rule step: try to detect and apply a shift rule
        let shift_rule = self.detect_shift_rule()?;
        self.apply_shift_rule(&shift_rule)?;
        Ok(())
    }
}

impl fmt::Display for FormulaTape {
    /// Returns the string representation of a formula tape.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.tape.tape_content.len() {
            match &self.tape.tape_content[i] {
                TapeContent::InfiniteZero => write!(f, "0∞")?,
                TapeContent::Symbol(x) => {
                    if self.pos_is_repeater_beg(i) {
                        write!(f, "({}", x)?;
                    } else if self.pos_is_repeater_end(i + 1) {
                        write!(f, "{})", x)?;
                    } else {
                        write!(f, "{}", x)?;
                    }
                }
                TapeContent::Head(head) => {
                    if i != self.tape.head_pos {
                        panic!("Stored head position {} is not consistent with actual head position {} in tape.", self.tape.head_pos, i);
                    }

                    write!(f, "{}", head)?;
                }
            }
        }
        write!(f, "")
    }
}
