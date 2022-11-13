use std::{
    collections::{HashSet, VecDeque},
    fmt,
    io::Read,
};

use rayon::prelude::*;
use std::convert::TryInto;
use std::fs::File;

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

#[derive(Debug, PartialEq, Eq)]
enum HaltingSegmentResult {
    MACHINE_DOES_NOT_HALT(usize),
    CANNOT_CONCLUDE(usize),
    NODE_LIMIT_EXCEED,
}

fn get_initial_nodes(tm: &TM, segment_size: u8, initial_pos_in_segment: usize) -> Vec<Node> {
    assert!(initial_pos_in_segment < segment_size as usize);
    let mut to_return: Vec<Node> = Vec::new();
    for i_state in 0..tm.n_states {
        for symbol in 0..tm.n_symbols {
            let transition = tm.transitions[i_state as usize][symbol as usize];
            if let HaltOrGoto::Halt = transition.goto {
                let mut initial_segment = vec![SegmentCell::Unallocated; segment_size as usize];
                initial_segment[initial_pos_in_segment] = SegmentCell::Bit(symbol);
                to_return.push(Node {
                    state: OutsideSegmentOrState::State(i_state),
                    segment: SegmentCells(initial_segment),
                    pos_in_segment: initial_pos_in_segment,
                })
            }
        }
    }

    to_return
}

fn halting_segment_decider(
    tm: &TM,
    segment_size: u8,
    initial_pos_in_segment: usize,
    node_limit: usize,
    print_run_info: bool,
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
            return HaltingSegmentResult::CANNOT_CONCLUDE(node_seen.len() + 1);
        }

        node_queue.append(&mut VecDeque::from(curr_node.get_neighbours(&tm)));
        node_seen.insert(curr_node.clone());

        if print_run_info {
            println!("{} ; Node: {}", curr_node, node_seen.len());
        }
    }

    if node_queue.is_empty() {
        HaltingSegmentResult::MACHINE_DOES_NOT_HALT(node_seen.len())
    } else {
        HaltingSegmentResult::NODE_LIMIT_EXCEED
    }
}

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";
const PATH_TO_UNDECIDED_INDEX: &str = "../bb5_undecided_index";

fn Iijil_strategy(machine_id: u32, node_limit: usize) -> bool {
    /* Implements @Iijil's strategy for running the backward halting segment decider:
        - The decider is run with all odd segment length until success or cumulative node limit is reached
        - Initial position in the segment is middle of it
    */
    let mut distance_to_segment_end: u8 = 1;
    let mut total_nodes_consumed = 0;

    let tm = TM::from_bbchallenge_id(machine_id, PATH_TO_BBCHALLENGE_DB).unwrap();

    while total_nodes_consumed < node_limit {
        let segment_size = 2 * distance_to_segment_end + 1;
        let initial_pos_in_segment = distance_to_segment_end as usize;
        let result =
            halting_segment_decider(&tm, segment_size, initial_pos_in_segment, node_limit, false);

        match result {
            HaltingSegmentResult::MACHINE_DOES_NOT_HALT(_) => {
                return true;
            }
            HaltingSegmentResult::CANNOT_CONCLUDE(nb_nodes) => {
                total_nodes_consumed += nb_nodes;
            }
            HaltingSegmentResult::NODE_LIMIT_EXCEED => {
                return false;
            }
        }
    }

    false
}

fn main() {
    const NODE_LIMIT: usize = 10000;

    let mut undecided_index_file = File::open(PATH_TO_UNDECIDED_INDEX).unwrap();
    let mut raw_data: Vec<u8> = Vec::new();

    undecided_index_file.read_to_end(&mut raw_data).unwrap();

    let undecided_ids: Vec<u32> = raw_data
        .chunks_exact(4)
        .map(|s| s.try_into().unwrap())
        .map(u32::from_be_bytes)
        .collect();

    let decided_ids: Vec<&u32> = undecided_ids
        .par_iter()
        .filter(|&id| Iijil_strategy(*id, NODE_LIMIT))
        .collect();

    println!(
        "{} machines decided by halting segment (using @Iijil's strategy)",
        decided_ids.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaotic_machine() {
        // http://turbotm.de/~heiner/BB/TM4-proof.txt
        // Chaotic Machine [Marxen & Buntrock, 1990]
        let chaotic_machine_id = 76708232;
        let tm: TM = TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB).unwrap();
        assert_eq!(
            halting_segment_decider(&tm, 5, 2, 1000, false),
            // 7 nodes expanded, cross checked with @Iijil
            HaltingSegmentResult::MACHINE_DOES_NOT_HALT(7)
        );
    }

    #[test]
    fn complex_counter() {
        // Complex Counter [Marxen & Buntrock, 1990]
        let chaotic_machine_id = 10936909;
        let tm: TM = TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB).unwrap();

        assert_eq!(
            halting_segment_decider(&tm, 7, 3, 1000, false),
            // 38 nodes expanded, cross checked with @Iijil
            HaltingSegmentResult::MACHINE_DOES_NOT_HALT(38)
        );
    }

    #[test]
    fn machine_108115() {
        // bbchallenge machine 108115
        let chaotic_machine_id = 108115;
        let tm: TM = TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB).unwrap();

        assert_eq!(
            halting_segment_decider(&tm, 3, 1, 1000, false),
            // 18 nodes expanded, cross checked with @Iijil
            HaltingSegmentResult::MACHINE_DOES_NOT_HALT(18)
        );
    }
}
