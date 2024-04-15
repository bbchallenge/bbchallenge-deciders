use crate::directional_tm;
use crate::directional_tm::{Direction, Tape, TapeContent, TapeHead};
use std::collections::HashSet;
use std::fmt;

mod alignment;
mod shift_rule_detection;
mod special_case;

/// Represents a bouncer shift rule (c.f. bouncer writeup).
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
/// use decider_bouncers_reproduction::formula_tape::v2s;
/// let v = vec![0,0,1,1,1,0];
/// assert_eq!(v2s(&v), "001110");
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

#[derive(Debug, PartialEq, Eq)]
pub enum FormulaTapeError {
    TMError(directional_tm::TMError),
    InvalidFormulaTapeError,
    NoShiftRule,
    ShiftRuleNotApplicable,
    InvalidRepeaterIndex,
}

impl From<directional_tm::TMError> for FormulaTapeError {
    fn from(tm_error: directional_tm::TMError) -> Self {
        FormulaTapeError::TMError(tm_error)
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FormulaTape {
    pub tape: Tape,
    pub repeaters_pos: Vec<RepeaterPos>, // sorted by beg *and* end (if flattened the array is a sorted array of positions)
}
use std::str::FromStr;
impl FromStr for FormulaTape {
    type Err = FormulaTapeError;
    /// Converts strings such as `0∞(111)111111011(11)0A>10∞` to FormulaTape.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let formula_tape = FormulaTape { tape: Tape::new("", &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0], TapeHead::default(), &[1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// //assert_eq!(FormulaTape::from_str("0∞(111)111111011(11)0A>10∞"), Ok(formula_tape));
    /// let formula_tape = FormulaTape::from_str("<E000011110(11110111101111011110)000(1111011110)000(11110)000(11110)01111111").unwrap();
    /// assert_eq!(format!("{formula_tape}"), "<E000011110(11110111101111011110)000(1111011110)000(11110)000(11110)01111111");
    /// let formula_tape = FormulaTape::from_str("0∞1(11)1(11)01D>10(11)11(11)111110∞").unwrap();
    /// assert_eq!(format!("{formula_tape}"), "0∞1(11)1(11)01D>10(11)11(11)111110∞");
    /// let formula_tape = FormulaTape::from_str("0∞1(11)1(11)01D>10(11)11(11)11111").unwrap();
    /// assert_eq!(format!("{formula_tape}"), "0∞1(11)1(11)01D>10(11)11(11)11111");
    /// let formula_tape = FormulaTape::from_str("1(11)1(11)01D>10(11)11(11)111110∞").unwrap();
    /// assert_eq!(format!("{formula_tape}"), "1(11)1(11)01D>10(11)11(11)111110∞");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tape = Tape::from_str(&s.replace(['(', ')'], ""))?;
        let starts_with_infinite: i32 = s.starts_with("0∞") as i32;

        let s = s.replace("0∞", "");
        let mut repeaters_pos: Vec<RepeaterPos> = Vec::new();
        let mut pos = 0;
        let mut in_repeater = false;
        let mut num_non_zero_one_symbols = 0;
        let mut head_seen = 0;

        for (i, c) in s.chars().enumerate() {
            if c == '>' || c == '<' {
                head_seen = 1;
            }

            if c == '(' {
                if in_repeater {
                    return Err(FormulaTapeError::InvalidFormulaTapeError);
                }
                in_repeater = true;
                num_non_zero_one_symbols += 1;
                pos = i;
                if head_seen == 1 {
                    pos -= 2;
                }
            } else if c == ')' {
                if !in_repeater {
                    return Err(FormulaTapeError::InvalidFormulaTapeError);
                }
                repeaters_pos.push(RepeaterPos {
                    beg: pos + 1 - num_non_zero_one_symbols
                        + starts_with_infinite as usize
                        + head_seen,
                    end: i - head_seen - num_non_zero_one_symbols + starts_with_infinite as usize,
                });
                num_non_zero_one_symbols += 1;
                in_repeater = false;
            }
        }

        Ok(FormulaTape {
            tape,
            repeaters_pos,
        })
    }
}

impl FormulaTape {
    /// Returns a repeater's word.
    ///
    ///  ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError, v2s};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0], TapeHead::default(), &[1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(v2s(&formula_tape.get_repeater_word(0).unwrap()), "111");
    /// assert_eq!(v2s(&formula_tape.get_repeater_word(1).unwrap()), "11");
    /// ```
    pub fn get_repeater_word(&self, repeater_index: usize) -> Result<Vec<u8>, FormulaTapeError> {
        let repeater_pos = self
            .repeaters_pos
            .get(repeater_index)
            .ok_or(FormulaTapeError::InvalidRepeaterIndex)?;
        let mut word: Vec<u8> = Vec::new();

        for content in self
            .tape
            .tape_content
            .iter()
            .skip(repeater_pos.beg)
            .take(repeater_pos.len())
        {
            match &content {
                TapeContent::Symbol(symbol) => word.push(*symbol),
                _ => return Err(FormulaTapeError::InvalidFormulaTapeError),
            }
        }

        if word.is_empty() {
            return Err(FormulaTapeError::InvalidFormulaTapeError);
        }

        Ok(word)
    }

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

