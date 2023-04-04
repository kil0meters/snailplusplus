use crate::{direction::Direction, image::Image, lfsr::LFSR, snail::DEFAULT_PALETTE, utils::Vec2i};

// [2, 4, 1, 3]
// based on https://codeincomplete.com/articles/javascript-tetris/
const BLOCKS: [[u16; 4]; 7] = [
    // [0x2222, 0x4444, 0x0F00, 0x00F0], // I
    [0x2222, 0x2222, 0x00F0, 0x00F0], // I
    [0x8E00, 0x0E20, 0x44C0, 0x6440], // J
    [0x0E80, 0x2E00, 0x4460, 0xC440], // L
    [0xCC00, 0xCC00, 0xCC00, 0xCC00], // O
    // [0x8C40, 0x4620, 0x06C0, 0x6C00], // S
    [0x8C40, 0x8C40, 0x06C0, 0x06C0], // S
    [0x4C40, 0x4640, 0x0E40, 0x4E00], // T
    // [0x4C80, 0x2640, 0x0C60, 0xC600], // Z
    [0x4C80, 0x4C80, 0x0C60, 0x0C60], // Z
];

const J_PALETTE: [[u8; 3]; 6] = [
    [0xff, 0x00, 0x00], // yellow
    [0xff, 0xff, 0xff], // purple
    [0xff, 0xff, 0xff], // orange
    [0xff, 0xff, 0xff], // white
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];

const L_PALETTE: [[u8; 3]; 6] = [
    [0xaa, 0x00, 0x55],
    [0xff, 0xaa, 0x00],
    [0xff, 0xff, 0x55],
    [0xff, 0xff, 0xff],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];

const O_PALETTE: [[u8; 3]; 6] = [
    [0x00, 0x55, 0xaa],
    [0x00, 0xaa, 0xaa],
    [0xff, 0xff, 0x55],
    [0xff, 0xff, 0xff],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];

const S_PALETTE: [[u8; 3]; 6] = [
    [0xaa, 0xaa, 0x00],
    [0xaa, 0xff, 0xaa],
    [0x00, 0xff, 0xaa],
    [0xff, 0xff, 0xff],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];

const T_PALETTE: [[u8; 3]; 6] = [
    [0xff, 0x55, 0xff],
    [0xff, 0xaa, 0xaa],
    [0xaa, 0x55, 0xaa],
    [0xff, 0xff, 0xff],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];

const Z_PALETTE: [[u8; 3]; 6] = [
    [0x55, 0x00, 0xff],
    [0x55, 0xff, 0xaa],
    [0xaa, 0x55, 0x55],
    [0xff, 0xff, 0xff],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];

const COLORS: [[[u8; 3]; 6]; 7] = [
    DEFAULT_PALETTE, // I
    J_PALETTE,       // J
    L_PALETTE,       // L
    O_PALETTE,       // O
    S_PALETTE,       // S
    T_PALETTE,       // T
    Z_PALETTE,       // Z
];

const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const FALL_TIME: f32 = 16.67 * 60.0;
const AUTO_SHIFT_DELAY: f32 = 16.67 * 24.0;
const PIECE_MOVE_DELAY: f32 = 16.67 * 2.0;

const LINE_SCORES: [i64; 4] = [40, 100, 300, 1200];

fn for_each_block(
    block_id: usize,
    x: i32,
    y: i32,
    dir: Direction,
    mut callbackfn: impl FnMut(i32, i32),
) {
    let mut row = 0;
    let mut col = 0;

    let block = BLOCKS[block_id][dir as usize];

    let mut bit = 0x8000;
    while bit > 0 {
        if block & bit != 0 {
            callbackfn(x + col, y + row);
        }

        col += 1;
        if col == 4 {
            col = 0;
            row += 1;
        }

        bit >>= 1;
    }
}

pub struct FallingSnailsGame {
    // 1 byte per cell
    // top 2 bits = direction
    // bottom 6 bits = color
    // e.g. 0bDDCCCCCC where D = direction and C = color
    grid: [u8; WIDTH * HEIGHT],

    selected_piece: usize,
    held_piece_pos: Vec2i,
    fall_timer: f32,
    time: f32,
    last_shift_update: f32,

    can_rotate: bool,
    right_held: f32,
    left_held: f32,
    held_piece_dir: Direction,
}

impl FallingSnailsGame {
    pub fn new(lfsr: &mut LFSR) -> FallingSnailsGame {
        FallingSnailsGame {
            grid: [0; _],
            held_piece_pos: Vec2i::new(4, 0),
            held_piece_dir: Direction::Right,
            selected_piece: lfsr.big() % BLOCKS.len(),
            can_rotate: true,
            right_held: -1000.0,
            left_held: -1000.0,
            last_shift_update: 0.0,
            fall_timer: 0.0,
            time: 0.0,
        }
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![200, 200]
    }

