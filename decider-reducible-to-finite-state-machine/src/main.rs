pub mod core;
pub mod io;
pub mod provers;

use argh::FromArgs;
use io::Database;
use provers::{DirectProver, MitMDFAProver, Prover, ProverOptions};

const DEFAULT_DB: &str = "../all_5_states_undecided_machines_with_global_header";
type ProverList = Vec<Box<dyn Prover>>;

#[derive(FromArgs)]
/// Find non-halting TMs, as witnessed by finite-state recognizers for their halting configurations.
struct DeciderArgs {
    /// path to the DB file
    #[argh(option, short = 'd', default = "String::from(DEFAULT_DB)")]
    db: String,

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
    let db = Database::open(std::path::Path::new(&args.db))?;
    let mut provers: ProverList = vec![];
    if args.prover.is_empty() {
        args.prover.extend(["direct", "mitm_dfa"].map(String::from));
    }
    for (i, name) in args.prover.iter().enumerate() {
        match name.as_str() {
            "direct" => provers_from_args::<DirectProver>(&mut provers, &args, i),
            "mitm_dfa" => provers_from_args::<MitMDFAProver>(&mut provers, &args, i),
            _ => {}
        }
    }
    for (i, tm) in db.read(0..) {
        for prover in &mut provers {
            match prover.prove(&tm).map(|proof| proof.validate(&tm)) {
                Some(Ok(())) => {
                    println!("{}, infinite", i);
                    break;
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
