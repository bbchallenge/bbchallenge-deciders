use std::fmt;

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

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*
            State: E ;  _ . .[0]. . _  ;
        */
        let mut segment_string: String = String::new();

        let mut state_string: char = '*';
        if !self.is_outside_segment {
            state_string = (b'A' + self.state - 1) as char
        }

        write!(f, "State: {} ; ", state_string)?;
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
}
