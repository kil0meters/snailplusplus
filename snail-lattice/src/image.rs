use crate::{
    direction::Direction,
    utils::{Vec2i},
};

pub struct Image<'a> {
    pub buffer: &'a mut [u8],
    pub width: usize,
    pub height: usize,
}

impl<'a> Image<'a> {
    #[inline(always)]
    pub fn draw_pixel(&mut self, index: usize, pixel: [u8; 3]) {
        self.buffer[index] = pixel[0];
        self.buffer[index + 1] = pixel[1];
        self.buffer[index + 2] = pixel[2];
        self.buffer[index + 3] = 0xFF;
    }

    #[inline(always)]
    pub fn draw_pixel_xy(&mut self, pixel: [u8; 3], x: usize, y: usize) {
        self.draw_pixel(4 * (y * self.width + x), pixel);
    }

    pub fn draw_circle_with(
        &mut self,
        mut color: impl FnMut() -> [u8; 3],
        x: usize,
        y: usize,
        radius: i32,
    ) {
        for dx in (-radius)..radius {
            for dy in (-radius)..radius {
                if (dx * dx) + (dy * dy) < radius * radius {
                    let draw_x = x as i32 + dx;
                    let draw_y = y as i32 + dy;

                    if draw_x > 0
                        && draw_x < self.width as i32
                        && draw_y > 0
                        && draw_y < self.height as i32
                    {
                        self.draw_pixel_xy(color(), draw_x as usize, draw_y as usize);
                    }
                }
            }
        }
    }

    pub fn draw_circle(&mut self, color: [u8; 3], x: usize, y: usize, radius: i32) {
        for dx in (-radius)..radius {
            for dy in (-radius)..radius {
                if (dx * dx) + (dy * dy) < radius * radius {
                    let draw_x = x as i32 + dx;
                    let draw_y = y as i32 + dy;

                    if draw_x > 0
                        && draw_x < self.width as i32
                        && draw_y > 0
                        && draw_y < self.height as i32
                    {
                        self.draw_pixel_xy(color, draw_x as usize, draw_y as usize);
                    }
                }
            }
        }
    }

    pub fn draw_rectangle_with(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        mut color: impl FnMut() -> [u8; 3],
        bx: usize,
        by: usize,
    ) {
        let px = 4 * ((y + by) * self.width + x + bx);

        for row in 0..h {
            for col in 0..w {
                self.draw_pixel(px + 4 * (row * self.width + col), color());
            }
        }
    }

    fn draw_char(&mut self, c: char, x: usize, y: usize) {
        let character_buffer = match c {
            'a' => include_bytes!("../../assets/bitmap_font/a.bin"),
            'b' => include_bytes!("../../assets/bitmap_font/b.bin"),
            'c' => include_bytes!("../../assets/bitmap_font/c.bin"),
            'd' => include_bytes!("../../assets/bitmap_font/d.bin"),
            'e' => include_bytes!("../../assets/bitmap_font/e.bin"),
            'f' => include_bytes!("../../assets/bitmap_font/f.bin"),
            'g' => include_bytes!("../../assets/bitmap_font/g.bin"),
            'h' => include_bytes!("../../assets/bitmap_font/h.bin"),
            'i' => include_bytes!("../../assets/bitmap_font/i.bin"),
            'j' => include_bytes!("../../assets/bitmap_font/j.bin"),
            'k' => include_bytes!("../../assets/bitmap_font/k.bin"),
            'l' => include_bytes!("../../assets/bitmap_font/l.bin"),
            'm' => include_bytes!("../../assets/bitmap_font/m.bin"),
            'n' => include_bytes!("../../assets/bitmap_font/n.bin"),
            'o' => include_bytes!("../../assets/bitmap_font/o.bin"),
            'p' => include_bytes!("../../assets/bitmap_font/p.bin"),
            'q' => include_bytes!("../../assets/bitmap_font/q.bin"),
            'r' => include_bytes!("../../assets/bitmap_font/r.bin"),
            's' => include_bytes!("../../assets/bitmap_font/s.bin"),
            't' => include_bytes!("../../assets/bitmap_font/t.bin"),
            'u' => include_bytes!("../../assets/bitmap_font/u.bin"),
            'v' => include_bytes!("../../assets/bitmap_font/v.bin"),
            'w' => include_bytes!("../../assets/bitmap_font/w.bin"),
            'x' => include_bytes!("../../assets/bitmap_font/x.bin"),
            'y' => include_bytes!("../../assets/bitmap_font/y.bin"),
            'z' => include_bytes!("../../assets/bitmap_font/z.bin"),
            '0' => include_bytes!("../../assets/bitmap_font/0.bin"),
            '1' => include_bytes!("../../assets/bitmap_font/1.bin"),
            '2' => include_bytes!("../../assets/bitmap_font/2.bin"),
            '3' => include_bytes!("../../assets/bitmap_font/3.bin"),
            '4' => include_bytes!("../../assets/bitmap_font/4.bin"),
            '5' => include_bytes!("../../assets/bitmap_font/5.bin"),
            '6' => include_bytes!("../../assets/bitmap_font/6.bin"),
            '7' => include_bytes!("../../assets/bitmap_font/7.bin"),
            '8' => include_bytes!("../../assets/bitmap_font/8.bin"),
            '9' => include_bytes!("../../assets/bitmap_font/9.bin"),
            ':' => include_bytes!("../../assets/bitmap_font/:.bin"),

            _ => unreachable!(),
        };

        let px = 4 * (y * self.width + x);
        for byte in character_buffer {
            let x = byte >> 6;
            let y = (byte >> 4) & 0b11;

            if x == 3 && y == 3 {
                break;
            }

            self.draw_pixel(
                px + 4 * (y as usize * self.width + x as usize),
                [0xFF, 0xFF, 0xFF],
            );

            let x = (byte >> 2) & 0b11;
            let y = byte & 0b11;

            if x == 3 && y == 3 {
                break;
            }

            self.draw_pixel(
                px + 4 * (y as usize * self.width + x as usize),
                [0xFF, 0xFF, 0xFF],
            );
        }
    }

