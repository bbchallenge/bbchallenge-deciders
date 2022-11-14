use crate::*;

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*
            State: E ;  _ . .[0]. . _  ;
        */
        let state_char: char = match self.state {
            OutsideSegmentOrState::OutsideSegment => '*',
            OutsideSegmentOrState::State(state) => (b'A' + state) as char,
        };

        write!(f, "State: {} ; ", state_char)?;

        let is_outside = matches!(self.state, OutsideSegmentOrState::OutsideSegment);

        if is_outside && self.pos_in_segment == 0 {
            write!(f, "[_]")?;
        } else {
            write!(f, " _")?;
        }

        for i in 0..self.segment.0.len() {
            if i == 0 && (i != self.pos_in_segment && !is_outside) {
                write!(f, " ")?;
            }

            let mut space_after = true;
            match self.segment.0[i] {
                SegmentCell::Unallocated => write!(f, ".")?,
                SegmentCell::Bit(bit) => {
                    if i != self.pos_in_segment || is_outside {
                        write!(f, "{}", bit as u8)?;
                    } else {
                        write!(f, "[{}]", bit as u8)?;
                        space_after = false;
                    }
                }
            }
            if space_after && (i + 1 != self.pos_in_segment || is_outside) {
                write!(f, " ")?;
            }
        }

        if is_outside && self.pos_in_segment + 1 == self.segment.0.len() {
            write!(f, "[_]")?;
        } else {
            write!(f, " _")?;
        }

        write!(f, "")
    }
}

impl fmt::Display for Nodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, node) in self.0.iter().enumerate() {
            writeln!(f, "{}: {}", i, node)?;
        }
        write!(f, "")
    }
}

mod tests {
    use super::*;

    #[test]
    fn machine_324_trace() {
        let chaotic_machine_id = 324;
        let tm: TM = TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB).unwrap();

        halting_segment_decider(&tm, 5, 2, NodeLimit::NodeLimit(1000), true);
    }

    #[test]
    fn machine_chaotic_trace() {
        // Chaotic Machine [Marxen & Buntrock, 1990]
        let chaotic_machine_id = 76708232;
        let tm: TM = TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB).unwrap();

        halting_segment_decider(&tm, 5, 2, NodeLimit::NodeLimit(1000), true);
    }
}
