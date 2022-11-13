use std::{
    collections::{HashSet, VecDeque},
    fmt,
};

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

#[derive(Clone, PartialEq, Eq)]
struct SegmentCells(pub Vec<SegmentCell>);

impl SegmentCells {
    fn is_all_zero(&self) -> bool {
        for cell in self.0.iter() {
            match cell {
                SegmentCell::Unallocated => return false,
                SegmentCell::Bit(b) => {
                    if *b != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Node {
    state: OutsideSegmentOrState,
    segment: SegmentCells,
    pos_in_segment: usize,
}

impl Node {
    fn is_fatal(&self) -> bool {
        /* Fatal nodes are nodes whose segment is all-0 and:
           - head is outside segment
           - Or, state is A
        Detecting these nodes is important because when the decider meets one, we know
        that we cannot conclude that the machine does not halt.
        */
        match self.state {
            OutsideSegmentOrState::OutsideSegment => self.segment.is_all_zero(),
            OutsideSegmentOrState::State(state) => state == 0 && self.segment.is_all_zero(),
        }
    }
}

struct Nodes(pub Vec<Node>);

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";

enum HaltingSegmentResult {
    MACHINE_DOES_NOT_HALT,
    CANNOT_CONCLUDE,
    NODE_LIMIT_EXCEED,
}

fn get_initial_nodes(tm: &TM, segment_size: u8, initial_pos_in_segment: usize) -> Vec<Node> {
    vec![]
}

fn halting_segment_decider(
    tm: TM,
    segment_size: u8,
    initial_pos_in_segment: usize,
    node_limit: usize,
) -> HaltingSegmentResult {
    let initial_nodes = get_initial_nodes(&tm, segment_size, initial_pos_in_segment);

    let mut node_queue: VecDeque<Node> = VecDeque::from(initial_nodes);
    let mut node_seen: HashSet<Node> = HashSet::new();

    while !node_queue.is_empty() && node_seen.len() <= node_limit {
        let curr_node = node_queue.pop_front().unwrap();

        if node_seen.contains(&curr_node) {
            continue;
        }

        if curr_node.is_fatal() {
            return HaltingSegmentResult::CANNOT_CONCLUDE;
        }

        node_queue.append(&mut VecDeque::from(curr_node.get_neighbours(&tm)));
        node_seen.insert(curr_node.clone());
    }

    if node_queue.is_empty() {
        HaltingSegmentResult::MACHINE_DOES_NOT_HALT
    } else {
        HaltingSegmentResult::NODE_LIMIT_EXCEED
    }
}

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
