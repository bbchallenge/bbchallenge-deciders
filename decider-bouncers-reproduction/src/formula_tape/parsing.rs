use super::*;
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
