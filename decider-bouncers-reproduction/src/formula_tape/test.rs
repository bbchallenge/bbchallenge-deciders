use super::*;
#[test]
/// Proving that https://bbchallenge.org/43477769 is a bouncer (i.e. non-halting) from a given formula tape (i.e. formula tape not guessed).
fn no_guessing_prove_bouncer_43_477_769() {
    let machine_str = "1RB0RD_1LC1LE_1RA1LB_---0RC_1LB0LE";
    let formula_tape_str =
        "0∞<E000011110(11110111101111011110)000(1111011110)000(11110)000(11110)011111110∞";
    let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    formula_tape.set_machine_str(machine_str);
    assert_eq!(format!("{formula_tape}"), formula_tape_str);

    let cert = formula_tape.prove_non_halt(200_000).unwrap().unwrap();
    assert_eq!(cert.num_macro_steps_until_special_case, 1118);
}

#[test]
/// Proving that https://bbchallenge.org/88427177 is a bouncer (i.e. non-halting) from a given formula tape (i.e. formula tape not guessed).
fn no_guessing_prove_bouncer_88_427_177() {
    let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    let formula_tape_str = "0∞(111)1110(11)00D>0∞";
    let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    formula_tape.set_machine_str(machine_str);
    assert_eq!(format!("{formula_tape}"), formula_tape_str);

    let cert = formula_tape.prove_non_halt(200_000).unwrap().unwrap();
    assert_eq!(cert.num_macro_steps_until_special_case, 41);
}

#[test]
/// Proving that https://bbchallenge.org/6416853 is a bouncer (i.e. non-halting) from a given formula tape (i.e. formula tape not guessed).
fn no_guessing_prove_bouncer_6_416_853() {
    let machine_str = "1RB0LC_0LA1RC_0LD0LE_1LA1RA_---1LC";
    let formula_tape_str = "0∞<C(10)00(0)0∞";
    let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    formula_tape.set_machine_str(machine_str);
    assert_eq!(format!("{formula_tape}"), formula_tape_str);

    let cert = formula_tape.prove_non_halt(200_000).unwrap().unwrap();
    assert_eq!(cert.num_macro_steps_until_special_case, 13);
}

#[test]
fn guess_formula_tape_6_416_853() {
    use formula_tape_guessing::guess_formula_tape;
    let machine_str = "1RB0LC_0LA1RC_0LD0LE_1LA1RA_---1LC";
    guess_formula_tape(machine_str, 100).unwrap();
    assert!(false)
}

#[test]
fn guess_formula_tape_43_477_769() {
    use formula_tape_guessing::guess_formula_tape;
    let machine_str = "1RB0RD_1LC1LE_1RA1LB_---0RC_1LB0LE";
    guess_formula_tape(machine_str, 5000).unwrap();
    assert!(false)
}
