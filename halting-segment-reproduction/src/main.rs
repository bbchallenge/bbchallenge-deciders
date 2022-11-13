use std::fmt;

mod tm;

use tm::{HaltOrGoto, HeadMove, Transition, TM};

#[derive(Copy, Clone)]
enum SegmentCell {
    Unallocated,
    Bit(u8),
}

#[derive(Copy, Clone)]
enum OutsideSegmentOrState {
    OutsideSegment,
    State(u8),
}

#[derive(Clone)]
struct Node {
    state: OutsideSegmentOrState,
    segment: Vec<SegmentCell>,
    pos_in_segment: usize,
}

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";

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
        write!(f, "_")?;
        for (i, segment_pos) in self.segment.iter().enumerate() {
            match segment_pos {
                SegmentCell::Unallocated => segment_string += " . ",
                SegmentCell::Bit(bit) => {
                    if i == self.pos_in_segment {
                        write!(f, "[{}]", bit)?;
                    } else {
                        write!(f, "{}", bit)?;
                    }
                }
            }
        }
        write!(f, "_")
    }
}

impl Node {
    fn get_neighbours(&self, tm: &TM) -> Vec<Node> {
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
        vec![]
    }

    fn get_neighbours_when_inside_segment(&self, state: u8, tm: &TM) -> Vec<Node> {
        let mut to_return: Vec<Node> = vec![];

        for i_state in 0..tm.n_states {
            for read_symbol in 0..tm.n_symbols {
                let transition = tm.transitions[i_state as usize][read_symbol as usize];
                match transition.goto {
                    HaltOrGoto::Halt => continue,
                    HaltOrGoto::Goto(goto_state) => {
                        if goto_state != state {
                            continue;
                        }
                    }
                }

                let curr_segment_cell = &self.segment[self.pos_in_segment];

                if let SegmentCell::Bit(bit) = curr_segment_cell {
                    // Backward transition write is not consistent with future
                    if *bit != transition.write {
                        continue;
                    }
                }

                let mut new_segment = self.segment.clone();
                new_segment[self.pos_in_segment] = SegmentCell::Bit(read_symbol);

                // Case where backward transition makes us leave segment
                if (self.pos_in_segment == 0 && transition.hmove == HeadMove::Right)
                    || (self.pos_in_segment + 1 == self.segment.len()
                        && transition.hmove == HeadMove::Left)
                {
                    // Note that we do not update pos_in_segment when getting outside of segment
                    to_return.push(Node {
                        pos_in_segment: self.pos_in_segment,
                        segment: new_segment,
                        state: OutsideSegmentOrState::OutsideSegment,
                    });
                } else {
                    let new_position = if transition.hmove == HeadMove::Right {
                        self.pos_in_segment + 1
                    } else {
                        self.pos_in_segment - 1
                    };

                    to_return.push(Node {
                        pos_in_segment: new_position,
                        segment: new_segment,
                        state: OutsideSegmentOrState::State(i_state),
                    });
                }
            }
        }

        return to_return;
    }
}

fn main() {
    let n: Node = Node {
        state: OutsideSegmentOrState::OutsideSegment,
        segment: vec![
            SegmentCell::Unallocated,
            SegmentCell::Unallocated,
            SegmentCell::Bit(0),
            SegmentCell::Unallocated,
            SegmentCell::Unallocated,
        ],
        pos_in_segment: 2,
    };

    println!("{}", n);

    let tm: TM = TM::from_bbchallenge_id(234, PATH_TO_BBCHALLENGE_DB).unwrap();

    println!("{}", tm);
}