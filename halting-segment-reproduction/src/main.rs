use std::fmt;

mod tm;

use tm::{HaltOrGoto, HeadMove, Transition, TM};

enum SegmentPos {
    Unallocated,
    Bit(u8),
}

struct Node {
    state: u8,
    is_outside_segment: bool,
    segment: Vec<SegmentPos>,
    pos_in_segment: usize,
}

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*
            State: E ;  _ . .[0]. . _  ;
        */
        let mut segment_string: String = String::new();

        let mut state_char: char = if self.is_outside_segment {
            '*'
        } else {
            ('A' as u8 + self.state - 1) as char
        };

        write!(f, "State: {} ; ", state_char)?;
        write!(f, "_")?;
        for (i, segment_pos) in self.segment.iter().enumerate() {
            match segment_pos {
                SegmentPos::Unallocated => segment_string += " . ",
                SegmentPos::Bit(bit) => {
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

fn main() {
    let n: Node = Node {
        state: 1,
        is_outside_segment: true,
        segment: vec![
            SegmentPos::Unallocated,
            SegmentPos::Unallocated,
            SegmentPos::Bit(0),
            SegmentPos::Unallocated,
            SegmentPos::Unallocated,
        ],
        pos_in_segment: 2,
    };

    println!("{}", n);

    let tm: TM = TM::from_bbchallenge_id(234, PATH_TO_BBCHALLENGE_DB).unwrap();

    println!("{}", tm);
}
