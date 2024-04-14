use super::*;

impl FormulaTape {
    /// Gives the finite word (i.e. excluding 0∞) left of a repeater.
    ///
    ///  ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError, v2s};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0], TapeHead::default(), &[1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(v2s(&formula_tape.finite_word_left_of_repeater(1).unwrap()), "111111011");
    /// assert_eq!(v2s(&formula_tape.finite_word_left_of_repeater(0).unwrap()), "");
    /// ```
    pub fn finite_word_left_of_repeater(
        &self,
        repeater_index: usize,
    ) -> Result<Vec<u8>, FormulaTapeError> {
        let repeater_pos = self
            .repeaters_pos
            .get(repeater_index)
            .ok_or(FormulaTapeError::InvalidRepeaterIndex)?;
        let mut pos = (repeater_pos.beg - 1) as i32;
        let mut word: Vec<u8> = Vec::new();
        while pos >= 0 {
            match &self.tape.tape_content[pos as usize] {
                TapeContent::Symbol(symbol) => word.push(*symbol),
                TapeContent::Head(_) => break,
                TapeContent::InfiniteZero => break,
            }

            if self.pos_is_repeater_end(pos as usize) {
                break;
            }

            pos -= 1;
        }

        word.reverse();
        Ok(word)
    }

    /// Gives the finite word (i.e. excluding 0∞) right of a repeater.
    ///
    ///  ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError, v2s};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0], TapeHead::default(), &[1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 13, end: 15 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)111111011(11)0A>10∞");
    /// assert_eq!(v2s(&formula_tape.finite_word_right_of_repeater(1).unwrap()), "0");
    /// assert_eq!(v2s(&formula_tape.finite_word_right_of_repeater(0).unwrap()), "111111011");
    /// ```
    pub fn finite_word_right_of_repeater(
        &self,
        repeater_index: usize,
    ) -> Result<Vec<u8>, FormulaTapeError> {
        let repeater_pos = self
            .repeaters_pos
            .get(repeater_index)
            .ok_or(FormulaTapeError::InvalidRepeaterIndex)?;
        let mut pos = repeater_pos.end;
        let mut word: Vec<u8> = Vec::new();
        while pos < self.tape.len() && !self.pos_is_repeater_beg(pos as usize) {
            match &self.tape.tape_content[pos as usize] {
                TapeContent::Symbol(symbol) => word.push(*symbol),
                TapeContent::Head(_) => break,
                TapeContent::InfiniteZero => break,
            }
            pos += 1;
        }
        Ok(word)
    }

    /// Implements formula tape alignement.
    pub fn align(&mut self) -> Result<(), FormulaTapeError> {
        Ok(())
    }
}
