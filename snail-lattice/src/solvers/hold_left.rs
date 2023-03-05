use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
};

use super::Inverted;

/// Hold Left Snail Upgrades:
/// - Left Glove:         With a glove on its left hand, Hold Left Snail is able to move 20% faster.
/// - Right Handed Snail: Left Handed Snail Enlists the help of Right Handed Snail to solve mazes faster.

pub struct HoldLeft<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    snail: Snail<S>,
    alt_snail: Option<Box<Inverted<S>>>,
    upgrades: u32,
}

impl<const S: usize> Solver<S> for HoldLeft<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        HoldLeft {
            snail: Snail::new(),
            alt_snail: None,
            upgrades: 0,
        }
    }

    fn set_upgrades(&mut self, upgrades: u32) {
        if (upgrades & 0b10) != 0 {
            let mut alt_snail = Box::new(Inverted::new());
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
        movement_timer: usize,
        lfsr: &mut LFSR,
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        if let Some(right_handed) = &mut self.alt_snail {
            right_handed.draw(animation_cycle, movement_timer, lfsr, image, bx, by);
        }

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer,
            self.movement_time(),
            image,
            bx,
            by,
        );
    }

    fn setup(&mut self, _maze: &Maze<S>, _lfsr: &mut LFSR) {
        self.snail.reset();
        if let Some(right_handed) = &mut self.alt_snail {
            right_handed.setup(_maze, _lfsr);
        }
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
        if let Some(right_handed) = &mut self.alt_snail {
            if right_handed.step(maze, lfsr) {
                return true;
            }
        }

        let cell = maze.get_cell(self.snail.pos.x, self.snail.pos.y);
        let left = self.snail.direction.rotate_counter();

        // if we can move left, do so
        if !cell.has_wall(left) {
            self.snail.direction = left;
        }
        // otherwise, if there's a wall blocking the front, rotate clockwise until we face an empty
        // wall
        else {
            while cell.has_wall(self.snail.direction) {
                self.snail.direction = self.snail.direction.rotate();
            }
        }

        self.snail.move_forward(maze);

        self.snail.pos == maze.end_pos
    }

    fn movement_time(&self) -> usize {
        // left glove
        if (self.upgrades & 0b1) != 0 {
            SNAIL_MOVEMENT_TIME * 4 / 5
        } else {
            SNAIL_MOVEMENT_TIME
        }
    }
}
