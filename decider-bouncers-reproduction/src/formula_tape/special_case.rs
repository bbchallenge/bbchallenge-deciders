use super::*;

impl FormulaTape {
    /// Detects if the formula tape is a *special* case of an other, given formula tape.
    ///
    /// f' is a special case of f if f' can be obtained from f by replacying repeaters words of the form (r) by r^n(r)r^m for some n,m>=0.
    ///
    /// ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let model_formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,0,1,1,0,0], TapeHead {state: 3, pointing_direction: Direction::RIGHT}, &[]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 8, end: 10 }] };
    /// assert_eq!(format!("{model_formula_tape}"), "0∞(111)1110(11)00D>0∞");
    /// assert_eq!(model_formula_tape.is_special_case_of(&model_formula_tape), Ok(true));
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,1,1,0,0], TapeHead {state: 3, pointing_direction: Direction::RIGHT}, &[]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 11, end: 13 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1111110(11)1100D>0∞");
    /// assert_eq!(formula_tape.is_special_case_of(&model_formula_tape), Ok(true));
    /// ````
    pub fn is_special_case_of(
        &self,
        model_formula_tape: &FormulaTape,
    ) -> Result<bool, FormulaTapeError> {
        // Working on aligned formula makes detection easier
        let mut aligned_self = self.clone();
        aligned_self.align()?;
        let mut aligned_model = model_formula_tape.clone();
        aligned_model.align()?;

        if aligned_self.repeaters_pos.len() != aligned_model.repeaters_pos.len() {
            return Ok(false);
        }

        // Check repeaters before head
        for repeater_index in 0..self.repeaters_pos.len() {
            let repeater_word = self.get_repeater_word(repeater_index)?;
            let repeater_pos = self.repeaters_pos[repeater_index];

            if repeater_pos.beg > self.tape.head_pos {
                break;
            }
            if repeater_pos.beg == self.tape.head_pos {
                return Err(FormulaTapeError::InvalidFormulaTapeError);
            }

            let self_right_word = aligned_self.finite_word_right_of_repeater(repeater_index)?;
            let model_right_word = aligned_model.finite_word_right_of_repeater(repeater_index)?;

            // Model must be smaller than instance
            if model_right_word.len() > self_right_word.len() {
                return Ok(false);
            }

            // Model and instance must differ only in repeating the repeater word
            if (self_right_word.len() - model_right_word.len()) % (repeater_word.len()) != 0 {
                return Ok(false);
            }

            let num_repetitions =
                (self_right_word.len() - model_right_word.len()) / repeater_word.len();

            // Check if the instance right word ends with the model right word
            if self_right_word[num_repetitions * repeater_word.len()..] != model_right_word {
                return Ok(false);
            }

            // Check if the instance right word starts with repetitions of the repeater word
            if self_right_word[..num_repetitions * repeater_word.len()]
                != repeater_word.repeat(num_repetitions)
            {
                return Ok(false);
            }
        }

        // Check repeaters after head
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

            let self_left_word = aligned_self.finite_word_right_of_repeater(repeater_index)?;
            let model_left_word = aligned_model.finite_word_right_of_repeater(repeater_index)?;

            // Model must be smaller than instance
            if model_left_word.len() > self_left_word.len() {
                return Ok(false);
            }

            // Model and instance must differ only in repeating the repeater word
            if (self_left_word.len() - model_left_word.len()) % (repeater_word.len()) != 0 {
                return Ok(false);
            }

            let num_repetitions =
                (self_left_word.len() - model_left_word.len()) / repeater_word.len();

            // Check if the instance left word starts with the model left word
            if self_left_word[..model_left_word.len()] != model_left_word {
                return Ok(false);
            }

            // Check if the instance left word ends with repetitions of the repeater word
            if self_left_word[model_left_word.len()
                ..model_left_word.len() + num_repetitions * repeater_word.len()]
                != repeater_word.repeat(num_repetitions)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
