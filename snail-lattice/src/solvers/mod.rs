use crate::{lfsr::LFSR, maze::Maze};

mod random_walk;
mod hold_left;

pub use random_walk::RandomWalk;
pub use hold_left::HoldLeft;

pub trait Solver {
    fn draw(&self, animation_cycle: bool, movement_timer: usize, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize);

    // returns true if the step solved the maze
    fn step(&mut self, maze: &Maze, lfsr: &mut LFSR) -> bool;
}
