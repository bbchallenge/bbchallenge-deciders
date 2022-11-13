use std::fmt;

mod display_nodes;
mod neighbours;
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

struct Nodes(pub Vec<Node>);

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";

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

    println!("{}", Nodes(n.get_neighbours(&tm)));
}
