pub mod core;
pub mod io;
pub mod provers;

use crate::core::Proof;
use io::Database;
use provers::{DirectProver, Prover};

// TODO: Command-line: resume,
fn main() {
    let mut provers: Vec<DirectProver> = (1..4).map(DirectProver::new).collect();
    if let Ok(db) = Database::open("/home/justinb/scratch/beaver/bbchallenge/bb5_undecided.db") {
        for (i, tm) in db.read(0..) {
            let mut res = None;
            for prover in &mut provers {
                res = prover.prove(&tm);
                if res.is_some() {
                    break;
                }
            }
            if let Some(automaton) = res {
                let steady_state = DirectProver::steady_state(automaton.dfa.len());
                let proof = Proof {
                    tm,
                    automaton,
                    steady_state,
                };
                if let Ok(()) = proof.validate() {
                    println!("{}, infinite", i);
                } else {
                    println!("{}, error", i)
                }
            }
        }
    }
}
