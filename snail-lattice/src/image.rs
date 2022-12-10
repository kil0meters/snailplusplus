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
}
