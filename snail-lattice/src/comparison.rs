#![feature(generic_const_exprs)]

use snail_lattice::lattice::SnailLattice;
use snail_lattice::maze::CELLS_PER_IDX;
use snail_lattice::solvers::{
    Clones, HoldLeft, RandomTeleport, RandomWalk, Solver, TimeTravel, Tremaux,
};

const MICROSECONDS_PER_SECOND: usize = 1_000_000;
const SECONDS: usize = 3600;

const TICK_AMOUNT: usize = SECONDS * MICROSECONDS_PER_SECOND;

fn run_bench<const S: usize, T: Solver<S>>(name: &str, multiplier: f64)
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    let mut lattice = SnailLattice::<S, T>::new(5, 0xDEAD);
    lattice.alter(100);

    let fragments = (lattice.tick(TICK_AMOUNT) as f64 * multiplier).floor() as usize;

    println!(
        "{name} generated an average of {:.2} fragments/second",
        fragments as f64 / (SECONDS as f64 * 100 as f64)
    );
}

fn main() {
    run_bench::<5, RandomWalk<5>>("random-walk", 1.0);
    run_bench::<7, RandomTeleport<7>>("random-teleport", 1.5);
    run_bench::<9, HoldLeft<9>>("hold-left", 1.0);
    run_bench::<11, Tremaux<11>>("tremaux", 5.0);
    run_bench::<13, TimeTravel<13>>("time-travel", 9.0);
    run_bench::<20, Clones<20>>("clone", 20.0);
}
