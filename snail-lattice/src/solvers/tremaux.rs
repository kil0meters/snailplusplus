use std::collections::HashMap;

use crate::{lfsr::LFSR, maze::Maze, snail::Snail, solvers::Solver};

pub struct Tremaux {
    snail: Snail,
    visited: HashMap<usize, usize>,
}

impl Tremaux {
    pub fn new(_upgrades: usize) -> Self {
        Tremaux {
            snail: Snail::new(),
            visited: HashMap::new(),
        }
    }
}

impl Solver for Tremaux {
    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        buffer: &mut [u8],
        buffer_width: usize,
        bx: usize,
        by: usize,
    ) {
        self.snail.draw(
            animation_cycle,
            movement_timer,
            buffer,
            buffer_width,
            bx,
            by,
        );
    }

    fn step(&mut self, maze: &Maze, lfsr: &mut LFSR) -> bool {
        let coord = 4 * (self.snail.pos.y * maze.width + self.snail.pos.x);
        let direction_count = maze.walls[coord] as usize
            + maze.walls[coord + 1] as usize
            + maze.walls[coord + 2] as usize
            + maze.walls[coord + 3] as usize;

        // if in junction
        if direction_count > 2 {
        } else {
        }

        if self.snail.pos == maze.end_pos {
            self.snail.reset();
            true
        } else {
            false
        }
    }
}
