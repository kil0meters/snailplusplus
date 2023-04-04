use crate::{image::Image, lfsr::LFSR, maze::Maze, snail::DEFAULT_PALETTE};

mod automaton;
mod clones;
mod demolitionist;
mod flying;
mod hold_left;
mod inverted;
mod learning;
mod random_teleport;
mod random_walk;
mod rpg;
mod telepathic;
mod time_travel;
mod tremaux;

pub use automaton::Automaton;
pub use clones::Clones;
pub use demolitionist::Demolitionist;
pub use flying::Flying;
pub use hold_left::HoldLeft;
pub use inverted::Inverted;
pub use learning::Learning;
pub use random_teleport::RandomTeleport;
pub use random_walk::RandomWalk;
pub use rpg::Rpg;
pub use telepathic::Telepathic;
pub use time_travel::TimeTravel;
pub use tremaux::Tremaux;

#[derive(Clone, Copy)]
pub enum SolveStatus {
    Solved(usize),
    Rerender,
    None,
}

impl SolveStatus {
    pub fn get_count(self) -> usize {
        match self {
            SolveStatus::Solved(count) => count,
            SolveStatus::Rerender => 0,
            SolveStatus::None => 0,
        }
    }
}

pub trait Solver {
    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        maze: &Maze,
        lfsr: &mut LFSR,
        image: &mut Image,
    );

    fn set_upgrades(&mut self, upgrades: u32);

    // run upon maze generation
    fn setup(&mut self, _maze: &Maze, _lfsr: &mut LFSR) {}

    // returns true if the step solved the maze
    // run at a fixed step rate based on movement_time
    fn step(&mut self, maze: &mut Maze, lfsr: &mut LFSR) -> SolveStatus;

    fn movement_time(&self) -> f32;

    fn custom_goal(&self) -> bool {
        false
    }

    fn palette(&self) -> [[u8; 3]; 6] {
        DEFAULT_PALETTE
    }
}
