use wasm_bindgen::prelude::*;

use crate::{snail_maze::SnailMaze, utils::set_panic_hook, lfsr::LFSR};

#[wasm_bindgen]
pub struct SnailLattice {
    width: usize,
    maze_size: usize,
    mazes: Vec<SnailMaze>,

    bg_buffer: Vec<u8>,

    lfsr: LFSR,
}

#[wasm_bindgen]
impl SnailLattice {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, maze_size: usize, count: usize, seed: u16) -> SnailLattice {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        let mut lattice = SnailLattice {
            width,
            maze_size,
            mazes: vec![SnailMaze::new(maze_size, maze_size); count],
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

        self.bg_buffer.reserve_exact(width * height * 4);
        // SAFETY: This is fine because we immedately set everything after.
        unsafe { self.bg_buffer.set_len(width * height * 4); }

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
    pub fn tick(&mut self, dt: usize) {
        let maze_size = self.maze_size * 10 + 1;

        for (i, maze) in self.mazes.iter_mut().enumerate() {
            if maze.tick(dt, &mut self.lfsr) {
                let width = (self.maze_size * 10 + 1) * self.width;

                maze.render_maze(
                    &mut self.bg_buffer,
                    width,
                    maze_size * (i % self.width),
                    maze_size * (i / self.width)
                );
            }
        }
    }

    #[wasm_bindgen]
    pub fn add(&mut self) {
        self.mazes.push(SnailMaze::new(self.maze_size, self.maze_size));
        self.draw_mazes();
    }
}

