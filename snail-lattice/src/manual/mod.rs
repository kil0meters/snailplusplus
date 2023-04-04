use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, ANIMATION_TIME, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE},
    utils::{set_panic_hook, Vec2},
};

use self::{
    asteroids::AsteroidsGame, falling_snails::FallingSnailsGame, pacsnail::PacSnail,
    wolfenstein::WolfensteinGame,
};

mod asteroids;
mod falling_snails;
mod pacsnail;
mod wolfenstein;

// am i really going to implemennt 3 full parody games inside my snail maze incremental game?
// yes, yes i am
enum ManualGame {
    SnailMaze(ManualMaze),
    PacSnail(PacSnail),
    Asteroids(AsteroidsGame),
    Wolfenstein(WolfensteinGame),
    FallingSnails(FallingSnailsGame),
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
        set_panic_hook();

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
            ManualGame::Asteroids(game) => game.resolution(),
            ManualGame::PacSnail(game) => game.resolution(),
            ManualGame::Wolfenstein(game) => game.resolution(),
            ManualGame::FallingSnails(game) => game.resolution(),
        }
    }

    #[wasm_bindgen]
    pub fn set_game(&mut self, game_type: u32) {
        match game_type {
            0 => self.game = ManualGame::SnailMaze(ManualMaze::new(&mut self.lfsr)),
            1 => self.game = ManualGame::PacSnail(PacSnail::new()),
            2 => self.game = ManualGame::Asteroids(AsteroidsGame::new()),
            3 => self.game = ManualGame::Wolfenstein(WolfensteinGame::new(&mut self.lfsr)),
            4 => self.game = ManualGame::FallingSnails(FallingSnailsGame::new(&mut self.lfsr)),
            _ => unreachable!(),
        }
    }

    #[wasm_bindgen]
    pub fn render(&mut self, buffer: &mut [u8], keys: Vec<u32>, mut dt: f32) -> i64 {
        match &mut self.game {
            ManualGame::SnailMaze(game) => {
                let ret = game.tick(&mut self.lfsr, keys, dt);
                game.render(buffer);
                ret as i64
            }
            ManualGame::PacSnail(game) => {
                let mut ret = 0;
                while dt > 30.0 {
                    ret += game.tick(&mut self.lfsr, &keys, 30.0);
                    dt -= 30.0;
                }
                ret += game.tick(&mut self.lfsr, &keys, dt);

                game.render(buffer);
                ret as i64
            }
            ManualGame::Asteroids(game) => {
                let ret = game.tick(&mut self.lfsr, keys, dt);
                game.render(buffer);
                ret as i64
            }
            ManualGame::Wolfenstein(game) => {
                let ret = game.tick(&mut self.lfsr, keys, dt);
                game.render(buffer);
                ret as i64
            }
            ManualGame::FallingSnails(game) => {
                let ret = game.tick(&mut self.lfsr, keys, dt);
                game.render(buffer);
                ret
            }
        }
    }
}

const MANUAL_MOVEMENT_TIME: f32 = SNAIL_MOVEMENT_TIME / 2.0;

struct ManualMaze {
    snail: Snail,
    maze: Maze,
    end_pos: Vec2,
    bg_buffer: Vec<u8>,

    solve_type: i32,
    time: f32,
    movement_timer: f32,
}

impl ManualMaze {
    fn new(lfsr: &mut LFSR) -> Self {
        let mut bg_buffer = vec![0; 4 * 71 * 71];
        let mut image = Image::new(&mut bg_buffer, 71, 71);
        let mut maze = Maze::new(7);
        maze.generate(lfsr);

        maze.draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image);

        ManualMaze {
            snail: Snail::new(),
            maze,
            end_pos: Vec2 { x: 6, y: 6 },
            bg_buffer,
            movement_timer: MANUAL_MOVEMENT_TIME,
            solve_type: 25,
            time: 0.0,
        }
    }

    fn resolution(&self) -> Vec<u32> {
        vec![71, 71]
    }

    // keys is a list of keys in the order the were pressed
    // mappings:
    // 1 => right
    // 2 => left
    // 4 => down
    // 8 => up
    fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: f32) -> i32 {
        self.time += dt;

        self.movement_timer += dt;
        let can_move = self.movement_timer / MANUAL_MOVEMENT_TIME >= 1.0;
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

            let mut image = Image::new(&mut self.bg_buffer, 71, 71);

            let solve_type = self.solve_type;

            if lfsr.big() % 10 == 0 {
                self.solve_type = -25;
            } else {
                self.solve_type = 25;
            }

            if self.solve_type > 0 {
                self.maze
                    .draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image);
            } else {
                self.maze
                    .draw_background(GRAYSCALE_PALETTE[4], GRAYSCALE_PALETTE[5], &mut image);
            }

            solve_type
        } else {
            0
        }
    }

    fn render(&mut self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bg_buffer);

        let mut image = Image::new(buffer, 71, 71);

        let animation_cycle = (self.time / ANIMATION_TIME).floor() as usize % 2 == 0;

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            self.movement_timer / MANUAL_MOVEMENT_TIME,
            &mut image,
        );

        if animation_cycle {
            image.draw_goal(DEFAULT_PALETTE[0], self.end_pos.x * 10, self.end_pos.y * 10);
        }
    }
}
