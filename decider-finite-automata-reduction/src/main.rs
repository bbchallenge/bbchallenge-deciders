pub mod core;
pub mod io;
pub mod provers;

use argh::FromArgs;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use io::{Database, Index, OutputFile};
use provers::{DirectProver, MitMDFAProver, Prover, ProverOptions};

const DEFAULT_DB: &str = "../all_5_states_undecided_machines_with_global_header";
const DEFAULT_INDEX: &str = "../bb5_undecided_index";
type ProverList = Vec<Box<dyn Prover>>;

#[derive(FromArgs)]
/// Find non-halting TMs, as witnessed by finite-state recognizers for their halting configurations.
struct DeciderArgs {
    /// path to the DB file
    #[argh(option, short = 'd', default = "String::from(DEFAULT_DB)")]
    db: String,

    /// path to the undecided index file (used if present)
    #[argh(option, short = 'i', default = "String::from(DEFAULT_INDEX)")]
    index: String,

    /// simply merge output files and exit
    #[argh(switch, short = 'm')]
    merge_only: bool,

    /// a prover to use: "direct" or "mitm_dfa". -l/-x options correspond to -p's in the same order
    #[argh(option, short = 'p')]
    prover: Vec<String>,

    /// limit on search depth (DFA size) for corresponding prover
    #[argh(option, short = 'l')]
    limit: Vec<usize>,

    /// excluded search depth (DFA size) for corresponding prover: assume it's already done
    #[argh(option, short = 'x')]
    exclude: Vec<usize>,
}

fn main() -> std::io::Result<()> {
    let mut args: DeciderArgs = argh::from_env();
    let db = Database::open(&args.db)?;
    let mut index = Index::open(&args.index).unwrap_or_else(|_| Index::new(db.len()));
    let mut provers: ProverList = vec![];
    if args.prover.is_empty() && !args.merge_only {
        args.prover.extend(["direct", "mitm_dfa"].map(String::from));
    }
    for (i, name) in args.prover.iter().enumerate() {
        match name.as_str() {
            "direct" => provers_from_args::<DirectProver>(&mut provers, &args, i),
            "mitm_dfa" => provers_from_args::<MitMDFAProver>(&mut provers, &args, i),
            _ => {}
        }
    }

    let progress = MultiProgress::new();
    progress.set_move_cursor(true);
    let index_progress = progress.add(ProgressBar::new(index.len_initial() as u64));
    let prover_style =
        ProgressStyle::with_template("{wide_bar} {pos:>7}/{len:7} ~{eta_precise:8} {msg} ")
            .unwrap();

    for prover in provers.iter_mut() {
        let mut out = OutputFile::new();
        index.resume()?;
        index_progress.set_position(index.len_solved() as u64);
        let prover_progress = progress
            .add(ProgressBar::new(index.len_unsolved() as u64))
            .with_style(prover_style.clone())
            .with_message(prover.name())
            .with_finish(ProgressFinish::AndLeave);
        for (i, tm) in db.read(index.iter().progress_with(prover_progress)) {
            match prover.prove(&tm).map(|proof| proof.validate(&tm)) {
                Some(Ok(())) => {
                    out.insert(i)?;
                    index_progress.inc(1);
                }
                Some(Err(e)) => {
                    println!("{}, error, {:?}", i, e);
                }
                None => {}
            }
        }
    }
    Ok(())
}

fn provers_from_args<T>(provers: &mut ProverList, args: &DeciderArgs, i: usize)
where
    T: Prover + ProverOptions + 'static,
{
    let range = T::depths();
    let lo = args.exclude.get(i).map_or(range.start, |x| x + 1);
    let hi = args.limit.get(i).map_or(range.end, |l| l + 1);
    for depth in lo..hi {
        provers.push(Box::new(T::new(depth)));
    }
}
