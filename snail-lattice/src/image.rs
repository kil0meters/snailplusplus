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

    pub fn draw_rectangle_with<F>(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        mut color: F,
        bx: usize,
        by: usize,
    ) where
        F: FnMut() -> [u8; 3],
    {
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
}
