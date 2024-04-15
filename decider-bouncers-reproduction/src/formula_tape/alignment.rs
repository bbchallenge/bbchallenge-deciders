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
    ///
    /// ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError, v2s};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let mut formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,0,1,1,1,0], TapeHead {state: 3, pointing_direction: Direction::RIGHT}, &[1,0,1,0,0,1,1]), repeaters_pos: vec![RepeaterPos { beg: 4, end: 6 },RepeaterPos { beg: 10, end: 13 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞101(11)0D>10(100)110∞");
    /// formula_tape.align().unwrap();
    /// assert_eq!(format!("{formula_tape}"), "0∞10(11)10D>101(001)10∞");
    /// let mut formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,0,1], TapeHead {state: 3, pointing_direction: Direction::RIGHT}, &[1,0,1,1,1,1,1,1,1,1,1,1,1]), repeaters_pos: vec![RepeaterPos { beg: 2, end: 4 },RepeaterPos { beg: 5, end: 7 },RepeaterPos { beg: 12, end: 14 }, RepeaterPos { beg: 16, end: 18 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞1(11)1(11)01D>10(11)11(11)111110∞");
    /// formula_tape.align().unwrap();
    /// assert_eq!(format!("{formula_tape}"), "0∞(11)(11)1101D>101111111(11)(11)0∞");
    /// ```
    pub fn align(&mut self) -> Result<(), FormulaTapeError> {
        // Align before head
        for repeater_index in 0..self.repeaters_pos.len() {
            let repeater_word = self.get_repeater_word(repeater_index)?;
            let repeater_pos = self.repeaters_pos[repeater_index];

            if repeater_pos.beg > self.tape.head_pos {
                break;
            }
            if repeater_pos.beg == self.tape.head_pos {
                return Err(FormulaTapeError::InvalidFormulaTapeError);
            }

            let left_word = self.finite_word_left_of_repeater(repeater_index)?;
            for i in 0..left_word.len() {
                let suffix = &left_word[i..];

                // Test `ar = r'a` with a = suffix and r = repeater_word
                let composite = [suffix, &repeater_word].concat();
                if &composite[repeater_word.len()..] == suffix {
                    let new_repeater_pos = RepeaterPos {
                        beg: repeater_pos.beg - suffix.len(),
                        end: repeater_pos.end - suffix.len(),
                    };
                    self.repeaters_pos[repeater_index] = new_repeater_pos;
                    break;
                }
            }
        }

        // Align after head
        for repeater_index in (0..self.repeaters_pos.len()).rev() {
            let repeater_word = self.get_repeater_word(repeater_index)?;
            let repeater_pos = self.repeaters_pos[repeater_index];

            if repeater_pos.beg < self.tape.head_pos {
                break;
            }
            if repeater_pos.beg == self.tape.head_pos {
                return Err(FormulaTapeError::InvalidFormulaTapeError);
            }

            if repeater_pos.beg <= self.tape.head_pos {
                return Err(FormulaTapeError::InvalidFormulaTapeError);
            }

            let right_word = self.finite_word_right_of_repeater(repeater_index)?;
            for i in (1..right_word.len() + 1).rev() {
                let prefix = &right_word[..i];

                // Test `ra = ar'` with a = prefix and r = repeater_word
                let composite = [&repeater_word, prefix].concat();
                if &composite[..prefix.len()] == prefix {
                    let new_repeater_pos = RepeaterPos {
                        beg: repeater_pos.beg + prefix.len(),
                        end: repeater_pos.end + prefix.len(),
                    };
                    self.repeaters_pos[repeater_index] = new_repeater_pos;
                    break;
                }
            }
        }

        Ok(())
    }
}
