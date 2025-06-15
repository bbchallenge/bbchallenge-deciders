// pub mod display_nodes;
pub mod node;
pub mod tm;
pub mod utils;

use indexmap::IndexSet;

use crate::{
    node::{Node, NodeLimit, OutsideSegmentOrState, SegmentCell, SegmentCells},
    tm::{HaltOrGoto, TM},
    utils::u8_to_bool,
};

// #[cfg(test)]
pub const PATH_TO_BBCHALLENGE_DB_TEST: &str =
    "../../all_5_states_undecided_machines_with_global_header";

#[derive(Debug, PartialEq, Eq)]
enum HaltingSegmentResult {
    MachineDoesNotHalt(usize),
    CannotConclude(usize),
    NodeLimitExceeded,
}

fn get_initial_nodes(tm: &TM, segment_size: u8, initial_pos_in_segment: usize) -> IndexSet<Node> {
    assert!(initial_pos_in_segment < segment_size as usize);
    let mut to_return: IndexSet<Node> = IndexSet::new();
    for i_state in 0..tm.n_states {
        for symbol in 0..tm.n_symbols {
            let transition = tm.transitions[i_state as usize][symbol as usize];
            if let HaltOrGoto::Halt = transition.goto {
                let mut initial_segment = vec![SegmentCell::Unallocated; segment_size as usize];
                initial_segment[initial_pos_in_segment] = SegmentCell::Bit(u8_to_bool(symbol));
                to_return.insert(Node {
                    state: OutsideSegmentOrState::State(i_state),
                    segment: SegmentCells(initial_segment),
                    pos_in_segment: initial_pos_in_segment,
                });
            }
        }
    }

    to_return
}

fn halting_segment_decider(
    tm: &TM,
    segment_size: u8,
    initial_pos_in_segment: usize,
    node_limit: NodeLimit,
    print_run_info: bool,
) -> HaltingSegmentResult {
    let mut nodes = get_initial_nodes(&tm, segment_size, initial_pos_in_segment);
    let mut idx_seen = 0;

    while let Some(node) = nodes.get_index(idx_seen) {
        idx_seen += 1;

        if node.is_fatal() {
            return HaltingSegmentResult::CannotConclude(idx_seen);
        }

        if print_run_info {
            println!("{} ; Node: {}", node, idx_seen);
        }

        if let NodeLimit::NodeLimit(limit) = node_limit {
            if idx_seen > limit {
                return HaltingSegmentResult::NodeLimitExceeded;
            }
        }

        nodes.extend(node.get_neighbours(&tm));
    }

    HaltingSegmentResult::MachineDoesNotHalt(idx_seen)
}

