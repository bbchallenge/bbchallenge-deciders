#![allow(dead_code)]
#![allow(non_snake_case)]

use criterion::{criterion_group, criterion_main, Criterion};
use decider_halting_segment_reproduction::{
    tm::TM, Iijil_strategy, Iijil_strategy_updated, PATH_TO_BBCHALLENGE_DB_TEST,
};
use std::time::Duration;

const WARM_UP_TIME_MS: u64 = 500;
const MEASUREMENT_TIME_MS: u64 = 2000;

fn benchmark_Iijil_strategy(c: &mut Criterion) {
    // let tm = TM::from_bbchallenge_id(23367211, PATH_TO_BBCHALLENGE_DB_TEST);

    let mut group = c.benchmark_group("Bench Tape Type");

    group.warm_up_time(Duration::from_millis(WARM_UP_TIME_MS));
    group.measurement_time(Duration::from_millis(MEASUREMENT_TIME_MS));
    // group.sample_size(50);

    group.bench_function("Iijil_strategy_23367211", |b| {
        b.iter(|| Iijil_strategy_23367211())
    });

    group.bench_function("Iijil_strategy_updated_23367211", |b| {
        b.iter(|| Iijil_strategy_updated_23367211())
    });

    group.finish();
}

fn Iijil_strategy_23367211() {
    let turing_machine = TM::from_bbchallenge_id(23367211, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();
    assert!(Iijil_strategy(turing_machine, 200000, false))
}

fn Iijil_strategy_updated_23367211() {
    // Segment size 15 = 2*7+1
    let turing_machine = TM::from_bbchallenge_id(23367211, PATH_TO_BBCHALLENGE_DB_TEST).unwrap();
    assert!(Iijil_strategy_updated(turing_machine, 7, false))
}

criterion_group!(benches, benchmark_Iijil_strategy,);
criterion_main!(benches);
