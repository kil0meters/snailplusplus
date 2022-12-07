use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX},
};

mod clones;
mod hold_left;
mod random_teleport;
mod random_walk;
mod time_travel;
mod tremaux;

pub use clones::Clones;
pub use hold_left::HoldLeft;
pub use random_teleport::RandomTeleport;
pub use random_walk::RandomWalk;
pub use time_travel::TimeTravel;
pub use tremaux::Tremaux;

pub trait Solver<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized
{
    fn new() -> Self;

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        lfsr: &mut LFSR,

        image: &mut Image,
        bx: usize,
        by: usize,
    );

    // returns true if the step solved the maze
    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool;

    fn movement_time(&self) -> usize;
}
