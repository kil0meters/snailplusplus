use crate::lfsr::LFSR;

use super::SnailMaze;

impl SnailMaze {
    pub fn ai_hold_left(&mut self) {
        let coord = 4 * (self.snail_pos.y * self.width + self.snail_pos.x);
        let left = self.snail_direction.rotate_counter();

        // if we can move left, do so
        if !self.maze[coord + left as usize] {
            self.snail_direction = left;
        }

        // otherwise, if there's a wall blocking the front, rotate clockwise until we face an empty
        // wall
        else {
            while self.maze[coord + self.snail_direction as usize] {
                self.snail_direction = self.snail_direction.rotate();
            }
        }

        self.move_forward();
    }
}