    pub fn head_is_pointing_at_repeater(&self) -> Result<bool, FormulaTapeError> {
        let head = self.tape.get_current_head()?;

        Ok((self.pos_is_repeater_beg(self.tape.head_pos + 1)
            && head.pointing_direction == Direction::RIGHT)
            || (self.pos_is_repeater_end(self.tape.head_pos)
                && head.pointing_direction == Direction::LEFT))
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
        if !self.head_is_pointing_at_repeater()? {
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
        if !self.head_is_pointing_at_repeater()? {
            return Err(FormulaTapeError::ShiftRuleNotApplicable);
        }

        let head = self.tape.get_current_head()?;

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
                beg: lhs_repeater_pos.beg + shift_rule.tail.len() + 1,
                end: lhs_repeater_pos.beg + shift_rule.tail.len() + 1 + lhs_repeater_pos.len(),
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
                self.tape.head_pos += lhs_repeater_pos.len();
            }
            Direction::LEFT => {
                self.tape.tape_content.make_contiguous()
                    [lhs_repeater_pos.beg..new_repeater_pos.end]
                    .rotate_left(lhs_repeater_pos.len());
                self.tape.head_pos -= lhs_repeater_pos.len();
            }
        }

        Ok(())
    }

    /// Implements a formula tape step: it corresponds to a standard TM step when the head in not pointing at a repeater and corresponds to running a shift rule (if any exists) otherwise.
    ///
    /// ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError, ShiftRule};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let mut formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,0,1,1,0,0], TapeHead {state: 3, pointing_direction: Direction::RIGHT}, &[]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 8, end: 10 }] };
    /// let initial_formula_tape = formula_tape.clone();
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1110(11)00D>0∞");
    /// formula_tape.steps(25);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1110(11)<A01010110∞");
    /// formula_tape.step();
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1110<A(01)01010110∞");
    /// formula_tape.step();
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1111B>(01)01010110∞");
    /// let res = formula_tape.step();
    /// assert_eq!(res, Err(FormulaTapeError::NoShiftRule));
    /// formula_tape.align();
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1111B>010101(01)10∞");
    /// formula_tape.steps(12);
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1111110110D>(01)10∞");
    /// formula_tape.step();
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0D>10∞");
    /// formula_tape.align();
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1111110(11)110D>10∞");
    /// formula_tape.step();
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1111110(11)1100D>0∞");
    /// assert!(formula_tape.is_special_case_of(&initial_formula_tape).unwrap());
    /// ```
    pub fn step(&mut self) -> Result<(), FormulaTapeError> {
        // Usual step: perform a TM step if head not pointing at a repeater
        if !self.head_is_pointing_at_repeater()? {
            return Ok(self.tape.step()?);
        }

        // Shift rule step: try to detect and apply a shift rule
        let shift_rule = self.detect_shift_rule()?;
        println!("{} {} steps", shift_rule, shift_rule.num_steps);
        self.apply_shift_rule(&shift_rule)?;
        Ok(())
    }

    pub fn steps(&mut self, num_steps: usize) -> Result<(), FormulaTapeError> {
        for _ in 0..num_steps {
            self.step()?;
        }
        Ok(())
    }

    pub fn set_machine_str(&mut self, machine_std_format: &str) {
        self.tape.set_machine_str(machine_std_format);
    }

    /// Returns true if the formula tape once reaches a special-case of itself (c.f. bouncers writeup).
    ///
    /// ```
    /// use std::str::FromStr;
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let mut formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,0,1,1,0,0], TapeHead {state: 3, pointing_direction: Direction::RIGHT}, &[]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 8, end: 10 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1110(11)00D>0∞");
    /// assert_eq!(formula_tape.prove_non_halt(200_000), Ok(true));
    /// let machine_str = "1RB0RD_1LC1LE_1RA1LB_---0RC_1LB0LE";
    /// let mut formula_tape = FormulaTape::from_str("0∞<E000011110(11110111101111011110)000(1111011110)000(11110)000(11110)011111110∞").unwrap();
    /// formula_tape.set_machine_str(machine_str);
    /// assert_eq!(format!("{formula_tape}"), "0∞<E000011110(11110111101111011110)000(1111011110)000(11110)000(11110)011111110∞");
    /// assert_eq!(formula_tape.prove_non_halt(200_000), Ok(true));
    /// ````
    pub fn prove_non_halt(&mut self, step_limit: usize) -> Result<bool, FormulaTapeError> {
        let initial_formula_tape = self.clone();
        self.align()?;
        println!("0 {}", self);
        for k in 0..step_limit {
            self.step()?;
            self.align()?;

            if self.head_is_pointing_at_repeater()? {
                println!("{} {}", k + 1, self);
            }
            if self.is_special_case_of(&initial_formula_tape)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl fmt::Display for FormulaTape {
    /// Returns the string representation of a formula tape.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.tape.tape_content.len() {
            match &self.tape.tape_content[i] {
                TapeContent::InfiniteZero => write!(f, "0∞")?,
                TapeContent::Symbol(x) => {
                    if self.pos_is_repeater_beg(i) && self.pos_is_repeater_end(i + 1) {
                        write!(f, "({})", x)?;
                    } else if self.pos_is_repeater_beg(i) {
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
