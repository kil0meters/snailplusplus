use crate::{lfsr::LFSR, maze::Maze};

mod clones;
mod hold_left;
mod random_teleport;
mod random_walk;
mod tremaux;

pub use clones::Clones;
pub use hold_left::HoldLeft;
pub use random_teleport::RandomTeleport;
pub use random_walk::RandomWalk;
pub use tremaux::Tremaux;

pub trait Solver {
    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        buffer: &mut [u8],
        buffer_width: usize,
        bx: usize,
        by: usize,
    );

    // returns true if the step solved the maze
    fn step(&mut self, maze: &Maze, lfsr: &mut LFSR) -> bool;
}
