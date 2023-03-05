use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE},
    solvers::Solver,
};

// Cloning Snail:
// - Self-Improvement:  Each Cloning Snail moves slightly faster than the last.
// - Snail Singularity: Each Cloning Snail moves even faster than the last.

pub struct Clones<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    active_snails: Vec<Snail<S>>,
    inactive_snails: Vec<Snail<S>>,
    move_count: usize,
    upgrades: u32,
}

impl<const S: usize> Solver<S> for Clones<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        Clones {
            active_snails: vec![Snail::new()],
            inactive_snails: vec![],
            move_count: 0,
            upgrades: 0,
        }
    }

    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
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
                GRAYSCALE_PALETTE,
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
                DEFAULT_PALETTE,
                animation_cycle,
                movement_timer,
                self.movement_time(),
                image,
                bx,
                by,
            );
        }
    }

    fn setup(&mut self, _maze: &Maze<S>, _lfsr: &mut LFSR) {
        self.move_count = 0;
        self.active_snails = vec![Snail::new()];
        self.inactive_snails = vec![]
    }

    fn step(&mut self, maze: &Maze<S>, _lfsr: &mut LFSR) -> bool {
        self.move_count += 1;
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

            // if we can't move forward, second condition is an upgrade
            if !snail.move_forward(maze) {
                snail.active = false;

                let owned = self.active_snails.remove(i);
                self.inactive_snails.push(owned);
            } else {
                if snail.pos == maze.end_pos {
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
                return true;
            }

            self.active_snails.push(snail);
        }

        false
    }

    fn movement_time(&self) -> usize {
        let mut movement_time = SNAIL_MOVEMENT_TIME;

        // self-improvement
        if (self.upgrades & 0b1) != 0 {
            movement_time -= 1_000 * self.move_count;
        }

        // singularity
        if (self.upgrades & 0b10) != 0 && self.move_count != 0 {
            movement_time /= self.move_count;
        }

        movement_time.max(10_000)
    }
}
