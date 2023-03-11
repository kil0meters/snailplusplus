use crate::{
    image::Image,
    maze::CELLS_PER_IDX,
    utils::{discrete_lerp, Vec2},
};

use super::{direction::Direction, maze::Maze};

pub const DEFAULT_PALETTE: [[u8; 3]; 6] = [
    [0xf8, 0xfc, 0x00], // yellow
    [0xa8, 0x54, 0x50], // purple
    [0xf8, 0x54, 0x00], // orange
    [0xff, 0xff, 0xff], // white
    [0x06, 0x8F, 0xEF], // light blue
    [0x11, 0x0A, 0xEF], // dark blue
];

pub const INVERTED_PALETTE: [[u8; 3]; 6] = [
    [0x07, 0x03, 0xff], // blue
    [0x57, 0xab, 0xaf], // cyan?
    [0x07, 0xab, 0xff], // light blue
    [0x00, 0x00, 0x00], // black
    [0xf9, 0x70, 0x10], // orange
    [0xee, 0xf5, 0x10], // yellow
];

pub const GRAYSCALE_PALETTE: [[u8; 3]; 6] = [
    [0xdf, 0xdf, 0xdf], // yellow
    [0x6c, 0x6c, 0x6c], // purple
    [0x7b, 0x7b, 0x7b], // orange
    [0xff, 0xff, 0xff], // white
    [0x70, 0x70, 0x70], // so far not relevant
    [0x25, 0x25, 0x25], // so far not relevant
];

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
        palette: [[u8; 3]; 6],

        animation_cycle: bool,
        movement_timer: usize,
        movement_time: usize,

        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        let snail_image = if animation_cycle || !self.active {
            include_bytes!("../../assets/snail1_8x8.bin")
        } else {
            include_bytes!("../../assets/snail2_8x8.bin")
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
                let snail_px = y * SNAIL_IMAGE_SIZE + x;
                // only draw if not transparent
                if snail_image[snail_px] != 255 {
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

                    let col = palette[snail_image[snail_px] as usize];

                    image.buffer[px] = col[0];
                    image.buffer[px + 1] = col[1];
                    image.buffer[px + 2] = col[2];
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
