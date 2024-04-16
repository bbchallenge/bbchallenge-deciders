use super::*;
impl FromStr for Tape {
    type Err = TMError;
    /// Converts strings such as `0∞111111111011110A>10∞` to Tape.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use decider_bouncers_reproduction::directional_tm::{Tape, TapeContent, TapeHead, Direction};
    /// let tape = Tape::from_str("0∞111111111011110A>10∞").unwrap();
    /// assert_eq!(format!("{tape}"), "0∞111111111011110A>10∞");
    /// let tape = Tape::from_str("0∞01001010111<E10110∞").unwrap();
    /// assert_eq!(format!("{tape}"), "0∞01001010111<E10110∞");
    /// let tape = Tape::from_str("<E000011110111101111011110111100001111011110000111100001111001111111").unwrap();
    /// assert_eq!(format!("{tape}"), "<E000011110111101111011110111100001111011110000111100001111001111111");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tape_content: VecDeque<TapeContent> = VecDeque::new();
        let mut has_initial_infinite_zero = 0;
        let mut has_final_infinite_zero = false;
        let head: TapeHead;
        let head_pos: usize;

        for (i, _) in s.match_indices("0∞") {
            if i == 0 {
                tape_content.push_back(TapeContent::InfiniteZero);
                has_initial_infinite_zero = 1;
            } else if i == s.len() - 4 {
                // -4 because "0∞" has length 4 in UTF
                has_final_infinite_zero = true;
            } else {
                return Err(TMError::InvalidTapeError);
            }
        }
        let s = &s.replace("0∞", "");

        let head_indices_right = s.match_indices('>').collect::<Vec<_>>();
        let head_indices_left = s.match_indices('<').collect::<Vec<_>>();

        if head_indices_right.len() + head_indices_left.len() != 1 {
            return Err(TMError::InvalidTapeError);
        }
        if !head_indices_right.is_empty() {
            let head_index = head_indices_right[0].0;
            if head_index == 0 {
                return Err(TMError::InvalidTapeError);
            }
            head = TapeHead::from_str(&s[head_index - 1..head_index + 1])?;
            head_pos = head_index - 1;
        } else {
            let head_index = head_indices_left[0].0;
            if head_index == s.len() - 1 {
                return Err(TMError::InvalidTapeError);
            }
            head = TapeHead::from_str(&s[head_index..head_index + 2])?;
            head_pos = head_index;
        }

        for (i, c) in s.chars().enumerate() {
            if i == head_pos {
                tape_content.push_back(TapeContent::Head(head));
                continue;
            }

            if i == head_pos + 1 {
                continue;
            }

            tape_content.push_back(match c {
                '0' => TapeContent::Symbol(0),
                '1' => TapeContent::Symbol(1),
                _ => return Err(TMError::InvalidTapeError),
            });
        }

        if has_final_infinite_zero {
            tape_content.push_back(TapeContent::InfiniteZero);
        }

        Ok(Tape {
            machine_transition: TMTransitionTable::new(""),
            tape_content,
            head_pos: head_pos + has_initial_infinite_zero,
            step_count: 0,
        })
    }
}
