use crate::{
    direction::Direction,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::Snail,
    solvers::Solver,
};

pub struct RandomWalk {
    snail: Snail,
}

impl RandomWalk {
    // RANDOM WALK UPGRADES
    // 1: Don't randomly walk into walls
    // 2: Straight along corridors
    // 3:
    pub fn new(_upgrades: usize) -> Self {
        RandomWalk {
            snail: Snail::new(),
        }
    }
}

impl Solver for RandomWalk {
    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        _maze: &Maze,
        _lfsr: &mut LFSR,
        buffer: &mut [u8],
        buffer_width: usize,
        bx: usize,
        by: usize,
    ) {
        self.snail.draw(
            animation_cycle,
            movement_timer,
            self.movement_time(),
            buffer,
            buffer_width,
            bx,
            by,
        );
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

    fn movement_time(&self) -> usize {
        SNAIL_MOVEMENT_TIME
    }
}
