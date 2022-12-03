use crate::{lfsr::LFSR, maze::Maze, snail::Snail, solvers::Solver};

pub struct Clones {
    active_snails: Vec<Snail>,
    inactive_snails: Vec<Snail>,
}

impl Clones {
    pub fn new(_upgrades: usize) -> Self {
        Clones {
            active_snails: vec![Snail::new()],
            inactive_snails: vec![],
        }
    }

    fn reset(&mut self) {
        self.active_snails = vec![Snail::new()];
        self.inactive_snails = vec![]
    }
}

impl Solver for Clones {
    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        buffer: &mut [u8],
        buffer_width: usize,
        bx: usize,
        by: usize,
    ) {
        for snail in self.inactive_snails.iter() {
            snail.draw(
                animation_cycle,
                movement_timer,
                buffer,
                buffer_width,
                bx,
                by,
            );
        }

        for snail in self.active_snails.iter() {
            snail.draw(
                animation_cycle,
                movement_timer,
                buffer,
                buffer_width,
                bx,
                by,
            );
        }
    }

    fn step(&mut self, maze: &Maze, _lfsr: &mut LFSR) -> bool {
        let mut new_snails = Vec::new();

        let mut i = 0;
        while i < self.active_snails.len() {
            let snail = &mut self.active_snails[i];

            let coord = 4 * (snail.pos.y * maze.width + snail.pos.x);
            let left = snail.direction.rotate_counter();
            let right = snail.direction.rotate();

            // if there's an option to the left, we create a new snail facing that direction and
            // move that direction
            if !maze.walls[coord + left as usize] {
                let mut new_snail = snail.clone();
                new_snail.direction = left;
                new_snails.push(new_snail);
            }

            // same for right
            if !maze.walls[coord + right as usize] {
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
}
