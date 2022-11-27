use crate::{
    direction::Direction,
    maze::Maze,
    solvers::Solver,
    snail::Snail, lfsr::LFSR
};

pub struct Clones {
    snails: Vec<Snail>,
}

impl Clones {
    pub fn new(_upgrades: usize) -> Self {
        Clones {
            snails: vec![Snail::new()]
        }
    }

    fn reset(&mut self) {
        self.snails = vec![Snail::new()];
    }
}

impl Solver for Clones {
    fn draw(&self, animation_cycle: bool, movement_timer: usize, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        for snail in self.snails.iter() {
            snail.draw(animation_cycle, movement_timer, buffer, buffer_width, bx, by);
        }
    }

    fn step(&mut self, maze: &Maze, _lfsr: &mut LFSR) -> bool {
        let mut new_snails = Vec::new();

        for snail in self.snails.iter_mut() {
            if !snail.active {
                continue;
            }

            let coord = 4 * (snail.pos.y * maze.width + snail.pos.x);
            let left = snail.direction.rotate_counter();
            let right = snail.direction.rotate();
            // let mut current_snails = vec![snail];

            // if there's an option to the left, we create a new snail facing that direction and
            // move that direction
            if !maze.walls[coord + left as usize] {
                let mut new_snail = snail.clone();
                new_snail.direction = left;
                new_snails.push(new_snail);
                // current_snails.push(self.snails.last_mut().unwrap());
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
            }

            if snail.pos == maze.end_pos {
                self.reset();
                return true;
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

            self.snails.push(snail);
        }

        false
    }
}
