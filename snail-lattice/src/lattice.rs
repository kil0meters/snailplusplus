use std::collections::HashSet;

use wasm_bindgen::prelude::*;

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::AutoMaze,
    solvers::{Clones, HoldLeft, RandomTeleport, RandomWalk, Solver, TimeTravel, Tremaux},
    utils::set_panic_hook,
};

#[derive(Clone, Copy)]
pub enum MazeType {
    RandomTeleport,
    RandomWalk,
    HoldLeft,
    Tremaux,
    TimeTravel,
    Clone,
}

#[wasm_bindgen]
pub struct SnailLattice {
    width: usize,
    maze_size: usize,
    mazes: Vec<AutoMaze>,
    maze_type: MazeType,

    // stores the indexes of mazes which need to be rerendered
    render_marked: HashSet<usize>,

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
        seed: u16,
    ) -> SnailLattice {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        let maze_type = match maze_type {
            "random-walk" => MazeType::RandomWalk,
            "random-teleport" => MazeType::RandomTeleport,
            "hold-left" => MazeType::HoldLeft,
            "tremaux" => MazeType::Tremaux,
            "time-travel" => MazeType::TimeTravel,
            "clone" => MazeType::Clone,
            _ => unreachable!(),
        };

        let mut lattice = SnailLattice {
            width,
            maze_size,
            mazes: Vec::new(),
            maze_type,
            render_marked: HashSet::new(),
            bg_buffer: Vec::new(),
            lfsr: LFSR::new(seed),
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

        let mut bg_image = Image {
            buffer: &mut self.bg_buffer,
            buffer_width: width,
        };

        for maze in self.mazes.iter_mut() {
            maze.maze.draw_background(&mut bg_image, cx, cy);

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

        let maze_size = self.maze_size * 10 + 1;
        let width = maze_size * self.width;

        if !self.render_marked.is_empty() {
            let mut bg_image = Image {
                buffer_width: width,
                buffer: &mut self.bg_buffer,
            };

            // render all things necessary
            for &i in self.render_marked.iter() {
                self.mazes[i].maze.draw_background(
                    &mut bg_image,
                    maze_size * (i % self.width),
                    maze_size * (i / self.width),
                );
            }

            self.render_marked.clear();
        }

        buffer.copy_from_slice(&self.bg_buffer);

        let mut cx = 0;
        let mut cy = 0;

        let mut image = Image {
            buffer,
            buffer_width: width,
        };

        // render foreground of each maze into framebuffer
        // also updates mazes if necessary
        for maze in self.mazes.iter_mut() {
            maze.draw(&mut self.lfsr, &mut image, cx, cy);

            cx += maze_size; // maze_size;
            if cx >= width {
                cx = 0;
                cy += maze_size;
            }
        }
    }

    // progresses all snails a certain number of microseconds
    // returns the number of maze framents accrued
    pub fn tick(&mut self, dt: usize) -> usize {
        let mut total = 0;

        for (i, maze) in self.mazes.iter_mut().enumerate() {
            let fragments = maze.tick(dt, &mut self.lfsr);
            if fragments != 0 {
                total += fragments;

                // mark that the current maze needs to be rerendered
                self.render_marked.insert(i);
                // queue
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
            let solver_builder: Box<dyn Fn() -> Box<dyn Solver>> = match self.maze_type {
                MazeType::RandomWalk => Box::new(|| Box::new(RandomWalk::new(0))),
                MazeType::RandomTeleport => Box::new(|| Box::new(RandomTeleport::new(0))),
                MazeType::HoldLeft => Box::new(|| Box::new(HoldLeft::new(0))),
                MazeType::Tremaux => Box::new(|| Box::new(Tremaux::new(0))),
                MazeType::TimeTravel => Box::new(|| Box::new(TimeTravel::new(0))),
                MazeType::Clone => Box::new(|| Box::new(Clones::new(0))),
            };

            for _ in 0..difference {
                let mut new_maze = AutoMaze::new(solver_builder(), self.maze_size, self.maze_size);
                new_maze.maze.generate(&mut self.lfsr);
                self.mazes.push(new_maze);
            }
        }

        self.draw_mazes();
    }
}
