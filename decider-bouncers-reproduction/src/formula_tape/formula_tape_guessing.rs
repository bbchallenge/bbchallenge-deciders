use self::directional_tm::TMError;

use super::*;
use itertools::iproduct;

use std::collections::{HashMap, HashSet};

fn fit_formula_tape_from_triple(tape1: Tape, tape2: Tape, tape3: Tape) -> Option<FormulaTape> {
    println!("Testing tiplet");
    println!("{}\n{}\n{}\n", tape1, tape2, tape3);
    None
}

fn guess_formula_tape_given_record_breaking_tapes(
    record_breaking_tapes: &Vec<Tape>,
) -> Option<FormulaTape> {
    for tape in record_breaking_tapes.iter() {
        println!("{} {} {}", tape, tape.len(), tape.step_count);
    }

    if record_breaking_tapes.len() < 4 {
        return None;
    }

    let tapes_length = record_breaking_tapes.iter().rev().map(|tape| tape.len());

    let mut tested_tape_length: HashSet<usize> = HashSet::new();

    for len1 in tapes_length {
        let tape1_index = record_breaking_tapes
            .binary_search_by_key(&len1, |tape| tape.len())
            .unwrap();
        let tape1 = &record_breaking_tapes[tape1_index];

        let mut tested_len_diff_and_step_diff2: Vec<(usize, i32)> = Vec::new();
        for len2 in record_breaking_tapes
            .iter()
            .rev()
            .skip(record_breaking_tapes.len() - tape1_index)
            .map(|tape| tape.len())
        {
            let tape2_index = record_breaking_tapes
                .binary_search_by_key(&len2, |tape| tape.len())
                .unwrap();
            let tape2 = &record_breaking_tapes[tape2_index];

            let len_diff = len1 - len2;

            // No need to test prefix of already tested sequence
            if tested_tape_length.contains(&(len1 + len_diff)) {
                continue;
            }

            let len3 = tape2.len().checked_sub(len_diff);
            if len3.is_none() {
                continue;
            }
            let len3 = len3.unwrap();

            let tape3: &Tape =
                match record_breaking_tapes.binary_search_by_key(&len3, |tape| tape.len()) {
                    Ok(index) => &record_breaking_tapes[index],
                    Err(_) => continue,
                };

            // Testing quadratic sequence
            let diff_s1_s2 = tape1.step_count - tape2.step_count;
            let diff_s2_s3 = tape2.step_count - tape3.step_count;
            let diff2_s1_s2 = diff_s1_s2 - diff_s2_s3;
            let diff_s3_s4 = diff_s2_s3 - diff2_s1_s2;

            let step4 = tape3.step_count - diff_s3_s4;

            if step4 < 0 {
                continue;
            }

            println!(
                "{} {} {} {} (l={}, d2={})",
                tape1.step_count, tape2.step_count, tape3.step_count, step4, len_diff, diff2_s1_s2
            );

            // Ignoring subsequences
            // A subsesquence happen when there is (other_len_diff, other_step_diff2) such that:
            // - `len_diff` is a multiple of `other_len_diff`
            // - `diff2_s1_s2` is a equal to `other_step_diff2 * (lend_diff / other_len_diff)^2`
            let mut ignore_triple = false;
            for (other_len_diff, other_step_diff2) in tested_len_diff_and_step_diff2.iter() {
                if len_diff % other_len_diff == 0
                    && diff2_s1_s2
                        == (other_step_diff2 * (i32::pow((len_diff / other_len_diff) as i32, 2)))
                {
                    println!("Ignoring subsequence\n");
                    ignore_triple = true;
                    break;
                }
            }

            if ignore_triple {
                continue;
            }

            tested_len_diff_and_step_diff2.push((len_diff, diff2_s1_s2));

            match record_breaking_tapes.binary_search_by_key(&step4, |tape| tape.step_count) {
                Ok(tape4_index) => {
                    let tape4 = &record_breaking_tapes[tape4_index];
                    if tape3.len() - tape4.len() != len_diff {
                        continue;
                    }

                    match fit_formula_tape_from_triple(tape3.clone(), tape2.clone(), tape1.clone())
                    {
                        Some(formula_tape) => return Some(formula_tape),
                        None => continue,
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        tested_tape_length.insert(len1);
    }

    None
}

pub fn guess_formula_tape(
    machine_str: &str,
    step_limit: usize,
) -> Result<Option<FormulaTape>, TMError> {
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

    for (head, tapes) in record_breaking_tapes.iter() {
        println!("HEAD {}", head);
        if let Some(formula_tape) = guess_formula_tape_given_record_breaking_tapes(tapes) {
            return Ok(Some(formula_tape));
        }
        println!();
    }

    Ok(None)
}
