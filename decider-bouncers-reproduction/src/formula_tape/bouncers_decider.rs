use super::*;
use crate::directional_tm::*;
use itertools::Itertools;

use super::formula_tape_guessing::guess_formula_tape_given_record_breaking_tapes;
use std::collections::HashMap;

pub fn bouncers_decider(
    machine_str: &str,
    step_limit: usize,
    macro_step_limit: usize,
    formula_tape_limit: usize,
) -> Result<Option<BouncerCertificate>, FormulaTapeError> {
    let mut tape = Tape::new_initial(machine_str);

    // Storing record breaking tapes per head
    let mut record_breaking_tapes: HashMap<TapeHead, Vec<Tape>> = HashMap::new();

    record_breaking_tapes.insert(tape.get_current_head()?, vec![tape.clone()]);

    let mut max_tape_len = 0;

    for _ in 0..step_limit {
        tape.step()?;

        if tape.get_current_read_pos()? == 0 || tape.get_current_read_pos()? == tape.len() - 1 {
            max_tape_len = tape.len();
            match record_breaking_tapes.get_mut(&tape.get_current_head()?) {
                Some(tapes) => {
                    tapes.push(tape.clone());
                }
                None => {
                    record_breaking_tapes.insert(tape.get_current_head()?, vec![tape.clone()]);
                }
            }
        }
    }

    let mut num_formula_tested = 0;
    for head in record_breaking_tapes.keys().sorted() {
        let tapes = record_breaking_tapes.get(head).unwrap();
        //println!("HEAD {}", head);
        if let Some((mut formula_tape, step_count)) =
            guess_formula_tape_given_record_breaking_tapes(tapes)
        {
            num_formula_tested += 1;
            if let Some(cert) = formula_tape.prove_non_halt(macro_step_limit, step_count)? {
                return Ok(Some(cert.clone()));
            }

            if num_formula_tested >= formula_tape_limit {
                return Ok(None);
            }
        }
        //println!();
    }

    Ok(None)
}
