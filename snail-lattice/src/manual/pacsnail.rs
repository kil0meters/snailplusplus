use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, ANIMATION_TIME, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    utils::{console_log, Vec2},
};

const PACMAN_BOARD: &[u8] = concat!(
    "#####################",
    "#. . . . .#. . . . .#",
    "# ### ### # ### ### #",
    "#* . . . . . . . . *#",
    "# ### # ##### # ### #",
    "#. . .#. .#. .#. . .#",
    "##### ### # ### #####",
    "#   #.#. . . .#.#   #",
    "##### # ##### # #####",
    "#    . .#   #. .    #",
    "##### # ##### # #####",
    "#   #.#. . . .#.#   #",
    "##### # ##### # #####",
    "#. . . . .#. . . . .#",
    "# ### ### # ### ### #",
    "#. .#. . . . . .#. .#",
    "### # # ##### # # ###",
    "#* . .#. .#. .#. . *#",
    "# ####### # ####### #",
    "#. . . . . . . . . .#",
    "#####################",
)
.as_bytes();

fn pacman_maze() -> (Maze<10>, Vec<Pellet>, usize) {
    let mut maze = Maze::new();
    let mut pellets = vec![Pellet::None; 10 * 10];
    let mut pellet_count = 0;
    let width = 21;

    for x in 0..10 {
        for y in 0..10 {
            let pos = (y * 2 + 1) * width + (x * 2 + 1);
            let mut cell = 0;

            if PACMAN_BOARD[pos] == b'.' {
                pellets[y * 10 + x] = Pellet::Pellet;
                pellet_count += 1;
            } else if PACMAN_BOARD[pos] == b'*' {
                pellets[y * 10 + x] = Pellet::Powerup;
                pellet_count += 1;
            }

            // right
            if PACMAN_BOARD[pos + 1] == b'#' {
                cell |= 1;
            }

            // left
            if PACMAN_BOARD[pos - 1] == b'#' {
                cell |= 2;
            }

            // up
            if PACMAN_BOARD[pos + width] == b'#' {
                cell |= 4;
            }

            // down
            if PACMAN_BOARD[pos - width] == b'#' {
                cell |= 8;
            }

            maze.set_cell(x, y, cell);
        }
    }

    (maze, pellets, pellet_count)
}

const PACMAN_MOVEMENT_TIME: usize = SNAIL_MOVEMENT_TIME * 3 / 2;
const POWERUP_TIME: usize = SNAIL_MOVEMENT_TIME * 50;

#[derive(Clone, Copy, PartialEq)]
enum Pellet {
    Pellet,
    Powerup,
    None,
}

pub struct PacSnail {
    bg_buffer: Vec<u8>,
    pellets: Vec<Pellet>,
    snail: Snail<10>,
    maze: Maze<10>,
    time: usize,
    stuck: bool,
    pellet_count: usize,
    next_direction: Option<Direction>,
    powerup_timer: usize,
    movement_timer: usize,
}

impl PacSnail {
    pub fn new() -> Self {
        let (mut maze, pellets, pellet_count) = pacman_maze();
        let mut bg_buffer = vec![0; 4 * 101 * 101];

        let mut image = Image {
            buffer: &mut bg_buffer,
            buffer_width: 101,
        };

        maze.draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image, 0, 0);

        let mut snail = Snail::new();
        snail.pos = Vec2 { x: 4, y: 7 };
        snail.prev_pos = snail.pos;

        let mut s = Self {
            maze,
            pellets,
            bg_buffer,
            snail,
            stuck: true,
            time: 0,
            pellet_count,
            powerup_timer: 0,
            next_direction: None,
            movement_timer: PACMAN_MOVEMENT_TIME,
        };

        s.draw_pellets();
        s
    }

    fn draw_pellet(&mut self, x: usize, y: usize) {
        match self.pellets[y * 10 + x] {
            Pellet::Pellet => {
                let px = 4 * ((y * 10 + 5) * 101 + x * 10 + 5);
                self.bg_buffer[px] = 0xFF;
                self.bg_buffer[px + 1] = 0xFF;
                self.bg_buffer[px + 2] = 0x00;
            }
            Pellet::Powerup => {
                let mut px = 4 * ((y * 10 + 4) * 101 + x * 10 + 4);

                for _ in 0..3 {
                    for x in 0..3 {
                        self.bg_buffer[px + 4 * x] = 0xFF;
                        self.bg_buffer[px + 4 * x + 1] = 0xFF;
                        self.bg_buffer[px + 4 * x + 2] = 0x00;
                    }
                    px += 4 * 101;
                }
            }
            Pellet::None => {
                let mut px = 4 * ((y * 10 + 4) * 101 + x * 10 + 4);

                for _ in 0..3 {
                    for x in 0..3 {
                        self.bg_buffer[px + 4 * x] = DEFAULT_PALETTE[5][0];
                        self.bg_buffer[px + 4 * x + 1] = DEFAULT_PALETTE[5][1];
                        self.bg_buffer[px + 4 * x + 2] = DEFAULT_PALETTE[5][2];
                    }
                    px += 4 * 101;
                }
            }
        }
    }

    fn draw_pellets(&mut self) {
        for x in 0..10 {
            for y in 0..10 {
                self.draw_pellet(x, y);
            }
        }
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![101, 101]
    }

    pub fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: usize) -> i32 {
        if self.pellet_count == 0 {
            let (maze, pellets, pellet_count) = pacman_maze();

            self.maze = maze;
            self.pellets = pellets;
            self.pellet_count = pellet_count;

            self.draw_pellets();

            self.snail.pos = Vec2 { x: 4, y: 7 };
            self.snail.prev_pos = self.snail.pos;

            self.stuck = true;

            return 0;
        }

        self.time = self.time.wrapping_add(dt);

        self.movement_timer += dt;
        let can_move = self.movement_timer / PACMAN_MOVEMENT_TIME > 0;

        match keys.first() {
            Some(1) => {
                self.next_direction = Some(Direction::Right);
                self.stuck = false;
            }
            Some(2) => {
                self.next_direction = Some(Direction::Left);
                self.stuck = false;
            }
            Some(4) => {
                self.next_direction = Some(Direction::Down);
                self.stuck = false;
            }
            Some(8) => {
                self.next_direction = Some(Direction::Up);
                self.stuck = false;
            }
            _ => {}
        }

        let mut score = 0;

        if !self.stuck && can_move {
            if let Some(next_direction) = self.next_direction {
                self.snail.direction = next_direction;
            }

            let pellet = self
                .pellets
                .get_mut(self.snail.pos.y * 10 + self.snail.pos.x)
                .unwrap();

            match *pellet {
                Pellet::Pellet => score = 5,
                Pellet::Powerup => {
                    score = 10;
                    self.powerup_timer = POWERUP_TIME;
                }
                Pellet::None => score = 0,
            }

            if *pellet != Pellet::None {
                self.pellet_count -= 1;
                *pellet = Pellet::None;
                self.draw_pellet(self.snail.pos.x, self.snail.pos.y);
            }

            self.next_direction = None;
            if !self.snail.move_forward(&self.maze) {
                self.stuck = true;
            } else {
                self.movement_timer = 0;
            }
        }

        score
    }

    pub fn render(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bg_buffer);
        let animation_cycle = self.stuck || (self.time / (ANIMATION_TIME / 4)) % 2 == 0;

        let mut image = Image {
            buffer,
            buffer_width: 101,
        };

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            self.movement_timer,
            PACMAN_MOVEMENT_TIME,
            &mut image,
            0,
            0,
        );
    }
}
