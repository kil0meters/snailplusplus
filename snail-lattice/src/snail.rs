use crate::{
    image::Image,
    maze::CELLS_PER_IDX,
    utils::{discrete_lerp, Vec2},
};

use super::{direction::Direction, maze::Maze};

#[derive(Clone)]
pub struct Snail<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub direction: Direction,
    pub active: bool,
}

impl<const S: usize> Snail<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    pub fn new() -> Snail<S> {
        Snail {
            pos: Vec2 { x: 0, y: 0 },
            prev_pos: Vec2 { x: 0, y: 0 },

            direction: Direction::Right,
            active: true,
        }
    }

    pub fn draw(
        &self,
        animation_cycle: bool,
        movement_timer: usize,
        movement_time: usize,

        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        let snail_image = if self.active {
            if animation_cycle {
                include_bytes!("../../assets/snail1_8x8.bin")
            } else {
                include_bytes!("../../assets/snail2_8x8.bin")
            }
        } else {
            include_bytes!("../../assets/snail1_grayscale_8x8.bin")
        };

        let offset_y = if self.prev_pos.y != self.pos.y {
            discrete_lerp(
                (self.prev_pos.y * 10) as i32,
                (self.pos.y * 10) as i32,
                movement_timer as i32,
                movement_time as i32,
            )
        } else {
            (self.pos.y * 10) as i32
        };

        let offset_x = if self.prev_pos.x != self.pos.x {
            discrete_lerp(
                (self.prev_pos.x * 10) as i32,
                (self.pos.x * 10) as i32,
                movement_timer as i32,
                movement_time as i32,
            )
        } else {
            (self.pos.x * 10) as i32
        };

        const SNAIL_IMAGE_SIZE: usize = 8;

        // draw goal
        for y in 0..SNAIL_IMAGE_SIZE {
            for x in 0..SNAIL_IMAGE_SIZE {
                let snail_px = 4 * (y * SNAIL_IMAGE_SIZE + x);
                // only draw if not transparent
                if snail_image[snail_px + 3] != 0 {
                    // I'm so, so, sorry.
                    let px = match self.direction {
                        Direction::Up => {
                            4 * (((by + (SNAIL_IMAGE_SIZE - y)) as i32 + offset_y) as usize
                                * image.buffer_width
                                + bx
                                + x
                                + offset_x as usize
                                + 2)
                        }
                        Direction::Down => {
                            4 * (((by + y + 2) as i32 + offset_y) as usize * image.buffer_width
                                + bx
                                + (SNAIL_IMAGE_SIZE - x)
                                + offset_x as usize)
                        }
                        Direction::Left => {
                            4 * ((by + x + offset_y as usize + 2) * image.buffer_width
                                + ((bx + (SNAIL_IMAGE_SIZE - y)) as i32 + offset_x) as usize)
                        }
                        Direction::Right => {
                            4 * ((by + x + offset_y as usize + 2) * image.buffer_width
                                + ((bx + y + 2) as i32 + offset_x) as usize)
                        }
                    };

                    image.buffer[px] = snail_image[snail_px];
                    image.buffer[px + 1] = snail_image[snail_px + 1];
                    image.buffer[px + 2] = snail_image[snail_px + 2];
                }
            }
        }
    }

    pub fn move_forward(&mut self, maze: &Maze<S>) -> bool {
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
