mod directional_tm;
use directional_tm as dtm;

fn main() {
    let machine_str = "1RB1LE_1LC1RD_1LB1RC_1LA0RD_---0LA";
    let mut tape = dtm::Tape::new_partial(
        machine_str,
        &vec![
            dtm::TapeContent::Symbol(1),
            dtm::TapeContent::Symbol(0),
            dtm::TapeContent::Symbol(0),
            dtm::TapeContent::Symbol(1),
        ],
        dtm::TapeHead {
            state: 0,
            pointing_direction: dtm::Direction::RIGHT,
        },
        vec![dtm::TapeContent::Symbol(1), dtm::TapeContent::Symbol(1)],
    );
}
