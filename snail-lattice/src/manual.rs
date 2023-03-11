use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, ANIMATION_TIME, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE},
    utils::{console_log, Vec2},
};

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum SolveType {
    None,
    Regular,
    Special,
}

// am i really going to implemennt 3 full parody games inside my snail maze incremental game?
// yes, yes i am
enum ManualGame {
    SnailMaze(ManualMaze),
    Asteroids,
    PacMan,
    Wolfenstein,
}

#[wasm_bindgen]
pub struct Game {
    game: ManualGame,
    lfsr: LFSR,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u16) -> Self {
        let mut lfsr = LFSR::new(seed);

        Self {
            game: ManualGame::SnailMaze(ManualMaze::new(&mut lfsr)),
            lfsr,
        }
    }

    // returns an array with two elements, can't use tuples because wasm_bindgen is sad.
    #[wasm_bindgen]
    pub fn resolution(&self) -> Vec<u32> {
        match &self.game {
            ManualGame::SnailMaze(game) => game.resolution(),
            ManualGame::Asteroids => todo!(),
            ManualGame::PacMan => todo!(),
            ManualGame::Wolfenstein => todo!(),
        }
    }

    // keys is a list of keys in the order the were pressed
    // mappings:
    // 1 => right
    // 2 => left
    // 4 => down
    // 8 => up
    #[wasm_bindgen]
    pub fn render(&mut self, buffer: &mut [u8], keys: Vec<u32>, dt: usize) -> SolveType {
        match &mut self.game {
            ManualGame::SnailMaze(game) => {
                let ret = game.tick(&mut self.lfsr, keys, dt);
                game.render(buffer);
                ret
            }
            ManualGame::Asteroids => todo!(),
            ManualGame::PacMan => todo!(),
            ManualGame::Wolfenstein => todo!(),
        }
    }
}

const MANUAL_MOVEMENT_TIME: usize = SNAIL_MOVEMENT_TIME / 2;

struct ManualMaze {
    snail: Snail<7>,
    maze: Maze<7>,
    end_pos: Vec2,
    bg_buffer: Vec<u8>,

    solve_type: SolveType,
    time: usize,
    movement_timer: usize,
}

impl ManualMaze {
    fn new(lfsr: &mut LFSR) -> Self {
        let mut bg_buffer = vec![0; 4 * 71 * 71];
        let mut image = Image {
            buffer: &mut bg_buffer,
            buffer_width: 71,
        };

        let mut maze = Maze::new();
        maze.generate(lfsr);

        maze.draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image, 0, 0);

        ManualMaze {
            snail: Snail::new(),
            maze,
            end_pos: Vec2 { x: 6, y: 6 },
            bg_buffer,
            movement_timer: MANUAL_MOVEMENT_TIME,
            solve_type: SolveType::Regular,
            time: 0,
        }
    }

    fn resolution(&self) -> Vec<u32> {
        vec![71, 71]
    }

    fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: usize) -> SolveType {
        self.time = self.time.wrapping_add(dt);

        self.movement_timer += dt;
        let can_move = self.movement_timer / MANUAL_MOVEMENT_TIME > 0;
        self.movement_timer %= MANUAL_MOVEMENT_TIME;

        if can_move {
            let direction = match keys.first() {
                Some(1) => Some(Direction::Right),
                Some(2) => Some(Direction::Left),
                Some(4) => Some(Direction::Down),
                Some(8) => Some(Direction::Up),
                _ => {
                    self.snail.prev_pos = self.snail.pos;
                    self.movement_timer = MANUAL_MOVEMENT_TIME;
                    None
                }
            };

            if let Some(direction) = direction {
                self.snail.direction = direction;
                if !self.snail.move_forward(&self.maze) {
                    self.movement_timer = MANUAL_MOVEMENT_TIME;
                }
            }
        }

        if self.snail.prev_pos == self.end_pos {
            self.snail.reset();

            self.maze.generate(lfsr);

            let mut image = Image {
                buffer: &mut self.bg_buffer,
                buffer_width: 71,
            };

            let solve_type = self.solve_type;

            if lfsr.big() % 10 == 0 {
                self.solve_type = SolveType::Special;
            } else {
                self.solve_type = SolveType::Regular;
            }

            if self.solve_type == SolveType::Regular {
                self.maze
                    .draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image, 0, 0);
            } else {
                self.maze.draw_background(
                    GRAYSCALE_PALETTE[4],
                    GRAYSCALE_PALETTE[5],
                    &mut image,
                    0,
                    0,
                );
            }

            solve_type
        } else {
            SolveType::None
        }
    }

    fn render(&mut self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bg_buffer);

        let mut image = Image {
            buffer,
            buffer_width: 71,
        };

        let animation_cycle = (self.time / ANIMATION_TIME) % 2 == 0;

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            self.movement_timer,
            MANUAL_MOVEMENT_TIME,
            &mut image,
            0,
            0,
        );

        if animation_cycle {
            image.draw_goal(DEFAULT_PALETTE[0], self.end_pos, 0, 0);
        }
    }
}
