use std::collections::HashSet;

use super::*;
#[test]
/// Proving that https://bbchallenge.org/43477769 is a bouncer (i.e. non-halting) from a given formula tape (i.e. formula tape not guessed).
fn no_guessing_prove_bouncer_43_477_769() {
    let machine_str = "1RB0RD_1LC1LE_1RA1LB_---0RC_1LB0LE";
    let formula_tape_str =
        "0∞<B100001111011110111101111011110(11110111101111011110)0001111011110(1111011110)00011110(11110)00011110(11110)011111110∞";
    let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    formula_tape.set_machine_str(machine_str);
    assert_eq!(format!("{formula_tape}"), formula_tape_str);

    let cert = formula_tape.prove_non_halt(200_000, 0).unwrap().unwrap();
    assert_eq!(cert.num_macro_steps_until_special_case, 1892);
}

#[test]
/// Proving that https://bbchallenge.org/88427177 is a bouncer (i.e. non-halting) from a given formula tape (i.e. formula tape not guessed).
fn no_guessing_prove_bouncer_88_427_177() {
    let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    let formula_tape_str = "0∞(111)1110(11)00D>0∞";
    let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    formula_tape.set_machine_str(machine_str);
    assert_eq!(format!("{formula_tape}"), formula_tape_str);

    let cert = formula_tape.prove_non_halt(200_000, 0).unwrap().unwrap();
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

    let cert = formula_tape.prove_non_halt(200_000, 0).unwrap().unwrap();
    assert_eq!(cert.num_macro_steps_until_special_case, 13);
}

#[test]
fn decide_bouncer_43_477_769() {
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB0RD_1LC1LE_1RA1LB_---0RC_1LB0LE";
    let cert: BouncerCertificate = bouncers_decider(machine_str, 6000, 2000, 10)
        .unwrap()
        .unwrap();
    assert_eq!(
        cert.formula_tape.to_string(),
        "0∞<B1000011110(11110111101111011110)000(1111011110)000(11110)000(11110)011111110∞"
    );
    assert_eq!(cert.num_steps_until_formula_tape, 3215);
    assert_eq!(cert.num_macro_steps_until_special_case, 1118);
}

#[test]
fn decide_bouncer_88_427_177() {
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    let cert: BouncerCertificate = bouncers_decider(machine_str, 200, 2000, 10)
        .unwrap()
        .unwrap();
    assert_eq!(cert.formula_tape.to_string(), "0∞111(111)01111(11)C>0∞");
    assert_eq!(cert.num_steps_until_formula_tape, 121);
    assert_eq!(cert.num_macro_steps_until_special_case, 41);
}

#[test]
fn decide_bouncer_6_416_853() {
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB0LC_0LA1RC_0LD0LE_1LA1RA_---1LC";
    let cert = bouncers_decider(machine_str, 1000, 2000, 10)
        .unwrap()
        .unwrap();

    assert_eq!(cert.formula_tape.to_string(), "0∞<A10(10)00(0)0∞");
    assert_eq!(cert.num_steps_until_formula_tape, 33);
    assert_eq!(cert.num_macro_steps_until_special_case, 13);
}

#[test]
fn decider_bouncer_892_918() {
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB---_0LC0RB_1RA1LD_1LE1LD_1LB1LC";
    let cert = bouncers_decider(machine_str, 10000, 10000, 10)
        .unwrap()
        .unwrap();

    assert_eq!(
        cert.formula_tape.to_string(),
        "0∞<B11111011110(11110111101111011110)0∞"
    );
    assert_eq!(cert.num_steps_until_formula_tape, 1217);
    assert_eq!(cert.num_macro_steps_until_special_case, 406);
}

#[test]
fn decider_bouncer_13_138_739() {
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB1LD_1RC0RC_1RD1RA_1LE1LA_---0LA";
    let cert = bouncers_decider(machine_str, 10000, 10000, 10)
        .unwrap()
        .unwrap();

    assert_eq!(
        cert.formula_tape.to_string(),
        "0∞<A1101(111111101111111101111111101111111101)0∞"
    );
    assert_eq!(cert.num_steps_until_formula_tape, 1855);
    assert_eq!(cert.num_macro_steps_until_special_case, 514);
}

#[test]
fn decider_bouncer_83_795_500() {
    // This bouncer encounters a looper shift rule on
    // 0∞110011001100A>(11000100)01000100010∞
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB1LD_1RC1RE_1LA0LC_0RA0LA_0RD---";
    let cert = bouncers_decider(machine_str, 200, 50000, 10)
        .unwrap()
        .unwrap();

    assert_eq!(cert.formula_tape.to_string(), "0∞<A011(00010001)0∞");
    assert_eq!(cert.num_steps_until_formula_tape, 99);
    assert_eq!(cert.num_macro_steps_until_special_case, 37);
}

#[test]
fn decider_bouncer_87_860_001() {
    // This bouncer was the only machine that mei's implem decided within 10k that cosmo's didnt
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB1LE_1LC---_1LD0LC_0RA0RE_1RD0LB";
    let cert = bouncers_decider(machine_str, 10000, 10000, 10)
        .unwrap()
        .unwrap();

    assert_eq!(
        cert.formula_tape.to_string(),
        "0∞10101101(101)011010(11010)A>0∞"
    );
    assert_eq!(cert.num_steps_until_formula_tape, 222);
    assert_eq!(cert.num_macro_steps_until_special_case, 74);
}

#[test]
fn decider_bouncer_347_505() {
    // This bouncer was the only machine that mei's implem decided within 10k that cosmo's didnt
    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB---_0RC1RD_0LD1RC_1LE0RA_1RA0LE";
    let cert = bouncers_decider(machine_str, 250000, 50000, 20)
        .unwrap()
        .unwrap();

    assert_eq!(
        cert.formula_tape.to_string(),
        "0∞11001011010100100101011010110101001001010110101101010010010101101011010100100101011(01011010100100101011010110101001001010110101101010010010101101011010100100101011)1011011010(1011011010)01001110110110111011011011101101101110110110111011011011101101101(1101101101110110110111011011011101101101110110110111011011011101101101)001001B>0∞"
    );

    assert_eq!(cert.num_steps_until_formula_tape, 124541);
    assert_eq!(cert.num_macro_steps_until_special_case, 41628);
}

#[test]
fn decider_bouncer_9_756_305() {
    // This bouncer was the only machine that mei's implem decided within 250k that cosmo didnt with 250k steps, 50k macro steps and 100 formula tapes tested per head

    use super::bouncers_decider::bouncers_decider;
    let machine_str = "1RB1RD_1RC0LC_0LD0RB_0RE1LC_1RA---";
    let cert = bouncers_decider(machine_str, 250000, 50000, 10)
        .unwrap()
        .unwrap();

    assert_eq!(cert.formula_tape.to_string(), "0∞0111010101(01)C>0∞");
    assert_eq!(cert.num_steps_until_formula_tape, 206);
    assert_eq!(cert.num_macro_steps_until_special_case, 45);
}
