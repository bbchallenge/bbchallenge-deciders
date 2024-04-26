use super::*;
use crate::directional_tm::*;
use itertools::Itertools;

use std::{cell::Cell, process::exit};

use core::num;
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

    for _ in 0..step_limit {
        tape.step()?;

        if tape.get_current_read_pos()? == 0 || tape.get_current_read_pos()? == tape.len() - 1 {
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
        let res =
            solve_bouncer_given_record_breaking_tapes(tapes, macro_step_limit, formula_tape_limit);
        if res.is_some() {
            return Ok(res);
        }
    }

    Ok(None)
}

use super::formula_tape_guessing::{
    fit_formula_tape_from_triple_greedy_iterative_implem,
    fit_formula_tape_from_triple_recursive_implem,
};

fn is_quadratic(a: i32, b: i32, c: i32, d: i32) -> bool {
    let diff_ba = b - a;
    let diff_cb = c - b;
    let diff_dc = d - c;
    let diff2 = diff_cb - diff_ba;
    diff2 == (diff_dc - diff_cb)
}

pub fn solve_bouncer_given_record_breaking_tapes(
    record_breaking_tapes: &Vec<Tape>,
    macro_steps_limit: usize,
    formula_tape_limit: usize,
) -> Option<BouncerCertificate> {
    // for tape in record_breaking_tapes.iter() {
    //     println!("{} {} {}", tape, tape.len(), tape.step_count);
    // }

    if record_breaking_tapes.len() < 4 {
        return None;
    }

    let mut num_formula_tested = 0;

    for (i, tape4) in record_breaking_tapes.iter().enumerate() {
        if i < 3 {
            continue;
        }
        for (j, tape3) in record_breaking_tapes.iter().take(i).enumerate() {
            if j < 2 {
                continue;
            }
            let len_diff = tape4.len() - tape3.len();

            let tape2_len = tape3.len().checked_sub(len_diff);

            if tape2_len.is_none() {
                continue;
            }
            let tape2_len = tape2_len.unwrap();

            let tape2: &Tape =
                match record_breaking_tapes.binary_search_by_key(&tape2_len, |tape| tape.len()) {
                    Ok(index) => &record_breaking_tapes[index],
                    Err(_) => continue,
                };

            let tape1_len = tape2.len().checked_sub(len_diff);

            if tape1_len.is_none() {
                continue;
            }
            let tape1_len = tape1_len.unwrap();

            let tape1: &Tape =
                match record_breaking_tapes.binary_search_by_key(&tape1_len, |tape| tape.len()) {
                    Ok(index) => &record_breaking_tapes[index],
                    Err(_) => continue,
                };

            // Testing quadratic sequence
            if (!is_quadratic(
                tape1.step_count,
                tape2.step_count,
                tape3.step_count,
                tape4.step_count,
            )) {
                continue;
            }

            let res = fit_formula_tape_from_triple_greedy_iterative_implem(
                tape2.clone(),
                tape3.clone(),
                tape4.clone(),
            );

            match res {
                Some(mut formula_tape) => {
                    //println!("{}\n{}\n{}\n{}\n", tape1, tape2, tape3, formula_tape);

                    let decider_res =
                        formula_tape.prove_non_halt(macro_steps_limit, tape3.step_count as usize);

                    if let Ok(Some(_)) = decider_res {
                        return decider_res.unwrap();
                    }

                    num_formula_tested += 1;

                    if num_formula_tested == formula_tape_limit {
                        return None;
                    }

                    //println!("Continue search");
                }
                None => continue,
            }
        }
        //tested_tape_length.insert(len1);
    }

    None
}
