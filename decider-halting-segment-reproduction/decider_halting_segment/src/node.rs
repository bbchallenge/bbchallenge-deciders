use std::fmt;

use crate::{tm::HeadMove, *};

pub(crate) enum NodeLimit {
    NoLimit,
    NodeLimit(usize),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum OutsideSegmentOrState {
    OutsideSegment,
    State(u8),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SegmentCell {
    Unallocated,
    Bit(bool),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct SegmentCells(pub Vec<SegmentCell>);

impl SegmentCells {
    fn are_there_no_ones(&self) -> bool {
        !self
            .0
            .iter()
            .any(|cell| matches!(cell, SegmentCell::Bit(true)))
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Node {
    pub(crate) state: OutsideSegmentOrState,
    pub(crate) segment: SegmentCells,
    pub(crate) pos_in_segment: usize,
}

impl Node {
    pub(crate) fn is_fatal(&self) -> bool {
        /* Fatal nodes are nodes whose segment contain no 1s and:
           - head is outside segment
           - Or, state is A
        Detecting these nodes is important because when the decider meets one, we know
        that we cannot conclude that the machine does not halt.
        */
        match self.state {
            OutsideSegmentOrState::OutsideSegment | OutsideSegmentOrState::State(0) => {
                self.segment.are_there_no_ones()
            }
            _ => false,
        }
    }

    pub fn get_neighbours(&self, tm: &TM) -> Vec<Node> {
        /* Returns the halting segment neighbours of the node:

        - If node is not outside segment (i.e. state is defined) then neighbours correspond to
        valid backward transitions.

        - If node is outside segment then neighbours correspond to states about to leave the segment (and where wrote tape symbol is consistent with future).
        */

        match self.state {
            OutsideSegmentOrState::OutsideSegment => self.get_neighbours_when_outside_segment(tm),
            OutsideSegmentOrState::State(state) => {
                self.get_neighbours_when_inside_segment(state, tm)
            }
        }
    }

    fn get_neighbours_when_outside_segment(&self, tm: &TM) -> Vec<Node> {
        /* When we are outside of the segment, neighbouring nodes are those
           that are inside and make us leave again.
        */
        let mut to_return: Vec<Node> = vec![];
        for i_state in 0..tm.n_states {
            for read_symbol in 0..tm.n_symbols {
                let transition = tm.transitions[i_state as usize][read_symbol as usize];

                // If transition halts, its not valid for backward
                match transition.goto {
                    HaltOrGoto::Halt => continue,
                    HaltOrGoto::Goto(_) => {}
                }

                // Check that transition makes us leave segment
                if !((self.pos_in_segment == 0 && transition.hmove == HeadMove::Left)
                    || (self.pos_in_segment + 1 == self.segment.0.len()
                        && transition.hmove == HeadMove::Right))
                {
                    continue;
                }

                // Check that backward transition write is consistent with current segment
                // (which is the future of that transition)
                let curr_segment_cell = &self.segment.0[self.pos_in_segment];
                if let SegmentCell::Bit(bit) = curr_segment_cell {
                    if *bit != transition.write {
                        continue;
                    }
                }

                // Then, add neighbouring node
                let mut new_segment = self.segment.clone();
                new_segment.0[self.pos_in_segment] = SegmentCell::Bit(u8_to_bool(read_symbol));

                let node_to_add = Node {
                    pos_in_segment: self.pos_in_segment,
                    segment: new_segment,
                    state: OutsideSegmentOrState::State(i_state),
                };

                // Avoid double add
                if to_return.contains(&node_to_add) {
                    continue;
                }
                to_return.push(node_to_add);
            }
        }

        to_return
    }

    fn get_neighbours_when_inside_segment(&self, state: u8, tm: &TM) -> Vec<Node> {
        /* When we are inside of the segment, neighbouring nodes are those that correspond
        to valid backward transitions.
        */
        let mut to_return: Vec<Node> = vec![];

        for i_state in 0..tm.n_states {
            for read_symbol in 0..tm.n_symbols {
                let transition = tm.transitions[i_state as usize][read_symbol as usize];

                // If transition halts, its not valid for backward
                match transition.goto {
                    HaltOrGoto::Halt => continue,
                    HaltOrGoto::Goto(goto_state) => {
                        if goto_state != state {
                            continue;
                        }
                    }
                }

                // Then, two cases:
                // Case 1: backward transition makes us leave segment
                if (self.pos_in_segment == 0 && transition.hmove == HeadMove::Right)
                    || (self.pos_in_segment + 1 == self.segment.0.len()
                        && transition.hmove == HeadMove::Left)
                {
                    // Note that we do not update pos_in_segment when getting outside of segment
                    to_return.push(Node {
                        pos_in_segment: self.pos_in_segment,
                        segment: self.segment.clone(),
                        state: OutsideSegmentOrState::OutsideSegment,
                    });
                    continue;
                }

                // Case 2: backward transition does not make us leave segment

                let new_position = if transition.hmove == HeadMove::Right {
                    self.pos_in_segment - 1
                } else {
                    self.pos_in_segment + 1
                };

                // Check that backward transition write is consistent with current segment
                // (which is the future of that transition)
                let new_segment_cell = &self.segment.0[new_position];
                if let SegmentCell::Bit(bit) = new_segment_cell {
                    if *bit != transition.write {
                        continue;
                    }
                }

                // We can now construct the neighbouring Node
                // First, we update the segment with read symbol
                let mut new_segment = self.segment.clone();
                new_segment.0[new_position] = SegmentCell::Bit(u8_to_bool(read_symbol));

                let node_to_add = Node {
                    pos_in_segment: new_position,
                    segment: new_segment,
                    state: OutsideSegmentOrState::State(i_state),
                };

                // Avoid double add
                if to_return.contains(&node_to_add) {
                    continue;
                }
                to_return.push(node_to_add);
            }
        }

        to_return
    }
}

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

// pub(crate) struct Nodes(pub Vec<Node>);
//
// impl fmt::Display for Nodes {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         for (i, node) in self.0.iter().enumerate() {
//             writeln!(f, "{}: {}", i, node)?;
//         }
//         write!(f, "")
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_324_trace() {
        let chaotic_machine_id = 324;
        let tm: TM = TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();

        halting_segment_decider(&tm, 5, 2, NodeLimit::NodeLimit(1000), true);
    }

    #[test]
    fn machine_chaotic_trace() {
        // Chaotic Machine [Marxen & Buntrock, 1990]
        let chaotic_machine_id = 76708232;
        let tm: TM = TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();

        halting_segment_decider(&tm, 5, 2, NodeLimit::NodeLimit(1000), true);
    }
}
