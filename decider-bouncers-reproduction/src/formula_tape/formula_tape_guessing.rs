use self::directional_tm::TMError;

use super::*;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::Display,
    vec,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum FormulaTapeAtoms {
    Symbol(u8),
    Repeater(Vec<u8>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct FormulaTapeSearchConfiguration {
    pos_tape1: usize,
    pos_tape2: usize,
    pos_tape3: usize,
    proto_formula_tape: Vec<FormulaTapeAtoms>,
}

impl std::fmt::Display for FormulaTapeSearchConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}, {}, {}",
            self.pos_tape1, self.pos_tape2, self.pos_tape3
        )?;
        for atom in self.proto_formula_tape.iter() {
            match atom {
                FormulaTapeAtoms::Symbol(symbol) => write!(f, "{}", symbol)?,
                FormulaTapeAtoms::Repeater(repeater) => write!(f, "({})", v2s(repeater))?,
            }
        }
        write!(f, "")
    }
}

fn proto_formula_tape_to_formula_tape(
    machine_str: &str,
    head: TapeHead,
    proto_formula_tape: Vec<FormulaTapeAtoms>,
) -> FormulaTape {
    let mut tape_content = vec![];
    let mut repeaters_pos: Vec<RepeaterPos> = vec![];

    let offset = if head.pointing_direction == Direction::LEFT {
        2
    } else {
        1
    };

    //println!("HEY HEY {:?}", proto_formula_tape);
    let mut repeater_offset = 0;
    for (i, atom) in proto_formula_tape.iter().enumerate() {
        match atom {
            FormulaTapeAtoms::Symbol(symbol) => tape_content.push(*symbol),
            FormulaTapeAtoms::Repeater(repeater) => {
                tape_content.extend(repeater);
                repeaters_pos.push(RepeaterPos {
                    beg: offset + repeater_offset + i,
                    end: offset + repeater_offset + i + repeater.len(),
                });
                repeater_offset += repeater.len() - 1;
            }
        }
    }

    let tape = if head.pointing_direction == Direction::LEFT {
        Tape::new(machine_str, &[], head, &tape_content)
    } else {
        Tape::new(machine_str, &tape_content, head, &[])
    };

    FormulaTape {
        tape,
        repeaters_pos,
    }
}

impl From<(usize, usize, usize)> for FormulaTapeSearchConfiguration {
    fn from((pos_tape1, pos_tape2, pos_tape3): (usize, usize, usize)) -> Self {
        Self {
            pos_tape1,
            pos_tape2,
            pos_tape3,
            proto_formula_tape: vec![],
        }
    }
}

impl From<(usize, usize, usize, Vec<FormulaTapeAtoms>)> for FormulaTapeSearchConfiguration {
    fn from(
        (pos_tape1, pos_tape2, pos_tape3, proto_formula_tape): (
            usize,
            usize,
            usize,
            Vec<FormulaTapeAtoms>,
        ),
    ) -> Self {
        Self {
            pos_tape1,
            pos_tape2,
            pos_tape3,
            proto_formula_tape,
        }
    }
}

fn remove_head_and_infinite_0(tape: Tape) -> Vec<u8> {
    let tape = tape.clone();
    return tape
        .tape_content
        .iter()
        .filter(|content| match content {
            TapeContent::Head(_) => false,
            TapeContent::InfiniteZero => false,
            _ => true,
        })
        .map(|content| match content {
            TapeContent::Symbol(symbol) => symbol,
            _ => panic!("Unexpected content"),
        })
        .copied()
        .collect::<Vec<u8>>();
}

