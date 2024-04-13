use super::*;
use std::collections::HashSet;

impl FormulaTape {
    /// Detects a shift rule if any exists.
    ///
    /// ```
    /// use decider_bouncers_reproduction::formula_tape::{FormulaTape, RepeaterPos, FormulaTapeError, ShiftRule};
    /// use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
    /// let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,0,1,1], TapeHead {state: 0, pointing_direction: Direction::LEFT}, &[0,1,0,1,0,1,1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 8, end: 10 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1110(11)<A01010110∞");
    /// let shift_tape = formula_tape.shift_rule_tape().unwrap();
    /// assert_eq!(format!("{shift_tape}"), "11<A0101011");
    /// let shift_rule = formula_tape.detect_shift_rule().unwrap();
    /// assert_eq!(format!("{shift_rule}"), "(11)<A → <A(01)");
    /// assert_eq!(shift_rule.num_steps, 2);
    /// let formula_tape = FormulaTape { tape: Tape::new(machine_str, &[1,1,1,1,1,1,1,1,1,0,1,1,0], TapeHead {state: 3, pointing_direction: Direction::RIGHT}, &[0,1,1]), repeaters_pos: vec![RepeaterPos { beg: 1, end: 4 },RepeaterPos { beg: 15, end: 17 }] };
    /// assert_eq!(format!("{formula_tape}"), "0∞(111)1111110110D>(01)10∞");
    /// let shift_tape = formula_tape.shift_rule_tape().unwrap();
    /// assert_eq!(format!("{shift_tape}"), "1111110110D>01");
    /// let shift_rule = formula_tape.detect_shift_rule().unwrap();
    /// assert_eq!(format!("{shift_rule}"), "0D>(01) → (11)0D>");
    /// assert_eq!(shift_rule.num_steps, 4);
    /// ```
    ///
    /// TODO: this function could be factorised / improved.
    pub fn detect_shift_rule(&self) -> Result<ShiftRule, FormulaTapeError> {
        let mut shift_rule_tape = self.shift_rule_tape()?;
        let mut tapes_seen: HashSet<Tape> = HashSet::new();

        let initial_tape = shift_rule_tape.clone();
        let initial_head = FormulaTapeError::result_from_tm_error(initial_tape.get_current_head())?;
        tapes_seen.insert(initial_tape.clone());

        let (left_word_head, right_word_head) =
            FormulaTapeError::result_from_tm_error(initial_tape.finite_words_left_right_of_head())?;
        let lhs_repeater = match initial_head.pointing_direction {
            Direction::RIGHT => right_word_head,
            Direction::LEFT => left_word_head,
        };
        if lhs_repeater.is_empty() {
            return Err(FormulaTapeError::InvalidFormulaTapeError);
        }

        let lhs_repeater_size = lhs_repeater.len();

        let mut min_read_pos =
            FormulaTapeError::result_from_tm_error(shift_rule_tape.get_current_read_pos())?;
        let mut max_read_pos = min_read_pos;

        let mut num_steps = 0;

        loop {
            let res = shift_rule_tape.step();

            match res {
                Ok(()) => {
                    // Cycle detection
                    if tapes_seen.contains(&shift_rule_tape) {
                        return Err(FormulaTapeError::NoShiftRule);
                    }
                    tapes_seen.insert(shift_rule_tape.clone());
                    min_read_pos = min_read_pos.min(
                        shift_rule_tape
                            .get_current_read_pos()
                            .unwrap_or(min_read_pos),
                    );
                    max_read_pos = shift_rule_tape
                        .get_current_read_pos()
                        .unwrap_or(max_read_pos);
                    num_steps += 1;
                }
                Err(directional_tm::TMError::OutOfTapeError) => {
                    let final_head =
                        FormulaTapeError::result_from_tm_error(shift_rule_tape.get_current_head())?;

                    if initial_head.state == final_head.state {
                        let (final_left_word_head, final_right_word_head) =
                            FormulaTapeError::result_from_tm_error(
                                shift_rule_tape.finite_words_left_right_of_head(),
                            )?;
                        match initial_head.pointing_direction {
                            Direction::RIGHT => {
                                // Empty tail
                                if min_read_pos >= initial_tape.head_pos {
                                    return Ok(ShiftRule {
                                        head: initial_head,
                                        tail: vec![],
                                        lhs_repeater,
                                        rhs_repeater: final_left_word_head[..lhs_repeater_size]
                                            .to_vec(),
                                        num_steps,
                                    });
                                }

                                // Consider only part of the tape that has been visited
                                let interesting_initial_tape = initial_tape
                                    .sub_tape(min_read_pos, initial_tape.len())
                                    .unwrap();
                                let interesting_final_tape = shift_rule_tape
                                    .sub_tape(min_read_pos, shift_rule_tape.len())
                                    .unwrap();

                                let (tail, _) = FormulaTapeError::result_from_tm_error(
                                    interesting_initial_tape.finite_words_left_right_of_head(),
                                )?;

                                let (repeater_and_tail, _) =
                                    FormulaTapeError::result_from_tm_error(
                                        interesting_final_tape.finite_words_left_right_of_head(),
                                    )?;

                                if tail == repeater_and_tail[lhs_repeater.len()..] {
                                    let rhs_repeater =
                                        repeater_and_tail[..lhs_repeater.len()].to_vec();
                                    return Ok(ShiftRule {
                                        head: initial_head,
                                        tail: tail.to_vec(),
                                        lhs_repeater,
                                        rhs_repeater,
                                        num_steps,
                                    });
                                } else {
                                    return Err(FormulaTapeError::NoShiftRule);
                                }
                            }
                            Direction::LEFT => {
                                // Empty tail
                                if max_read_pos <= initial_tape.head_pos {
                                    return Ok(ShiftRule {
                                        head: initial_head,
                                        tail: vec![],
                                        lhs_repeater,
                                        rhs_repeater: final_right_word_head[..lhs_repeater_size]
                                            .to_vec(),
                                        num_steps,
                                    });
                                }

                                // Consider only part of the tape that has been visited
                                let interesting_initial_tape =
                                    initial_tape.sub_tape(0, max_read_pos + 1).unwrap();
                                let interesting_final_tape =
                                    shift_rule_tape.sub_tape(0, max_read_pos + 1).unwrap();

                                let (_, tail) = FormulaTapeError::result_from_tm_error(
                                    interesting_initial_tape.finite_words_left_right_of_head(),
                                )?;

                                let (_, tail_and_repeater) =
                                    FormulaTapeError::result_from_tm_error(
                                        interesting_final_tape.finite_words_left_right_of_head(),
                                    )?;

                                if tail
                                    == tail_and_repeater
                                        [0..(tail_and_repeater.len() - lhs_repeater_size)]
                                {
                                    let rhs_repeater = tail_and_repeater
                                        [0..(tail_and_repeater.len() - lhs_repeater_size)]
                                        .to_vec();
                                    return Ok(ShiftRule {
                                        head: initial_head,
                                        tail: tail.to_vec(),
                                        lhs_repeater,
                                        rhs_repeater,
                                        num_steps,
                                    });
                                } else {
                                    return Err(FormulaTapeError::NoShiftRule);
                                }
                            }
                        }
                    } else {
                        return Err(FormulaTapeError::NoShiftRule);
                    }
                }
                Err(e) => {
                    return FormulaTapeError::result_from_tm_error(Err(e));
                }
            }
        }
    }
}