    pub fn draw_text(&mut self, text: &str, mut x: usize, mut y: usize) {
        let line_start = x;

        for c in text.chars() {
            if c == '\n' {
                y += 5;
                x = line_start;

                continue;
            }

            if c != ' ' {
                self.draw_char(c, x, y);
            }
            x += 4;
        }
    }

    pub fn draw_line_high(&mut self, color: [u8; 3], x0: i32, y0: i32, x1: i32, y1: i32) {
        let mut dx = x1 - x0;
        let dy = y1 - y0;
        let mut xi = 1;
        if dx < 0 {
            xi = -1;
            dx = -dx;
        }

        let mut diff = (2 * dx) - dy;
        let mut x = x0;

        for y in y0..y1 {
            self.draw_pixel_xy(
                color,
                x.rem_euclid(self.width as i32) as usize,
                y.rem_euclid(self.height as i32) as usize,
            );
            if diff > 0 {
                x += xi;
                diff += 2 * (dx - dy);
            } else {
                diff += 2 * dx;
            }
        }
    }

    pub fn draw_line_low(&mut self, color: [u8; 3], x0: i32, y0: i32, x1: i32, y1: i32) {
        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }

        let mut diff = (2 * dy) - dx;
        let mut y = y0;

        for x in x0..x1 {
            self.draw_pixel_xy(
                color,
                x.rem_euclid(self.width as i32) as usize,
                y.rem_euclid(self.height as i32) as usize,
            );
            if diff > 0 {
                y += yi;
                diff += 2 * (dy - dx);
            } else {
                diff += 2 * dy;
            }
        }
    }

    pub fn draw_line(&mut self, color: [u8; 3], start: Vec2i, end: Vec2i) {
        let x0 = start.x;
        let x1 = end.x;
        let y0 = start.y;
        let y1 = end.y;

        if y1.abs_diff(y0) < x1.abs_diff(x0) {
            if x0 > x1 {
                self.draw_line_low(color, x1, y1, x0, y0);
            } else {
                self.draw_line_low(color, x0, y0, x1, y1);
            }
        } else {
            if y0 > y1 {
                self.draw_line_high(color, x1, y1, x0, y0);
            } else {
                self.draw_line_high(color, x0, y0, x1, y1);
            }
        }
    }

    pub fn draw_goal(&mut self, color: [u8; 3], dx: usize, dy: usize) {
        const GOAL_IMAGE_SIZE: usize = 7;

        let goal_image = include_bytes!("../../assets/goal_7x7.bin");

        for y in 0..GOAL_IMAGE_SIZE {
            for x in 0..GOAL_IMAGE_SIZE {
                let goal_px = y * GOAL_IMAGE_SIZE + x;
                let px = 4 * ((dy + x + 2) * self.width + dx + y + 2);

                // not transparent
                if goal_image[goal_px] != 255 {
                    self.buffer[px] = color[0];
                    self.buffer[px + 1] = color[1];
                    self.buffer[px + 2] = color[2];
                }
            }
        }
    }

    pub fn draw_snail(
        &mut self,
        palette: [[u8; 3]; 6],
        animation_cycle: bool,
        direction: Direction,
        dx: usize,
        dy: usize,
    ) {
        let snail_image = if animation_cycle {
            include_bytes!("../../assets/snail1_8x8.bin")
        } else {
            include_bytes!("../../assets/snail2_8x8.bin")
        };

        const SNAIL_IMAGE_SIZE: usize = 8;

        // draw goal
        for y in 0..SNAIL_IMAGE_SIZE {
            for x in 0..SNAIL_IMAGE_SIZE {
                let snail_px = y * SNAIL_IMAGE_SIZE + x;
                // only draw if not transparent
                if snail_image[snail_px] != 255 {
                    // I'm so, so, sorry.
                    let px = match direction {
                        Direction::Up => {
                            4 * ((dy + (SNAIL_IMAGE_SIZE - y)) * self.width + x + dx + 2)
                        }
                        Direction::Down => {
                            4 * ((y + 2 + dy) * self.width + dx + SNAIL_IMAGE_SIZE - x)
                        }
                        Direction::Left => {
                            4 * ((dy + x + 2) * self.width + dx + SNAIL_IMAGE_SIZE - y)
                        }
                        Direction::Right => 4 * ((dy + x + 2) * self.width + dx + y + 2),
                    };

                    let col = palette[snail_image[snail_px] as usize];

                    self.buffer[px] = col[0];
                    self.buffer[px + 1] = col[1];
                    self.buffer[px + 2] = col[2];
                }
            }
        }
    }
}
