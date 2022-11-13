use crate::*;

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*
            State: E ;  _ . .[0]. . _  ;
        */
        let mut segment_string: String = String::new();

        let state_char: char = match self.state {
            OutsideSegmentOrState::OutsideSegment => '*',
            OutsideSegmentOrState::State(state) => ('A' as u8 + state - 1) as char,
        };

        write!(f, "State: {} ; ", state_char)?;

        if let OutsideSegmentOrState::State(_) = self.state {
            write!(f, "_")?;
        } else if self.pos_in_segment == 0 {
            write!(f, "[_]")?;
        }

        for (i, segment_pos) in self.segment.iter().enumerate() {
            match segment_pos {
                SegmentCell::Unallocated => segment_string += " . ",
                SegmentCell::Bit(bit) => {
                    if let OutsideSegmentOrState::State(_) = self.state {
                        if i == self.pos_in_segment {
                            write!(f, "[{}]", bit)?;
                        } else {
                            write!(f, "{}", bit)?;
                        }
                    } else {
                        write!(f, "{}", bit)?;
                    }
                }
            }
        }

        if let OutsideSegmentOrState::State(_) = self.state {
            write!(f, "_")
        } else if self.pos_in_segment + 1 == self.segment.len() {
            write!(f, "[_]")
        } else {
            write!(f, "_")
        }
    }
}

impl fmt::Display for Nodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, node) in self.0.iter().enumerate() {
            write!(f, "{}: {}\n", i, node)?;
        }
        write!(f, "")
    }
}
