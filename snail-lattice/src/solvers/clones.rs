

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::Snail,
    solvers::Solver,
};

pub struct Clones<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    active_snails: Vec<Snail<S>>,
    inactive_snails: Vec<Snail<S>>,
}

impl<const S: usize> Clones<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn reset(&mut self) {
        self.active_snails = vec![Snail::new()];
        self.inactive_snails = vec![]
    }
}

impl<const S: usize> Solver<S> for Clones<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        Clones {
            active_snails: vec![Snail::new()],
            inactive_snails: vec![],
        }
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        _lfsr: &mut LFSR,
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        for snail in self.inactive_snails.iter() {
            snail.draw(
                animation_cycle,
                movement_timer,
                self.movement_time(),
                image,
                bx,
                by,
            );
        }

        for snail in self.active_snails.iter() {
            snail.draw(
                animation_cycle,
                movement_timer,
                self.movement_time(),
                image,
                bx,
                by,
            );
        }
    }

    fn step(&mut self, maze: &Maze<S>, _lfsr: &mut LFSR) -> bool {
        let mut new_snails = Vec::new();

        let mut i = 0;
        while i < self.active_snails.len() {
            let snail = &mut self.active_snails[i];

            let cell = maze.get_cell(snail.pos.x, snail.pos.y);
            let left = snail.direction.rotate_counter();
            let right = snail.direction.rotate();

            // if there's an option to the left, we create a new snail facing that direction and
            // move that direction
            if !cell.has_wall(left) {
                let mut new_snail = snail.clone();
                new_snail.direction = left;
                new_snails.push(new_snail);
            }

            // same for right
            if !cell.has_wall(right) {
                let mut new_snail = snail.clone();
                new_snail.direction = right;
                new_snails.push(new_snail);
            }

            // if we can't move forward,
            if !snail.move_forward(maze) {
                snail.active = false;

                let owned = self.active_snails.remove(i);
                self.inactive_snails.push(owned);
            } else {
                if snail.pos == maze.end_pos {
                    self.reset();
                    return true;
                }

                i += 1;
            }
        }

        while let Some(mut snail) = new_snails.pop() {
            if !snail.move_forward(maze) {
                snail.active = false;
            }

            if snail.pos == maze.end_pos {
                self.reset();
                return true;
            }

            self.active_snails.push(snail);
        }

        false
    }

    fn movement_time(&self) -> usize {
        SNAIL_MOVEMENT_TIME
    }
}
