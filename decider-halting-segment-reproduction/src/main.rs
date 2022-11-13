use std::{collections::HashSet, fmt, fs::File, io::Read};

use std::io::prelude::*;

use rand::{distributions::Alphanumeric, Rng};

use indicatif::{ParallelProgressIterator, ProgressStyle};

use rayon::prelude::*;
use std::convert::TryInto;

mod display_nodes;
mod neighbours;
mod tm;
mod utils;

use tm::{HaltOrGoto, HeadMove, TM};
use utils::u8_to_bool;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum SegmentCell {
    Unallocated,
    Bit(bool),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum OutsideSegmentOrState {
    OutsideSegment,
    State(u8),
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct SegmentCells(pub Vec<SegmentCell>);

impl SegmentCells {
    fn are_there_no_ones(&self) -> bool {
        !self
            .0
            .iter()
            .any(|cell| matches!(cell, SegmentCell::Bit(true)))
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
}

struct Nodes(pub Vec<Node>);

#[derive(Debug, PartialEq, Eq)]
enum HaltingSegmentResult {
    MachineDoesNotHalt(usize),
    CannotConclude(usize),
    NodeLimitExceeded,
}

fn get_initial_nodes(tm: &TM, segment_size: u8, initial_pos_in_segment: usize) -> Vec<Node> {
    assert!(initial_pos_in_segment < segment_size as usize);
    let mut to_return: Vec<Node> = Vec::new();
    for i_state in 0..tm.n_states {
        for symbol in 0..tm.n_symbols {
            let transition = tm.transitions[i_state as usize][symbol as usize];
            if let HaltOrGoto::Halt = transition.goto {
                let mut initial_segment = vec![SegmentCell::Unallocated; segment_size as usize];
                initial_segment[initial_pos_in_segment] = SegmentCell::Bit(u8_to_bool(symbol));
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

    let mut node_stack: Vec<Node> = initial_nodes;
    let mut node_seen: HashSet<Node> = HashSet::new();

    while !node_stack.is_empty() && node_seen.len() < node_limit {
        let curr_node = node_stack.pop().unwrap();

        if node_seen.contains(&curr_node) {
            continue;
        }

        if curr_node.is_fatal() {
            return HaltingSegmentResult::CannotConclude(node_seen.len() + 1);
        }

        node_stack.append(&mut curr_node.get_neighbours(&tm));
        node_seen.insert(curr_node.clone());

        if print_run_info {
            println!("{} ; Node: {}", curr_node, node_seen.len());
        }
    }

    if node_stack.is_empty() {
        HaltingSegmentResult::MachineDoesNotHalt(node_seen.len())
    } else {
        HaltingSegmentResult::NodeLimitExceeded
    }
}

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";
const PATH_TO_UNDECIDED_INDEX: &str = "../bb5_undecided_index";

fn Iijil_strategy(machine_id: u32, node_limit: usize, print_run_info: bool) -> bool {
    /* Implements @Iijil's strategy for running the backward halting segment decider:
        - The decider is run with all odd segment length until success or cumulative node limit is reached
        - Initial position in the segment is middle of it
    */
    let mut distance_to_segment_end: u8 = 1;
    let mut total_nodes_consumed = 0;

    let tm = TM::from_bbchallenge_id(machine_id, PATH_TO_BBCHALLENGE_DB).unwrap();
    if print_run_info {
        println!("Machine ID: {}", machine_id);
        println!("{}", tm);
    }
    while total_nodes_consumed < node_limit {
        let segment_size = 2 * distance_to_segment_end + 1;
        let initial_pos_in_segment = distance_to_segment_end as usize;

        if print_run_info {
            println!("Segment size: {}", segment_size);
        }

        let result = halting_segment_decider(
            &tm,
            segment_size,
            initial_pos_in_segment,
            node_limit,
            print_run_info,
        );

        match result {
            HaltingSegmentResult::MachineDoesNotHalt(nb_nodes) => {
                if print_run_info {
                    println!(
                        "Machine {} proved nonhalting with segment size {} and initial position {} after expanding {} nodes, and cumulatively {} nodes in search", machine_id,
                        segment_size, initial_pos_in_segment, nb_nodes, nb_nodes+total_nodes_consumed
                    );
                }
                return true;
            }

            HaltingSegmentResult::CannotConclude(nb_nodes) => {
                if print_run_info {
                    println!("Cannot conclude with segment size {} and initial position {}, {} nodes expanded",
                    segment_size, initial_pos_in_segment, nb_nodes);
                }
                total_nodes_consumed += nb_nodes;
            }
            HaltingSegmentResult::NodeLimitExceeded => {
                if print_run_info {
                    println!("Node limit exceeded");
                }
                return false;
            }
        }
        distance_to_segment_end += 1;
    }
    if print_run_info {
        println!("Node limit exceeded");
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

    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    let mut decided_ids: Vec<&u32> = undecided_ids
        .par_iter()
        .progress_with_style(style)
        .filter(|&id| Iijil_strategy(*id, NODE_LIMIT, false))
        .collect();

    decided_ids.sort();

    println!(
        "{} machines decided by halting segment (using @Iijil's strategy)",
        decided_ids.len()
    );

    let mut random_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    random_id = random_id.to_ascii_lowercase();

    let output_file =
        format!("output/halting-segment-reproduction-run-{random_id}-nodes-{NODE_LIMIT}");

    let mut file = File::create(output_file).unwrap();
    for id in decided_ids {
        file.write_all(&u32::to_be_bytes(*id)).unwrap();
    }
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
            HaltingSegmentResult::MachineDoesNotHalt(7)
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
            HaltingSegmentResult::MachineDoesNotHalt(38)
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
            HaltingSegmentResult::MachineDoesNotHalt(18)
        );
    }

    #[test]
    fn Iijil_strategy_23367211() {
        Iijil_strategy(23367211, 200000, true);
    }

    #[test]
    fn Iijil_strategy_missing() {
        let missing: Vec<u32> = vec![
            13185539, 3364358, 11660296, 6179850, 3364364, 6018060, 10361356, 5973009, 204306,
            75896850, 6397462, 6595606, 21812248, 11378722, 9931811, 11087396, 4454949, 7646244,
            1642018, 4040233, 5216298, 23367211, 4772908, 13132845, 11447341, 71394353, 13179441,
            14238771, 83583539, 1315382, 57396791, 11823670, 10781753, 1030713, 12347961, 8939582,
            5361728, 2319939, 9441862, 2319944, 4503112, 12910153, 11703371, 42589260, 11704393,
            13837896, 12873809, 9466449, 10104916, 4606045, 13174878, 2999391, 14114910, 5737057,
            8007786, 5164655, 13128818, 7892596, 6241396, 11593334, 5961336, 10668665, 12996734,
            6827136, 4517506, 3847298, 10720391, 10783368, 10749579, 2887824, 5767828, 44381333,
            5981844, 4711065, 14214297, 261277, 10903712, 6335137, 2896547, 45632675, 6646437,
            10841766, 5540006, 4777640, 11895461, 12092583, 13262502, 9385133, 7516339, 8640179,
            9113781, 1956533, 11716279, 5885623, 7861947, 4664509, 2841085, 6265538, 53705930,
            7245516, 650445, 11761872, 6396116, 4667096, 1395929, 42648284, 3390688, 4530401,
            8741604, 10304236, 10316013, 518900, 9385205, 3855608, 6362874, 7866106, 1854719,
            10819331, 8039684, 2181383, 45216519, 2730251, 8972556, 7792396, 3333392, 13482773,
            42617110, 3816217, 14049562, 3853091, 9076517, 5961514, 59653931, 5439788, 9102125,
            8039211, 23404853, 22009664, 3128136, 7141709, 873297, 3809106, 10680149, 12378966,
            11918684, 67603806, 6271838, 9876833, 6102369, 68522851, 13984102, 24772455, 11703149,
            9366382, 13148527, 6223726, 43396983, 6598520, 8171895, 6183289, 718203, 7549437,
            55091076, 69584265, 6758800, 14762385, 7270805, 5962645, 13940631, 6205849, 9891739,
            12474780, 7036834, 50813859, 13315491, 5914530, 6139815, 14272432, 11419569, 1798583,
            9269180, 8199612, 6545854, 7317441, 9089986, 511429, 3249608, 7793609, 1891273,
            11416011, 22735819, 13910993, 2650579, 58982868, 4742616, 6598620, 1169884, 9562083,
            2677222, 6561259, 3840491, 8948721, 6755314, 1315314, 68527089, 29798901, 6130682,
            29798909,
        ];

        for id in &missing {
            let mut node_limit = 20000;
            let mut r = Iijil_strategy(*id, node_limit, false);
            while !r && node_limit < 200000000 {
                node_limit *= 10;
                println!("Machine {} up {}", id, node_limit);
                r = Iijil_strategy(*id, node_limit, false);
            }
            assert!(r);
        }
    }
}