#[allow(non_snake_case)]
pub fn Iijil_strategy(turing_machine: TM, node_limit: usize, print_run_info: bool) -> bool {
    /* Implements @Iijil's strategy for running the backward halting segment decider:
        - The decider is run with all odd segment length until success or cumulative node limit is reached
        - Initial position in the segment is middle of it

        UPDATE: the issue of this strategy is that results depend on the order used to add neighbours to the DFS stack. This hurts reproducibility so we are updating the strategy, see `Iijil_strategy_updated`.
    */
    let mut distance_to_segment_end: u8 = 1;
    let mut total_nodes_consumed = 0;

    if print_run_info {
        println!("Machine ID: {}", turing_machine.machine_id);
        println!("{}", turing_machine);
    }
    while total_nodes_consumed < node_limit {
        let segment_size = 2 * distance_to_segment_end + 1;
        let initial_pos_in_segment = distance_to_segment_end as usize;

        if print_run_info {
            println!("Segment size: {}", segment_size);
        }

        let result = halting_segment_decider(
            &turing_machine,
            segment_size,
            initial_pos_in_segment,
            NodeLimit::NodeLimit(node_limit),
            print_run_info,
        );

        match result {
            HaltingSegmentResult::MachineDoesNotHalt(nb_nodes) => {
                if print_run_info {
                    println!(
                        "Machine {} proved nonhalting with segment size {} and initial position {} after expanding {} nodes, and cumulatively {} nodes in search", turing_machine. machine_id,
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

#[allow(non_snake_case)]
pub fn Iijil_strategy_updated(
    turing_machine: TM,
    distance_to_end_limit: u8,
    print_run_info: bool,
) -> bool {
    /* Instead of using a node limit, like in `Iijil_strategy`, we exhaustively search all odd segment sizes
    up to `2*distance_to_end_limit+1` (and stop early when one reaches a conclusion), from middle starting position.

    This is because node limit make reproducibility hard when a different order is used to push nodes in the stack in varying
    implementions. We feel that this method is simpler than imposing an order on nodes.
    */

    let mut distance_to_segment_end: u8 = 1;
    let mut total_nodes_consumed = 0;

    // let turing_machine = TM::from_bbchallenge_id(machine_id, path_to_bb_challenge_db).unwrap();
    if print_run_info {
        println!("Machine ID: {}", turing_machine.machine_id);
        println!("{}", turing_machine);
    }
    while distance_to_segment_end <= distance_to_end_limit {
        let segment_size = 2 * distance_to_segment_end + 1;
        let initial_pos_in_segment = distance_to_segment_end as usize;

        if print_run_info {
            println!("Segment size: {}", segment_size);
        }

        let result = halting_segment_decider(
            &turing_machine,
            segment_size,
            initial_pos_in_segment,
            NodeLimit::NoLimit,
            print_run_info,
        );

        match result {
            HaltingSegmentResult::MachineDoesNotHalt(nb_nodes) => {
                if print_run_info {
                    println!(
                        "Machine {} proved nonhalting with segment size {} and initial position {} after expanding {} nodes, and cumulatively {} nodes in search", turing_machine. machine_id,
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

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn chaotic_machine() {
        // http://turbotm.de/~heiner/BB/TM4-proof.txt
        // Chaotic Machine [Marxen & Buntrock, 1990]
        let chaotic_machine_id = 76708232;
        let tm: TM =
            TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();
        assert_eq!(
            halting_segment_decider(&tm, 5, 2, NodeLimit::NodeLimit(1000), false),
            // 7 nodes expanded, cross checked with @Iijil
            HaltingSegmentResult::MachineDoesNotHalt(7)
        );
    }

    #[test]
    fn complex_counter() {
        // Complex Counter [Marxen & Buntrock, 1990]
        let chaotic_machine_id = 10936909;
        let tm: TM =
            TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();

        assert_eq!(
            halting_segment_decider(&tm, 7, 3, NodeLimit::NodeLimit(1000), false),
            // 38 nodes expanded, cross checked with @Iijil
            HaltingSegmentResult::MachineDoesNotHalt(38)
        );
    }

    #[test]
    fn machine_108115() {
        // bbchallenge machine 108115
        let chaotic_machine_id = 108115;
        let tm: TM =
            TM::from_bbchallenge_id(chaotic_machine_id, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();

        assert_eq!(
            halting_segment_decider(&tm, 3, 1, NodeLimit::NodeLimit(1000), false),
            // 18 nodes expanded, cross checked with @Iijil
            HaltingSegmentResult::MachineDoesNotHalt(18)
        );
    }

    #[test]
    fn Iijil_strategy_23367211() {
        let turing_machine =
            TM::from_bbchallenge_id(23367211, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();
        assert!(Iijil_strategy(turing_machine, 200000, true))
    }

    #[test]
    fn Iijil_strategy_updated_23367211() {
        let turing_machine =
            TM::from_bbchallenge_id(23367211, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();
        // Segment size 15 = 2*7+1
        assert!(Iijil_strategy_updated(turing_machine, 7, true))
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

        for &id in &missing {
            let mut node_limit = 20000;
            let turing_machine = TM::from_bbchallenge_id(id, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();
            let mut r = Iijil_strategy(turing_machine.clone(), node_limit, false);
            while !r && node_limit < 200000000 {
                node_limit *= 10;
                println!("Machine {} up {}", id, node_limit);
                r = Iijil_strategy(turing_machine.clone(), node_limit, false);
            }
            assert!(r);
        }
    }
}
