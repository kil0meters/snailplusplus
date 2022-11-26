use wasm_bindgen::prelude::*;

use crate::{snail_maze::SnailMaze, utils::set_panic_hook, lfsr::LFSR};

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
    mazes: Vec<SnailMaze>,
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
            mazes: vec![SnailMaze::new(maze_type, maze_size, maze_size); count],
            maze_type,
            bg_buffer: Vec::new(),
            lfsr: LFSR::new(seed)
        };

        for maze in lattice.mazes.iter_mut() {
            maze.generate_maze(&mut lattice.lfsr);
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
            maze.render_maze(&mut self.bg_buffer, width, cx, cy);

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

        // render foreground of each maze into framebuffer
        // also updates mazes if necessary
        let maze_size = self.maze_size * 10 + 1;
        let mut cx = 0;
        let mut cy = 0;
        let width = self.get_dimensions()[0];

        for maze in self.mazes.iter_mut() {
            maze.render_foreground(buffer, width, cx, cy);

            cx += maze_size;
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

                maze.render_maze(
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
            for _ in 0..difference {
                let mut new_maze = SnailMaze::new(self.maze_type, self.maze_size, self.maze_size);
                new_maze.generate_maze(&mut self.lfsr);
                self.mazes.push(new_maze);
            }
        }

        self.draw_mazes();
    }
}

