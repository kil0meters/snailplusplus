use crate::utils::Vec2;

pub struct Image<'a> {
    pub buffer: &'a mut [u8],
    pub buffer_width: usize,
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
        self.draw_pixel(4 * (y * self.buffer_width + x), pixel);
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
        let px = 4 * ((y + by) * self.buffer_width + x + bx);

        for row in 0..h {
            for col in 0..w {
                self.draw_pixel(px + 4 * (row * self.buffer_width + col), color());
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

        let px = 4 * (y * self.buffer_width + x);
        for byte in character_buffer {
            let x = byte >> 6;
            let y = (byte >> 4) & 0b11;

            if x == 3 && y == 3 {
                break;
            }

            self.draw_pixel(
                px + 4 * (y as usize * self.buffer_width + x as usize),
                [0xFF, 0xFF, 0xFF],
            );

            let x = (byte >> 2) & 0b11;
            let y = byte & 0b11;

            if x == 3 && y == 3 {
                break;
            }

            self.draw_pixel(
                px + 4 * (y as usize * self.buffer_width + x as usize),
                [0xFF, 0xFF, 0xFF],
            );
        }
    }

    pub fn draw_text(&mut self, text: &str, mut x: usize, y: usize) {
        for c in text.chars() {
            if c != ' ' {
                self.draw_char(c, x, y);
            }
            x += 4;
        }
    }

    pub fn draw_line(&mut self, color: [u8; 3], start: Vec2, end: Vec2) {
        let dy = start.y - end.y;
        let dx = start.x - end.x;
        for x in start.x..end.x {
            let y = start.y + dy * (x - start.x) / dx;
            self.draw_pixel_xy(color, x, y);
        }
    }

    pub fn draw_goal(&mut self, color: [u8; 3], pos: Vec2, bx: usize, by: usize) {
        const GOAL_IMAGE_SIZE: usize = 7;

        let goal_image = include_bytes!("../../assets/goal_7x7.bin");

        for y in 0..GOAL_IMAGE_SIZE {
            for x in 0..GOAL_IMAGE_SIZE {
                let goal_px = y * GOAL_IMAGE_SIZE + x;
                let px =
                    4 * ((by + x + pos.y * 10 + 2) * self.buffer_width + bx + y + pos.x * 10 + 2);

                // not transparent
                if goal_image[goal_px] != 255 {
                    self.buffer[px] = color[0];
                    self.buffer[px + 1] = color[1];
                    self.buffer[px + 2] = color[2];
                }
            }
        }
    }
}
