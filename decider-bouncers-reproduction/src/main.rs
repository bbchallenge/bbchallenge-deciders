use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
use decider_bouncers_reproduction::formula_tape::{
    FormulaTape, FormulaTapeError, RepeaterPos, ShiftRule,
};
use std::str::FromStr;
fn main() {
    // let machine_str = "1RB0RD_1LC1LE_1RA1LB_---0RC_1LB0LE";
    // let formula_tape_str =
    //     "0∞<E000011110(11110111101111011110)000(1111011110)000(11110)000(11110)011111110∞";
    // let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    // formula_tape.set_machine_str(machine_str);
    // assert_eq!(format!("{formula_tape}"), formula_tape_str);

    // let res = formula_tape.prove_non_halt(200_000);
    // println!("{:?}", res);
    // let formula_tape_str =
    //     "0∞11111100111001A>(11110111101111011110)000(1111011110)000(11110)000(11110)011111110∞";
    // let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    // formula_tape.set_machine_str(machine_str);

    // formula_tape.detect_shift_rule().unwrap();

    // let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    // let formula_tape_str = "0∞(111)1110(11)00D>0∞";
    // let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    // formula_tape.set_machine_str(machine_str);
    // assert_eq!(format!("{formula_tape}"), formula_tape_str);

    // let res = formula_tape.prove_non_halt(200_000);
    // println!("{:?}", res);

    // let machine_str = "1RB0LC_0LA1RC_0LD0LE_1LA1RA_---1LC";
    // let formula_tape_str = "0∞<C(10)00(0)0∞";
    // let mut formula_tape = FormulaTape::from_str(formula_tape_str).unwrap();
    // formula_tape.set_machine_str(machine_str);
    // assert_eq!(format!("{formula_tape}"), formula_tape_str);

    // let res = formula_tape.prove_non_halt(100);
    // println!("{:?}", res);
}
