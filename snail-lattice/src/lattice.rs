use wasm_bindgen::prelude::*;

use crate::{lfsr::LFSR, utils::set_panic_hook, maze::AutoMaze, solvers::{RandomWalk, HoldLeft, Solver}};

#[derive(Clone, Copy)]
pub enum MazeType {
    RandomWalk,
    HoldLeft,
    Tremaux,
}

#[wasm_bindgen]
pub struct SnailLattice {
    width: usize,
    maze_size: usize,
    mazes: Vec<AutoMaze>,
    maze_type: MazeType,

    bg_buffer: Vec<u8>,

    lfsr: LFSR,
}

#[wasm_bindgen]
impl SnailLattice {
    #[wasm_bindgen(constructor)]
    pub fn new(
        maze_type: &str,
        width: usize,
        maze_size: usize,
        count: usize,
        seed: u16
    ) -> SnailLattice {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        let maze_type = match maze_type {
            "random-walk" => MazeType::RandomWalk,
            "hold-left" => MazeType::HoldLeft,
            "tremaux" => MazeType::Tremaux,
            _ => unreachable!(),
        };

        let mut lattice = SnailLattice {
            width,
            maze_size,
            mazes: Vec::new(),
            maze_type,
            bg_buffer: Vec::new(),
            lfsr: LFSR::new(seed)
        };

        lattice.alter(count as i32);

        for maze in lattice.mazes.iter_mut() {
            maze.maze.generate(&mut lattice.lfsr);
        }

        lattice.draw_mazes();

        lattice
    }

    #[wasm_bindgen]
    pub fn get_dimensions(&self) -> Vec<usize> {
        // ceiling division -> count / width
        let height = (self.mazes.len() + self.width - 1) / self.width;

        let height_px = (self.maze_size * 10 + 1) * height;
        let width_px = (self.maze_size * 10 + 1) * self.width;

        vec![width_px, height_px]
    }

    fn draw_mazes(&mut self) {
        let dimensions = self.get_dimensions();
        let width = dimensions[0];
        let height = dimensions[1];

        self.bg_buffer.resize(width * height * 4, 0);

        let maze_size = self.maze_size * 10 + 1;

        let mut cx = 0;
        let mut cy = 0;

        for maze in self.mazes.iter_mut() {
            maze.maze.draw_background(&mut self.bg_buffer, width, cx, cy);

            cx += maze_size;
            if cx >= width {
                cx = 0;
                cy += maze_size;
            }
        }
    }

    // renders to a buffer of size 4*self.get_dimensions()
    #[wasm_bindgen]
    pub fn render(&mut self, buffer: &mut [u8]) {
        // just so we don't panic in case the javascript code messes up
        if self.bg_buffer.len() != buffer.len() {
            return;
        }

        buffer.copy_from_slice(&self.bg_buffer);

        let mut cx = 0;
        let mut cy = 0;

        let maze_size = self.maze_size * 10 + 1;
        let width = maze_size * self.width;

        // render foreground of each maze into framebuffer
        // also updates mazes if necessary
        for maze in self.mazes.iter_mut() {
            maze.draw(buffer, width, cx, cy);

            cx += maze_size;// maze_size;
            if cx >= width {
                cx = 0;
                cy += maze_size;
            }
        }
    }

    // progresses all snails a certain number of microseconds
    // returns the number of maze framents accrued
    pub fn tick(&mut self, dt: usize) -> usize {
        let maze_size = self.maze_size * 10 + 1;

        let mut total = 0;

        for (i, maze) in self.mazes.iter_mut().enumerate() {
            let fragments = maze.tick(dt, &mut self.lfsr);
            if fragments != 0 {
                total += fragments;

                maze.maze.draw_background(
                    &mut self.bg_buffer,
                    maze_size * self.width,
                    maze_size * (i % self.width),
                    maze_size * (i / self.width)
                );
            }
        }

        total
    }

    #[wasm_bindgen]
    pub fn alter(&mut self, difference: i32) {
        if difference < 0 {
            for _ in 0..difference.abs() {
                self.mazes.pop();
            }
        } else {
            let solver_builder: fn() -> Box<dyn Solver> = match self.maze_type {
                MazeType::RandomWalk => || Box::new(RandomWalk::new(0)),
                MazeType::HoldLeft => || Box::new(HoldLeft::new(0)),
                MazeType::Tremaux => || Box::new(RandomWalk::new(0)),
            };

            for _ in 0..difference {
                let mut new_maze = AutoMaze::new(
                    solver_builder(),
                    self.maze_size,
                    self.maze_size
                );
                new_maze.maze.generate(&mut self.lfsr);
                self.mazes.push(new_maze);
            }
        }

        self.draw_mazes();
    }
}