fn fit_formula_tape_from_triple(tape1: Tape, tape2: Tape, tape3: Tape) -> Option<FormulaTape> {
    //println!("Testing tiplet");

    let machine_str = tape1.machine_transition.machine_std_format.clone();
    let head = tape1.get_current_head().unwrap();
    let tape1 = remove_head_and_infinite_0(tape1);
    let tape2 = remove_head_and_infinite_0(tape2);
    let tape3 = remove_head_and_infinite_0(tape3);

    //println!("{}\n{}\n{}\n", v2s(&tape1), v2s(&tape2), v2s(&tape3));

    let n = tape1.len();

    let mut conf: VecDeque<FormulaTapeSearchConfiguration> = VecDeque::new();
    let mut conf_seen: HashSet<FormulaTapeSearchConfiguration> = HashSet::new();
    conf.push_front((0, 0, 0).into());

    while !conf.is_empty() {
        let curr_conf = conf.pop_front().unwrap();
        if conf.contains(&curr_conf) {
            continue;
        }
        //println!("{}", curr_conf);
        conf_seen.insert(curr_conf.clone());

        let FormulaTapeSearchConfiguration {
            pos_tape1,
            pos_tape2,
            pos_tape3,
            proto_formula_tape,
        } = curr_conf;

        if pos_tape1 == tape1.len() - 1
            && pos_tape2 == tape2.len() - 1
            && pos_tape3 == tape3.len() - 1
        {
            let formula_tape =
                proto_formula_tape_to_formula_tape(&machine_str, head, proto_formula_tape);
            println!("FOUND: {}", formula_tape);
            return Some(formula_tape);
        }

        if pos_tape1 < tape1.len()
            && pos_tape2 < tape2.len()
            && pos_tape3 < tape3.len()
            && tape1[pos_tape1] == tape2[pos_tape2]
            && tape2[pos_tape2] == tape3[pos_tape3]
        {
            let mut proto_formula_tape = proto_formula_tape.clone();
            proto_formula_tape.push(FormulaTapeAtoms::Symbol(tape1[pos_tape1]));
            let to_push: FormulaTapeSearchConfiguration = (
                pos_tape1 + 1,
                pos_tape2 + 1,
                pos_tape3 + 1,
                proto_formula_tape,
            )
                .into();
            //println!("CASE 1\n{}", to_push);
            conf.push_front(to_push);
        }

        for i in (pos_tape2 + 1..=tape2.len()).rev() {
            let prefix: &[u8] = &tape2[pos_tape2..i];

            if pos_tape3 + prefix.len() >= tape3.len()
                || pos_tape3 + 2 * prefix.len() >= tape3.len()
            {
                continue;
            }

            if &tape3[pos_tape3..pos_tape3 + prefix.len()] == prefix
                && &tape3[pos_tape3 + prefix.len()..pos_tape3 + 2 * prefix.len()] == prefix
            {
                let mut proto_formula_tape = proto_formula_tape.clone();
                proto_formula_tape.push(FormulaTapeAtoms::Repeater(Vec::from(prefix)));
                let to_push: FormulaTapeSearchConfiguration = (
                    pos_tape1,
                    pos_tape2 + prefix.len(),
                    pos_tape3 + 2 * prefix.len(),
                    proto_formula_tape.clone(),
                )
                    .into();
                //println!("CASE 2\n{}", to_push);

                conf.push_back(to_push);
            }
        }
    }

    for i in 0..n {}

    None
}

fn guess_formula_tape_given_record_breaking_tapes(
    record_breaking_tapes: &Vec<Tape>,
) -> Option<FormulaTape> {
    for tape in record_breaking_tapes.iter() {
        //println!("{} {} {}", tape, tape.len(), tape.step_count);
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
            let step_diff2 = diff_s1_s2 - diff_s2_s3;

            let step4 = tape3.step_count - (diff_s2_s3 - step_diff2);

            if step4 < 0 {
                continue;
            }

            // //println!(
            //     "{} {} {} {} (l={}, d2={})",
            //     tape1.step_count, tape2.step_count, tape3.step_count, step4, len_diff, step_diff2
            // );

            // Ignoring subsequences
            // A subsesquence happen when there is (other_len_diff, other_step_diff2) such that:
            // - `len_diff` is a multiple of `other_len_diff`
            // - `step_diff2` is a equal to `other_step_diff2 * (len_diff / other_len_diff)^2`
            let mut ignore_triple = false;
            for (other_len_diff, other_step_diff2) in tested_len_diff_and_step_diff2.iter() {
                if len_diff % other_len_diff == 0
                    && step_diff2
                        == (other_step_diff2 * (i32::pow((len_diff / other_len_diff) as i32, 2)))
                {
                    //println!("Ignoring subsequence\n");
                    ignore_triple = true;
                    break;
                }
            }

            if ignore_triple {
                continue;
            }

            tested_len_diff_and_step_diff2.push((len_diff, step_diff2));

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
        //println!("HEAD {}", head);
        if let Some(formula_tape) = guess_formula_tape_given_record_breaking_tapes(tapes) {
            return Ok(Some(formula_tape));
        }
        //println!();
    }

    Ok(None)
}
