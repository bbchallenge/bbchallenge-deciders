use decider_bouncers_reproduction::directional_tm::TMError;
use decider_bouncers_reproduction::formula_tape::bouncer_certificate::BouncerCertificate;
use decider_bouncers_reproduction::formula_tape::bouncers_decider::bouncers_decider;
use decider_bouncers_reproduction::formula_tape::FormulaTapeError;

use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

fn main() -> io::Result<()> {
    let file = File::open("bb5_undecided_machines.csv")?;
    let reader = BufReader::new(file);

    let mut i = 0;
    let mut machines: Vec<String> = Vec::new();
    for line in reader.lines() {
        if i == 0 {
            i += 1;
            continue;
        }

        let line = line?;
        let machine_std_format = line.split(',').collect::<Vec<&str>>()[1];
        machines.push(machine_std_format.to_string());

        i += 1;
    }

    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    let mut results: Vec<(String, Result<Option<BouncerCertificate>, FormulaTapeError>)> = machines
        .par_iter()
        .progress_with_style(style)
        .map(|machine_std_format| {
            (
                machine_std_format.clone(),
                bouncers_decider(machine_std_format, 250000, 50000, 114, false),
            )
        })
        .collect();

    let mut nb_solved = 0;
    for (machine, res) in results.iter_mut() {
        if let Ok(Some(cert)) = res {
            //println!("{}", cert.formula_tape);
            nb_solved += 1;
        } else {
            println!("{}", machine);
        }

        let res = match res {
            Ok(res) => res,
            Err(e) => match e {
                FormulaTapeError::TMError(TMError::MachineHasHalted) => {
                    continue;
                }
                FormulaTapeError::NoShiftRule => {
                    continue;
                }
                FormulaTapeError::InvalidFormulaTapeError => {
                    println!("{}", machine);
                    panic!("Error: {:?}", e);
                }
                _ => {
                    println!("{}", machine);
                    panic!("Error: {:?}", e);
                }
            },
        };
    }

    println!("{}", nb_solved);

    Ok(())
}
