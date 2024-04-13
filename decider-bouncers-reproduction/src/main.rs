use decider_bouncers_reproduction::directional_tm::{Direction, Tape, TapeHead};
use decider_bouncers_reproduction::formula_tape::{
    FormulaTape, FormulaTapeError, RepeaterPos, ShiftRule,
};
fn main() {
    let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    let formula_tape = FormulaTape {
        tape: Tape::new(
            machine_str,
            &[1, 1, 1, 1, 1, 1, 0, 1, 1],
            TapeHead {
                state: 0,
                pointing_direction: Direction::LEFT,
            },
            &[0, 1, 0, 1, 0, 1, 1],
        ),
        repeaters_pos: vec![
            RepeaterPos { beg: 1, end: 4 },
            RepeaterPos { beg: 8, end: 10 },
        ],
    };
    assert_eq!(format!("{formula_tape}"), "0∞(111)1110(11)<A01010110∞");
    let shift_tape = formula_tape.shift_rule_tape().unwrap();
    assert_eq!(format!("{shift_tape}"), "11<A0101011");
    let shift_rule = formula_tape.detect_shift_rule();
}
