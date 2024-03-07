use crate::{
    image::Image,
    utils::{lerpf, Vec2},
    SNAIL_RENDER_ID,
};

use super::{direction::Direction, maze::Maze};

#[derive(Clone)]
pub struct Snail {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub direction: Direction,
    pub active: bool,
}

impl Snail {
    pub fn new() -> Snail {
        Snail {
            pos: Vec2 { x: 0, y: 0 },
            prev_pos: Vec2 { x: 0, y: 0 },

            direction: Direction::Right,
            active: true,
        }
    }

    pub fn render(&self, progress: f32, render_list: &mut Vec<f32>) {
        let offset_y = if self.prev_pos.y != self.pos.y {
            lerpf(
                self.prev_pos.y as f32 * 10.0,
                self.pos.y as f32 * 10.0,
                progress,
            )
        } else {
            self.pos.y as f32 * 10.0
        };

        let offset_x = if self.prev_pos.x != self.pos.x {
            lerpf(
                self.prev_pos.x as f32 * 10.0,
                self.pos.x as f32 * 10.0,
                progress,
            )
        } else {
            self.pos.x as f32 * 10.0
        };

        // push to render list
        render_list.push(SNAIL_RENDER_ID);
        render_list.push(offset_x);
        render_list.push(offset_y);
    }

    pub fn move_forward(&mut self, maze: &Maze) -> bool {
        let cell = maze.get_cell(self.pos.x, self.pos.y);
        self.prev_pos = self.pos;

        if !cell.has_wall(self.direction) {
            match self.direction {
                Direction::Up => {
                    self.pos.y -= 1;
                }
                Direction::Down => {
                    self.pos.y += 1;
                }
                Direction::Left => {
                    self.pos.x -= 1;
                }
                Direction::Right => {
                    self.pos.x += 1;
                }
            }

            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.pos.x = 0;
        self.pos.y = 0;
        self.prev_pos = self.pos;
    }
}
