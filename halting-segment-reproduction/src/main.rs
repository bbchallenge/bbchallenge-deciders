use std::fmt;

mod display_nodes;
mod hash_nodes;
mod neighbours;
mod tm;

use tm::{HaltOrGoto, HeadMove, TM};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum SegmentCell {
    Unallocated,
    Bit(u8),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum OutsideSegmentOrState {
    OutsideSegment,
    State(u8),
}

#[derive(Clone)]
struct SegmentCells(pub Vec<SegmentCell>);

#[derive(Clone, PartialEq, Eq, Hash)]
struct Node {
    state: OutsideSegmentOrState,
    segment: SegmentCells,
    pos_in_segment: usize,
}

struct Nodes(pub Vec<Node>);

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";

fn main() {
    let n: Node = Node {
        state: OutsideSegmentOrState::State(4),
        segment: SegmentCells(vec![
            SegmentCell::Unallocated,
            SegmentCell::Unallocated,
            SegmentCell::Bit(0),
            SegmentCell::Unallocated,
            SegmentCell::Unallocated,
        ]),
        pos_in_segment: 2,
    };

    println!("{}", n);

    let tm: TM = TM::from_bbchallenge_id(76708232, PATH_TO_BBCHALLENGE_DB).unwrap();

    println!("{}", tm);

    println!(
        "{}\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
        n.get_neighbours(&tm)[0],
        Nodes(n.get_neighbours(&tm)[0].get_neighbours(&tm)),
        Nodes(n.get_neighbours(&tm)[0].get_neighbours(&tm)[0].get_neighbours(&tm)),
        Nodes(
            n.get_neighbours(&tm)[0].get_neighbours(&tm)[0].get_neighbours(&tm)[0]
                .get_neighbours(&tm)
        ),
        Nodes(
            n.get_neighbours(&tm)[0].get_neighbours(&tm)[0].get_neighbours(&tm)[0]
                .get_neighbours(&tm)[0]
                .get_neighbours(&tm)
        ),
        Nodes(
            n.get_neighbours(&tm)[0].get_neighbours(&tm)[0].get_neighbours(&tm)[0]
                .get_neighbours(&tm)[1]
                .get_neighbours(&tm)
        )
    );
}
