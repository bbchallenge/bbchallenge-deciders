use argh::FromArgs;
use decider_halting_segment_reproduction::{tm::TM, Iijil_strategy_updated};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rand::{distributions::Alphanumeric, Rng};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{Read, Write},
};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

const PATH_TO_BBCHALLENGE_DB: &str = "../all_5_states_undecided_machines_with_global_header";
// "../../../../busy_beaver_challenge_extras/res/all_5_states_undecided_machines_with_global_header";
/// https://github.com/bbchallenge/bbchallenge-undecided-index
const PATH_TO_UNDECIDED_INDEX: &str = "../bb5_undecided_index";

fn default_distance_to_end_limit() -> u8 {
    5
}

#[derive(FromArgs)]
/// Halting segment deciders using @Iijil's updated search strategy.
struct SearchArgs {
    /// maximum size from center of segment to extremity, i.e. total size of segment is 2x+1
    #[argh(option, short = 'n', default = "default_distance_to_end_limit()")]
    distance_to_end_limit: u8,
}

fn main() {
    let search_args: SearchArgs = argh::from_env();

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
        .filter(|&id| {
            let turing_machine = TM::from_bbchallenge_id(*id, PATH_TO_BBCHALLENGE_DB).unwrap();
            Iijil_strategy_updated(turing_machine, search_args.distance_to_end_limit, false)
        })
        .collect();

    decided_ids.sort();

    println!(
        "{} machines decided by halting segment, starting from center of odd-size segments up to size {} (using @Iijil's updated strategy)",
        decided_ids.len(), 2*search_args.distance_to_end_limit+1
    );

    let mut random_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    random_id = random_id.to_ascii_lowercase();

    let d = search_args.distance_to_end_limit;
    let output_file =
        format!("output/halting-segment-reproduction-run-{random_id}-max-distance-to-end-{d}");

    let mut file = File::create(output_file).unwrap();
    for id in decided_ids {
        file.write_all(&u32::to_be_bytes(*id)).unwrap();
    }
}
