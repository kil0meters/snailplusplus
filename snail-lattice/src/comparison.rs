#![feature(generic_const_exprs)]

use snail_lattice::lattice::{MetaMaze, SnailLattice, TilableMaze};
use snail_lattice::maze::AutoMaze;
use snail_lattice::solvers::{
    Clones, Demolitionist, Flying, HoldLeft, Inverted, Learning, RandomTeleport, RandomWalk, Rpg,
    Telepathic, TimeTravel, Tremaux,
};

const MAZE_COUNT: i32 = 20;
const SECONDS: f64 = 10_000.0;
const TICK_AMOUNT: f32 = SECONDS as f32 * 1000.0;

fn run_bench<T: TilableMaze>(name: &str, multiplier: f64, upgrades: u32) {
    let mut lattice = SnailLattice::<T>::new(5, 0xDEAD);
    lattice.alter(MAZE_COUNT);
    lattice.set_upgrades(upgrades);

    let solves = lattice.tick(TICK_AMOUNT);
    println!(
        "{name} solved an average of {} mazes over {SECONDS} seconds",
        solves as f64 / MAZE_COUNT as f64
    );

    let fragments = (solves as f64 * multiplier).floor() as usize;
    println!(
        "{name} generated an average of {:.2} fragments/second",
        (fragments as f64 / SECONDS as f64) / MAZE_COUNT as f64
    );
    println!("---");
}

fn main() {
    // run_bench::<AutoMaze<5, RandomWalk<5>>>("random-walk", 100.0);
    // run_bench::<AutoMaze<7, RandomTeleport<7>>>("random-teleport", 150.);
    // run_bench::<AutoMaze<9, Learning<9>>>("learning", 500.);
    // run_bench::<AutoMaze<9, HoldLeft<9>>>("hold-left", 500.);
    // run_bench::<AutoMaze<9, Inverted<9>>>("inverted", 2400.);
    // run_bench::<AutoMaze<11, Tremaux<11>>>("tremaux", 10000.);
    // run_bench::<AutoMaze<11, Rpg<11>>>("rpg", 100000.);
    // run_bench::<AutoMaze<13, TimeTravel<13>>>("time-travel", 150000.);
    // run_bench::<AutoMaze<20, Clones<20>>>("clone", 400000.);
    // run_bench::<MetaMaze>("Meta", 14_000. * 49.);
    run_bench::<AutoMaze<15, Demolitionist<15>>>("Demolitionist", 25_000_000., 0b000);
    run_bench::<AutoMaze<15, Flying<15>>>("Flying", 3_000_000., 0b000);

    run_bench::<AutoMaze<10, Telepathic<10>>>("Telepathic", 280_000_000., 0b000);
    run_bench::<AutoMaze<10, Telepathic<10>>>("Telepathic", 280_000_000., 0b001);
    run_bench::<AutoMaze<10, Telepathic<10>>>("Telepathic", 280_000_000., 0b011);
    run_bench::<AutoMaze<10, Telepathic<10>>>("Telepathic", 280_000_000., 0b111);
}
