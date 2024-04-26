use super::*;
use std::cmp::max;
use std::collections::{HashSet, VecDeque};

impl FormulaTape {
    /// Detects a shift rule if any exists.
    ///
    /// ```
    /// use std::str::FromStr;
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
    /// let machine_str = "1RB0RD_1LC1LE_1RA1LB_---0RC_1LB0LE";
    /// let formula_tape_str =
    ///     "0∞11111100111001A>(11110111101111011110)000(1111011110)000(11110)000(11110)011111110∞";
    /// let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    /// formula_tape.set_machine_str(machine_str);
    /// let shift_rule = formula_tape.detect_shift_rule().unwrap();
    /// assert_eq!(format!("{shift_rule}"), "A>(11110111101111011110) → (11001110011100111001)A>");
    /// let mut formula_tape = FormulaTape::from_str("0∞(11001100)<A0110∞").unwrap();
    /// formula_tape.set_machine_str("1RB1LD_1RC1RE_1LA0LC_0RA0LA_0RD---");
    /// let shift_rule = formula_tape.detect_shift_rule().unwrap();
    /// assert_eq!(format!("{shift_rule}"), "(11001100)<A01 → <A01(10001000)");
    /// ```
    ///
    /// TODO: this function could be factorised / improved.
    pub fn detect_shift_rule(&self) -> Result<ShiftRule, FormulaTapeError> {
        let mut shift_rule_tape = self.shift_rule_tape()?;
        // println!("DETECT SHIFT RULE");
        // println!("{}", shift_rule_tape);
        // Doing HashSet<Tape> was bugging because tape's step count increases
        let mut tapes_seen: HashSet<String> = HashSet::new();

        let initial_tape = shift_rule_tape.clone();
        let initial_head = initial_tape.get_current_head()?;
        tapes_seen.insert(initial_tape.to_string());

        let (left_word_head, right_word_head) = initial_tape.finite_words_left_right_of_head()?;
        let lhs_repeater = match initial_head.pointing_direction {
            Direction::RIGHT => right_word_head,
            Direction::LEFT => left_word_head,
        };
        if lhs_repeater.is_empty() {
            return Err(FormulaTapeError::InvalidFormulaTapeError);
        }

        let lhs_repeater_size = lhs_repeater.len();

        let mut min_read_pos = shift_rule_tape.get_current_read_pos()?;
        let mut max_read_pos = min_read_pos;

        let mut num_steps = 0;

        loop {
            let res = shift_rule_tape.step();
            //println!("{}", shift_rule_tape);
            match res {
                Ok(()) => {
                    // Cycle detection
                    if tapes_seen.contains(&shift_rule_tape.to_string()) {
                        // Bouncer "1RB1LD_1RC1RE_1LA0LC_0RA0LA_0RD---" encounters a looper shift rule on:
                        // 0∞110011001100A>(11000100)01000100010∞
                        // //println!("here");
                        // for tape in tapes_seen.iter() {
                        //     println!("{}", tape);
                        // }
                        return Err(FormulaTapeError::NoShiftRule);
                    }

                    tapes_seen.insert(shift_rule_tape.to_string());
                    min_read_pos = min_read_pos.min(
                        shift_rule_tape
                            .get_current_read_pos()
                            .unwrap_or(min_read_pos),
                    );
                    max_read_pos = max(
                        max_read_pos,
                        shift_rule_tape
                            .get_current_read_pos()
                            .unwrap_or(max_read_pos),
                    );
                    num_steps += 1;
                }
                Err(directional_tm::TMError::OutOfTapeError) => {
                    let final_head = shift_rule_tape.get_current_head()?;
                    //println!("OUT OF TAPE {} {}", initial_head, final_head);
                    if initial_head == final_head {
                        let (final_left_word_head, final_right_word_head) =
                            shift_rule_tape.finite_words_left_right_of_head()?;
                        match initial_head.pointing_direction {
                            Direction::RIGHT => {
                                // Empty tail
                                if min_read_pos >= initial_tape.head_pos {
                                    return Ok(ShiftRule {
                                        head: initial_head,
                                        tail: vec![],
                                        lhs_repeater,
                                        rhs_repeater: final_left_word_head
                                            [final_left_word_head.len() - lhs_repeater_size..]
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

                                let (tail, _) =
                                    interesting_initial_tape.finite_words_left_right_of_head()?;

                                let (repeater_and_tail, _) =
                                    interesting_final_tape.finite_words_left_right_of_head()?;

                                //println!("{}", self);
                                //println!("{}", interesting_final_tape);
                                //println!("{} {}", initial_head, final_head);
                                // //println!(
                                //     "t:{} rt:{} r:{}",
                                //     v2s(&tail),
                                //     v2s(&repeater_and_tail),
                                //     v2s(&lhs_repeater)
                                // );
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
                                //println!("{} {}", max_read_pos, initial_tape.head_pos);
                                if max_read_pos <= initial_tape.head_pos {
                                    //println!("EMPTY TAIL");
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

                                let (_, tail) =
                                    interesting_initial_tape.finite_words_left_right_of_head()?;

                                let (_, tail_and_repeater) =
                                    interesting_final_tape.finite_words_left_right_of_head()?;

                                // println!("{}", self);
                                // println!("{}", interesting_final_tape);
                                // println!("{} {}", initial_head, final_head);
                                // println!(
                                //     "t:{} rt:{} r:{}",
                                //     v2s(&tail),
                                //     v2s(&tail_and_repeater),
                                //     v2s(&lhs_repeater)
                                // );
                                if tail == tail_and_repeater[..tail.len()] {
                                    //println!("FOUND SHIFT RULE");
                                    let rhs_repeater = tail_and_repeater[tail.len()..].to_vec();
                                    //println!("{} {}", v2s(&tail), v2s(&rhs_repeater));
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
                    return Err(e.into());
                }
            }
        }
    }
}
