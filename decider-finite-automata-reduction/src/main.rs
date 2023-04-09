#![allow(clippy::len_without_is_empty)]
pub mod core;
pub mod driver;
pub mod io;
pub mod provers;

use crate::core::{Machine, Proof};
use argh::FromArgs;
use driver::{process_remote, run_node, DeciderProgress, DeciderProgressIterator};
use io::{Database, DeciderVerificationFile, Index, MachineID, OutputFile, OWN_DVF, OWN_INDEX};
use provers::{prover_names, prover_range_by_name, ProverBox};
use serde_json::{json, to_string_pretty, Value};

const DEFAULT_DB: &str = "../all_5_states_undecided_machines_with_global_header";
const DEFAULT_INDEX: &str = "../bb5_undecided_index";

#[derive(FromArgs)]
#[argh(
    description = "Decide TMs, using finite-state recognizers for their halting configurations.",
    example = "# Analyze individual machines:\n\
    $ {command_name} -a 7410754 -a 1RB0RC_0LC1LE_0RD1LB_1RA1RC_0LB---\n\
    # Parallel processing:\n\
    $ {command_name} --server --ip 10.0.0.1 -p direct -x 0 -l 8 -p mitm_dfa -x 8\n\
    # And in other terminal tabs and/or computers on the network, once per CPU\n\
    $ {command_name} --ip 10.0.0.1 &"
)]
pub struct DeciderArgs {
    /// prover(s) to use: see example
    #[argh(option, short = 'p')]
    prover: Vec<String>,

    /// maximum search depth (DFA size) for corresponding prover
    #[argh(option, short = 'l')]
    limit: Vec<usize>,

    /// exclude search depth (DFA size) for corresponding prover
    #[argh(option, short = 'x')]
    exclude: Vec<usize>,

    /// run as a server; clients will solve in parallel
    #[argh(switch, short = 's')]
    server: bool,

    /// server IP address
    #[argh(option, default = "String::new()")]
    ip: String,

    /// server port
    #[argh(option, default = "25122")]
    port: u16,

    /// analyze only the given TMs/seeds and show any proofs found
    #[argh(option, short = 'a')]
    ad_hoc: Vec<String>,

    /// path to the DB file
    #[argh(option, short = 'd', default = "String::from(DEFAULT_DB)")]
    db: String,

    /// path to the undecided index file (used if present)
    #[argh(option, short = 'i', default = "String::from(DEFAULT_INDEX)")]
    index: String,
}

fn main() -> std::io::Result<()> {
    let mut args: DeciderArgs = argh::from_env();
    let db = Database::open(&args.db)?;
    if !args.ip.is_empty() && !args.server {
        run_node(args, db);
        return Ok(());
    }
    let mut provers: Vec<ProverBox> = vec![];
    if args.prover.is_empty() {
        args.prover.extend(prover_names());
    }
    for (i, name) in args.prover.iter().enumerate() {
        let lo = args.exclude.get(i).map_or(usize::MIN, |x| x + 1);
        let hi = args.limit.get(i).map_or(usize::MAX, |l| l + 1);
        provers.extend(prover_range_by_name(name, lo..hi));
    }

    if !args.ad_hoc.is_empty() {
        return process_ad_hoc(args.ad_hoc, db, provers);
    }

    let mut index = Index::open(&args.index).unwrap_or_else(|_| Index::new(db.len()));
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
    for (i, tm) in db.read(index.iter().decider_progress_with(progress, prover.name())) {
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

fn process_ad_hoc(
    tm_specs: Vec<String>,
    db: Database,
    mut provers: Vec<ProverBox>,
) -> std::io::Result<()> {
    let mut proofs: Vec<Option<Proof>> = vec![None; tm_specs.len()];
    let mut unsolved_pos = vec![usize::default(); 0];
    let mut tms = vec![Machine::default(); tm_specs.len()];
    let mut seed_pos = vec![usize::default(); 0];
    let mut seed_ids = vec![MachineID::default(); 0];
    for (pos, tm_spec) in tm_specs.iter().enumerate() {
        if let Ok(id) = tm_spec.parse::<MachineID>() {
            seed_pos.push(pos);
            seed_ids.push(id);
        } else if let Ok(tm) = tm_spec.parse::<Machine>() {
            unsolved_pos.push(pos);
            tms[pos] = tm;
        }
    }
    for (&pos, (_, tm)) in seed_pos.iter().zip(db.read(seed_ids.into_iter())) {
        unsolved_pos.push(pos);
        tms[pos] = tm;
    }
    let progress = DeciderProgress::new(unsolved_pos.len());
    for (pos, tm_spec) in tm_specs.iter().enumerate() {
        if !unsolved_pos.contains(&pos) {
            progress.println(format!("Could not understand '{}'. Ignoring.", tm_spec))?;
        }
    }
    for prover in provers.iter_mut() {
        if unsolved_pos.is_empty() {
            break;
        }
        for pos in unsolved_pos
            .clone()
            .into_iter()
            .decider_progress_with(&progress, prover.name())
        {
            let tm = &tms[pos];
            if let Some(proof) = prover.prove(tm) {
                match proof.validate(tm) {
                    Ok(()) => {
                        proofs[pos] = Some(proof);
                        progress.solve(1);
                        unsolved_pos.retain(|&p| p != pos);
                    }
                    Err(e) => {
                        let name = prover.name();
                        let msg = format!("Rejected {} proof of {} ({}): {:?}", name, pos, &tm, e);
                        progress.println(msg)?;
                    }
                }
            }
        }
    }
    progress.finish();
    let results: Vec<Value> = tm_specs
        .iter()
        .zip(proofs)
        .filter_map(|(spec, proof)| Some(json!({"machine": spec, "proof": proof?})))
        .collect();
    println!("{}", to_string_pretty(&results).unwrap());
    Ok(())
}
