#![feature(generic_const_exprs)]

use snail_lattice::lattice::{MetaMaze, SnailLattice, TilableMaze};
use snail_lattice::maze::AutoMaze;
use snail_lattice::solvers::{
    Clones, HoldLeft, Inverted, Learning, RandomTeleport, RandomWalk, Rpg, TimeTravel, Tremaux,
};

const MICROSECONDS_PER_SECOND: usize = 1_000_000;
const SECONDS: usize = 3600;

const TICK_AMOUNT: usize = SECONDS * MICROSECONDS_PER_SECOND;
//
fn run_bench<T: TilableMaze>(name: &str, multiplier: f64) {
    let mut lattice = SnailLattice::<T>::new(5, 0xDEAD);
    lattice.alter(10);

    let fragments = (lattice.tick(TICK_AMOUNT) as f64 * multiplier).floor() as usize;

    println!(
        "{name} generated an average of {:.2} fragments/second",
        (fragments as f64 / (SECONDS as f64 * 100 as f64)) / 10.0
    );
}

fn main() {
    run_bench::<AutoMaze<5, RandomWalk<5>>>("random-walk", 100.0);
    run_bench::<AutoMaze<7, RandomTeleport<7>>>("random-teleport", 150.);
    run_bench::<AutoMaze<9, Learning<9>>>("learning", 500.);
    run_bench::<AutoMaze<9, HoldLeft<9>>>("hold-left", 500.);
    run_bench::<AutoMaze<9, Inverted<9>>>("inverted", 2400.);
    run_bench::<AutoMaze<11, Tremaux<11>>>("tremaux", 10000.);
    run_bench::<AutoMaze<11, Rpg<11>>>("rpg", 100000.);
    run_bench::<AutoMaze<13, TimeTravel<13>>>("time-travel", 150000.);
    run_bench::<AutoMaze<20, Clones<20>>>("clone", 400000.);
    run_bench::<MetaMaze>("meta", 1400000.);
}
