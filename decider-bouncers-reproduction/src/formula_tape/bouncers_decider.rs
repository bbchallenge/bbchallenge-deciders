use super::*;
use crate::directional_tm::*;
use itertools::Itertools;

use std::collections::HashMap;

// pub fn bouncers_decider(
//     machine_str: &str,
//     step_limit: usize,
//     macro_step_limit: usize,
//     formula_tape_limit: usize,
// ) -> Result<Option<BouncerCertificate>, FormulaTapeError> {
//     let mut tape = Tape::new_initial(machine_str);

//     // Storing record breaking tapes per head
//     let mut record_breaking_tapes: HashMap<TapeHead, Vec<Tape>> = HashMap::new();

//     record_breaking_tapes.insert(tape.get_current_head()?, vec![tape.clone()]);

//     for _ in 0..step_limit {
//         tape.step()?;

//         if tape.get_current_read_pos()? == 0 || tape.get_current_read_pos()? == tape.len() - 1 {
//             match record_breaking_tapes.get_mut(&tape.get_current_head()?) {
//                 Some(tapes) => {
//                     tapes.push(tape.clone());
//                 }
//                 None => {
//                     record_breaking_tapes.insert(tape.get_current_head()?, vec![tape.clone()]);
//                 }
//             }
//         }
//     }

//     let mut num_formula_tested = 0;
//     for head in record_breaking_tapes.keys().sorted() {
//         let tapes = record_breaking_tapes.get(head).unwrap();
//         //println!("HEAD {}", head);
//         if let Some((mut formula_tape, step_count)) =
//             guess_formula_tape_given_record_breaking_tapes(tapes)
//         {
//             num_formula_tested += 1;
//             //println!("Formula tape: {}", formula_tape);
//             if let Some(cert) = formula_tape.prove_non_halt(macro_step_limit, step_count)? {
//                 return Ok(Some(cert.clone()));
//             }

//             // if num_formula_tested >= formula_tape_limit {
//             //     return Ok(None);
//             // }
//         }
//         //println!();
//     }

//     Ok(None)
// }

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
        let res = solve_bouncer_given_record_breaking_tapes(tapes, macro_step_limit);
        if res.is_some() {
            return Ok(res);
        }
    }

    Ok(None)
}

use super::formula_tape_guessing::fit_formula_tape_from_triple;
use std::collections::HashSet;

pub fn solve_bouncer_given_record_breaking_tapes(
    record_breaking_tapes: &Vec<Tape>,
    macro_steps_limit: usize,
) -> Option<BouncerCertificate> {
    // for tape in record_breaking_tapes.iter() {
    //     println!("{} {} {}", tape, tape.len(), tape.step_count);
    // }

    let mut tested_formula_tapes: HashSet<FormulaTape> = HashSet::new();
    if record_breaking_tapes.len() < 4 {
        return None;
    }

    let mut tested_tape_length: HashSet<usize> = HashSet::new();

    for (i, tape1) in record_breaking_tapes.iter().enumerate() {
        for (i, tape2) in record_breaking_tapes.iter().skip(i + 1).enumerate() {
            let len_diff = tape2.len() - tape1.len();

            // // No need to test prefix of already tested sequence
            // if tested_tape_length.contains(&(len1 + len_diff)) {
            //     continue;
            // }

            let len3 = tape2.len() + len_diff;

            let tape3: &Tape =
                match record_breaking_tapes.binary_search_by_key(&len3, |tape| tape.len()) {
                    Ok(index) => &record_breaking_tapes[index],
                    Err(_) => continue,
                };

            // Testing quadratic sequence
            let diff_s2_s1 = tape2.step_count - tape1.step_count;
            let diff_s3_s2 = tape3.step_count - tape2.step_count;
            let step_diff2 = diff_s3_s2 - diff_s2_s1;

            let step4 = tape3.step_count + (diff_s3_s2 + step_diff2);

            // //println!(
            //     "{} {} {} {} (l={}, d2={})",
            //     tape1.step_count, tape2.step_count, tape3.step_count, step4, len_diff, step_diff2
            // );

            // Ignoring subsequences
            // A subsesquence happen when there is (other_len_diff, other_step_diff2) such that:
            // - `len_diff` is a multiple of `other_len_diff`
            // - `step_diff2` is a equal to `other_step_diff2 * (len_diff / other_len_diff)^2`
            // let mut ignore_triple = false;
            // for (other_len_diff, other_step_diff2) in tested_len_diff_and_step_diff2.iter() {
            //     if len_diff % other_len_diff == 0
            //         && step_diff2
            //             == (other_step_diff2 * (i32::pow((len_diff / other_len_diff) as i32, 2)))
            //     {
            //         //println!("Ignoring subsequence\n");
            //         ignore_triple = true;
            //         break;
            //     }
            // }

            // if ignore_triple {
            //     continue;
            // }

            //tested_len_diff_and_step_diff2.push((len_diff, step_diff2));

            match record_breaking_tapes.binary_search_by_key(&step4, |tape| tape.step_count) {
                Ok(tape4_index) => {
                    let tape4 = &record_breaking_tapes[tape4_index];
                    if tape3.len() > tape4.len() || tape4.len() - tape3.len() != len_diff {
                        continue;
                    }

                    match fit_formula_tape_from_triple(tape1.clone(), tape2.clone(), tape3.clone())
                    {
                        Some(mut formula_tape) => {
                            if tested_formula_tapes.contains(&formula_tape) {
                                //println!("Already tested formula tape");
                                continue;
                            }
                            //("Testing formula tape {}", formula_tape);
                            tested_formula_tapes.insert(formula_tape.clone());
                            let decider_res = formula_tape
                                .prove_non_halt(macro_steps_limit, tape3.step_count as usize);
                            if decider_res.is_ok() {
                                return decider_res.unwrap();
                            }
                        }
                        None => continue,
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        //tested_tape_length.insert(len1);
    }

    None
}
