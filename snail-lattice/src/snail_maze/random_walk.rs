use crate::lfsr::LFSR;

use super::{SnailMaze, Direction};

impl SnailMaze {
    pub fn ai_random_walk(&mut self, lfsr: &mut LFSR) {
        loop {
            match lfsr.next() {
                0 => self.snail_direction = Direction::Up,
                1 => self.snail_direction = Direction::Down,
                2 => self.snail_direction = Direction::Left,
                3 => self.snail_direction = Direction::Right,
                _ => unreachable!(),
            }

            if self.move_forward() {
                break;
            }
        }
    }
}

