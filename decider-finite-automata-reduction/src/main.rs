pub mod core;
pub mod driver;
pub mod io;
pub mod provers;

use argh::FromArgs;
use driver::{process_remote, run_node, DeciderProgress, DeciderProgressIterator};
use io::{Database, DeciderVerificationFile, Index, OutputFile, OWN_DVF, OWN_INDEX};
use provers::{prover_names, prover_range_by_name, ProverBox};

const DEFAULT_DB: &str = "../all_5_states_undecided_machines_with_global_header";
const DEFAULT_INDEX: &str = "../bb5_undecided_index";

#[derive(FromArgs)]
/// Find non-halting TMs, as witnessed by finite-state recognizers for their halting configurations.
pub struct DeciderArgs {
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

    /// run as a server -- wait for you to start corresponding clients and run in parallel
    #[argh(switch, short = 's')]
    server: bool,

    /// server IP address
    #[argh(option, default = "String::new()")]
    ip: String,

    /// server port
    #[argh(option, default = "25122")]
    port: u16,
}

fn main() -> std::io::Result<()> {
    let mut args: DeciderArgs = argh::from_env();
    let db = Database::open(&args.db)?;
    if !args.ip.is_empty() && !args.server {
        run_node(args, db);
        return Ok(());
    }
    let mut index = Index::open(&args.index).unwrap_or_else(|_| Index::new(db.len()));
    let mut provers: Vec<ProverBox> = vec![];
    if args.prover.is_empty() && !args.merge_only {
        args.prover.extend(prover_names());
    }
    for (i, name) in args.prover.iter().enumerate() {
        let lo = args.exclude.get(i).map_or(usize::MIN, |x| x + 1);
        let hi = args.limit.get(i).map_or(usize::MAX, |l| l + 1);
        provers.extend(prover_range_by_name(name, lo..hi));
    }

    let mut out = OutputFile::append(OWN_INDEX)?;
    let mut dvf = DeciderVerificationFile::append(OWN_DVF)?;
    let progress = DeciderProgress::new(index.len_initial());
    if !args.server {
        for prover in provers.iter_mut() {
            process_local(&db, &mut index, &progress, prover, &mut out, &mut dvf)?;
        }
    } else {
        let prover_names = provers.into_iter().map(|p| p.name()).collect();
        process_remote(args, index, progress, prover_names, out, dvf);
    }
    Ok(())
}

fn process_local(
    db: &Database,
    index: &mut Index,
    progress: &DeciderProgress,
    prover: &mut ProverBox,
    out: &mut OutputFile,
    dvf: &mut DeciderVerificationFile,
) -> std::io::Result<()> {
    index.read_decided()?;
    progress.set_solved(index.len_solved());
    for (i, tm) in db.read(index.iter().decider_progress_with(&progress, prover.name())) {
        if let Some(proof) = prover.prove(&tm) {
            match proof.validate(&tm) {
                Ok(()) => {
                    out.insert(i)?;
                    dvf.insert(i, proof.automaton.direction, &proof.automaton.dfa)?;
                    progress.solve(1);
                }
                Err(e) => {
                    let name = prover.name();
                    let msg = format!("Rejected {} proof of {} ({}): {:?}", name, i, &tm, e);
                    progress.println(msg)?;
                }
            }
        }
    }
    Ok(())
}