    pub fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: f32) -> i64 {
        self.time += dt;

        let mut keys_bits = 0;
        for key in keys {
            keys_bits |= key;
        }

        // DOWN
        let fall_time = if keys_bits & 4 != 0 {
            FALL_TIME / 20.0
        } else {
            FALL_TIME
        };

        self.fall_timer += dt;
        if self.fall_timer >= fall_time {
            self.fall_timer %= fall_time;

            if !self.held_piece_collides(
                self.held_piece_pos.x,
                self.held_piece_pos.y + 1,
                self.held_piece_dir,
            ) {
                self.held_piece_pos.y += 1;
            } else {
                self.reset_piece(lfsr);
            }
        }

        // up / rotate
        if keys_bits & 8 != 0 {
            if self.can_rotate {
                let dir = match self.held_piece_dir {
                    Direction::Up => Direction::Right,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Up,
                    Direction::Right => Direction::Down,
                };

                if !self.held_piece_collides(self.held_piece_pos.x, self.held_piece_pos.y, dir) {
                    self.held_piece_dir = dir;
                }

                self.can_rotate = false;
            }
        } else {
            self.can_rotate = true;
        }

        // move right if possible
        if keys_bits & 1 != 0 {
            if self.time - self.right_held > AUTO_SHIFT_DELAY
                && self.time - self.last_shift_update > PIECE_MOVE_DELAY
            {
                if !self.held_piece_collides(
                    self.held_piece_pos.x + 1,
                    self.held_piece_pos.y,
                    self.held_piece_dir,
                ) {
                    self.held_piece_pos.x += 1;
                    self.last_shift_update = self.time;
                }
            }

            if self.right_held < 0.0 {
                self.right_held = self.time;
            }
        } else {
            self.right_held = -1.0;
        }

        if keys_bits & 2 != 0 {
            if self.time - self.left_held > AUTO_SHIFT_DELAY
                && self.time - self.last_shift_update > PIECE_MOVE_DELAY
            {
                if !self.held_piece_collides(
                    self.held_piece_pos.x - 1,
                    self.held_piece_pos.y,
                    self.held_piece_dir,
                ) {
                    self.held_piece_pos.x -= 1;
                    self.last_shift_update = self.time;
                }
            }

            if self.left_held < 0.0 {
                self.left_held = self.time;
            }
        } else {
            self.left_held = -1.0;
        }

        self.solve_check()
    }

    pub fn render(&self, buffer: &mut [u8]) {
        let mut image = Image::new(buffer, 200, 200);

        for i in (0..image.buffer.len()).step_by(4) {
            image.buffer[i] = DEFAULT_PALETTE[4][0];
            image.buffer[i + 1] = DEFAULT_PALETTE[4][1];
            image.buffer[i + 2] = DEFAULT_PALETTE[4][2];
            image.buffer[i + 3] = 0xff;
        }

        image.draw_rectangle_with(50, 0, 100, 200, || DEFAULT_PALETTE[5]);

        self.draw_grid(&mut image);

        for_each_block(
            self.selected_piece,
            self.held_piece_pos.x,
            self.held_piece_pos.y,
            self.held_piece_dir,
            |x, y| {
                image.draw_snail(
                    COLORS[self.selected_piece],
                    true,
                    self.held_piece_dir,
                    50 + x as usize * 10,
                    y as usize * 10,
                );
            },
        );
    }

    fn solve_check(&mut self) -> i64 {
        let mut has_four_lines = false;
        let mut score = 0;
        let mut line_streak = 0;

        let mut y = 0;
        while y < HEIGHT {
            let mut full_line = true;

            for x in 0..WIDTH {
                if self.grid[y * WIDTH + x] == 0 {
                    full_line = false;
                    break;
                }
            }

            if full_line {
                line_streak += 1;

                for x in 0..WIDTH {
                    self.grid[y * WIDTH + x] = 0;
                }

                self.shift_down(y);
            } else {
                if line_streak > 0 {
                    if line_streak == 4 {
                        has_four_lines = true;
                    }

                    score += 2_000_000 * LINE_SCORES[line_streak - 1];
                    line_streak = 0;
                }
            }

            y += 1;
        }

        if line_streak > 0 {
            if line_streak == 4 {
                has_four_lines = true;
            }

            score += 2_000_000 * LINE_SCORES[line_streak - 1];
        }

        if has_four_lines {
            if score != 0 {}

            -score
        } else {
            if score != 0 {}

            score
        }
    }

    fn shift_down(&mut self, start_y: usize) {
        for y in (1..(start_y + 1)).rev() {
            for x in 0..WIDTH {
                let tmp = self.grid[y * WIDTH + x];
                self.grid[y * WIDTH + x] = self.grid[(y - 1) * WIDTH + x];
                self.grid[(y - 1) * WIDTH + x] = tmp;
            }
        }
    }

    fn draw_grid(&self, image: &mut Image) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let cell = self.grid[y * WIDTH + x];
                let direction = Direction::from_number((cell >> 6) as usize);

                let color = cell & 0b00111111;

                if color != 0 {
                    image.draw_snail(
                        COLORS[(color - 1) as usize],
                        true,
                        direction,
                        50 + x * 10,
                        y * 10,
                    );
                }
            }
        }
    }

    fn reset_piece(&mut self, lfsr: &mut LFSR) {
        for_each_block(
            self.selected_piece,
            self.held_piece_pos.x,
            self.held_piece_pos.y,
            self.held_piece_dir,
            |x, y| {
                let cell_value =
                    ((self.held_piece_dir as u8) << 6) | (self.selected_piece + 1) as u8;
                self.grid[y as usize * WIDTH + x as usize] = cell_value;
            },
        );

        self.held_piece_pos = Vec2i::new(4, 0);
        self.held_piece_dir = Direction::Right;
        self.selected_piece = lfsr.big() % BLOCKS.len();

        // if piece collides, we reset the whole board

        if self.held_piece_collides(
            self.held_piece_pos.x,
            self.held_piece_pos.y,
            self.held_piece_dir,
        ) {
            self.grid.fill(0);
        }
    }

    fn held_piece_collides(&self, px: i32, py: i32, dir: Direction) -> bool {
        let mut collides = false;

        for_each_block(self.selected_piece, px, py, dir, |x, y| {
            if !(x >= 0
                && y >= 0
                && x < WIDTH as i32
                && y < HEIGHT as i32
                && self.grid[y as usize * WIDTH + x as usize] & 0b00111111 == 0)
            {
                collides = true;
            }
        });

        collides
    }
}
