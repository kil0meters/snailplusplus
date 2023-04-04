use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{Snail, INVERTED_PALETTE},
    solvers::Solver,
};

use super::{HoldLeft, SolveStatus};

/// Hold Right Snail Upgrades:
/// - Right Glove:         With a glove on its right hand, Hold Right Snail is able to move 10% faster.
/// - Left Handed Snail:   Right Handed Snail Enlists the help of Right Handed Snail to solve mazes faster.

pub struct Inverted {
    snail: Snail,
    alt_snail: Option<Box<HoldLeft>>,
    upgrades: u32,
}

impl Inverted {
    pub fn new() -> Self {
        Inverted {
            snail: Snail::new(),
            alt_snail: None,
            upgrades: 0,
        }
    }
}

impl Solver for Inverted {
    fn set_upgrades(&mut self, upgrades: u32) {
        if (upgrades & 0b10) != 0 {
            let mut alt_snail = Box::new(HoldLeft::new());
            alt_snail.set_upgrades(upgrades & 0b1);
            self.alt_snail = Some(alt_snail);
        } else {
            self.alt_snail = None;
        }

        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        maze: &Maze,
        lfsr: &mut LFSR,
        image: &mut Image,
    ) {
        if let Some(left_handed) = &mut self.alt_snail {
            left_handed.draw(animation_cycle, movement_timer, maze, lfsr, image);
        }

        self.snail.draw(
            INVERTED_PALETTE,
            animation_cycle,
            movement_timer / self.movement_time(),
            image,
        );
    }

    fn setup(&mut self, _maze: &Maze, _lfsr: &mut LFSR) {
        self.snail.reset();
        if let Some(left_handed) = &mut self.alt_snail {
            left_handed.setup(_maze, _lfsr);
        }
    }

    fn step(&mut self, maze: &mut Maze, lfsr: &mut LFSR) -> SolveStatus {
        if let Some(left_handed) = &mut self.alt_snail {
            match left_handed.step(maze, lfsr) {
                SolveStatus::Solved(count) => return SolveStatus::Solved(count),
                _ => {}
            }
        }

        let cell = maze.get_cell(self.snail.pos.x, self.snail.pos.y);
        let right = self.snail.direction.rotate();

        // if we can move right, do so
        if !cell.has_wall(right) {
            self.snail.direction = right;
        }
        // otherwise, if there's a wall blocking the front, rotate counterclockwise until we face an empty
        // wall
        else {
            while cell.has_wall(self.snail.direction) {
                self.snail.direction = self.snail.direction.rotate_counter();
            }
        }

        self.snail.move_forward(maze);

        if self.snail.pos == maze.end_pos {
            SolveStatus::Solved(1)
        } else {
            SolveStatus::None
        }
    }

    fn movement_time(&self) -> f32 {
        // right glove
        if (self.upgrades & 0b1) != 0 {
            SNAIL_MOVEMENT_TIME * 0.8
        } else {
            SNAIL_MOVEMENT_TIME
        }
    }

    fn palette(&self) -> [[u8; 3]; 6] {
        INVERTED_PALETTE
    }
}
