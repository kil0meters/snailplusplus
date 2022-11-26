use crate::{
    direction::Direction,
    maze::Maze,
    solvers::Solver,
    snail::Snail, lfsr::LFSR
};

pub struct RandomWalk {
    snail: Snail,
}

impl RandomWalk {
    pub fn new(_upgrades: usize) -> Self {
        RandomWalk {
            snail: Snail::new()
        }
    }
}

impl Solver for RandomWalk {
    fn draw(&self, animation_cycle: bool, movement_timer: usize, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        self.snail.draw(animation_cycle, movement_timer, buffer, buffer_width, bx, by);
    }

    fn step(&mut self, maze: &Maze, lfsr: &mut LFSR) -> bool {
        loop {
            match lfsr.next() {
                0 => self.snail.direction = Direction::Up,
                1 => self.snail.direction = Direction::Down,
                2 => self.snail.direction = Direction::Left,
                3 => self.snail.direction = Direction::Right,
                _ => unreachable!(),
            }

            if self.snail.move_forward(maze) {
                break;
            }
        }

        if self.snail.pos == maze.end_pos {
            self.snail.reset();
            true
        } else {
            false
        }
    }
}
