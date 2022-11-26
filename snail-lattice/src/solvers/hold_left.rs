use crate::{
    maze::Maze,
    solvers::Solver,
    snail::Snail, lfsr::LFSR
};

pub struct HoldLeft {
    snail: Snail,
}

impl HoldLeft {
    pub fn new(_upgrades: usize) -> Self {
        HoldLeft {
            snail: Snail::new()
        }
    }
}

impl Solver for HoldLeft {
    fn draw(&self, animation_cycle: bool, movement_timer: usize, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        self.snail.draw(animation_cycle, movement_timer, buffer, buffer_width, bx, by);
    }

    fn step(&mut self, maze: &Maze, _lfsr: &mut LFSR) -> bool {
        let coord = 4 * (self.snail.pos.y * maze.width + self.snail.pos.x);
        let left = self.snail.direction.rotate_counter();

        // if we can move left, do so
        if !maze.walls[coord + left as usize] {
            self.snail.direction = left;
        }

        // otherwise, if there's a wall blocking the front, rotate clockwise until we face an empty
        // wall
        else {
            while maze.walls[coord + self.snail.direction as usize] {
                self.snail.direction = self.snail.direction.rotate();
            }
        }

        self.snail.move_forward(maze);

        if self.snail.pos == maze.end_pos {
            self.snail.reset();
            true
        } else {
            false
        }
    }
}
