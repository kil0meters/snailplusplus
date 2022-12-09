use std::collections::HashSet;

use wasm_bindgen::prelude::*;

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{AutoMaze, CELLS_PER_IDX},
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

pub struct SnailLattice<const S: usize, T: Solver<S>>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    width: usize,
    mazes: Vec<AutoMaze<S, T>>,

    // stores the indexes of mazes which need to be rerendered
    render_marked: HashSet<usize>,

    bg_buffer: Vec<u8>,

    lfsr: LFSR,
}

impl<const S: usize, T: Solver<S>> SnailLattice<S, T>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    pub fn new(width: usize, seed: u16) -> SnailLattice<S, T> {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        let mut lattice = SnailLattice {
            width,
            mazes: Vec::new(),
            render_marked: HashSet::new(),
            bg_buffer: Vec::new(),
            lfsr: LFSR::new(seed),
        };

        for maze in lattice.mazes.iter_mut() {
            maze.maze.generate(&mut lattice.lfsr);
        }

        lattice
    }

    pub fn count(&self) -> usize {
        self.mazes.len()
    }

    pub fn get_dimensions(&self) -> Vec<usize> {
        // ceiling division -> count / width
        let height = (self.mazes.len() + self.width - 1) / self.width;

        // let height_px = (self.maze_size * 10 + 1) * height;
        let height_px = (S * 10 + 1) * height;
        // let width_px = (self.maze_size * 10 + 1) * self.width;
        let width_px = (S * 10 + 1) * self.width;

        vec![width_px, height_px]
    }

    fn draw_mazes(&mut self) {
        let dimensions = self.get_dimensions();
        let width = dimensions[0];
        let height = dimensions[1];

        self.bg_buffer.clear();
        self.bg_buffer.resize(width * height * 4, 0);

        // let maze_size = self.maze_size * 10 + 1;
        let maze_size = S * 10 + 1;

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
    pub fn render(&mut self, buffer: &mut [u8]) {
        // just so we don't panic in case the javascript code messes up
        if self.bg_buffer.len() != buffer.len() {
            return;
        }

        let maze_size = S * 10 + 1;
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

    pub fn set_width(&mut self, width: usize) {
        self.width = width;

        self.draw_mazes();
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
            }
        }

        total
    }

    pub fn alter(&mut self, difference: i32) {
        if difference < 0 {
            for _ in 0..difference.abs() {
                self.mazes.pop();
            }
        } else {
            for _ in 0..difference {
                let mut new_maze = AutoMaze::<S, T>::new(T::new());
                new_maze.maze.generate(&mut self.lfsr);
                self.mazes.push(new_maze);
            }
        }

        self.draw_mazes();
    }
}

macro_rules! lattice_impl {
    ($name:tt, $size:literal, $solver:ty) => {
        #[wasm_bindgen]
        pub struct $name(SnailLattice<$size, $solver>);

        #[wasm_bindgen]
        impl $name {
            #[wasm_bindgen(constructor)]
            pub fn new(width: usize, seed: u16) -> Self {
                Self(SnailLattice::new(width, seed))
            }

            #[wasm_bindgen]
            pub fn get_dimensions(&self) -> Vec<usize> {
                self.0.get_dimensions()
            }

            #[wasm_bindgen]
            pub fn render(&mut self, buffer: &mut [u8]) {
                self.0.render(buffer);
            }

            #[wasm_bindgen]
            pub fn tick(&mut self, dt: usize) -> usize {
                self.0.tick(dt)
            }

            #[wasm_bindgen]
            pub fn alter(&mut self, difference: i32) {
                self.0.alter(difference);
            }

            #[wasm_bindgen]
            pub fn count(&self) -> usize {
                self.0.mazes.len()
            }

            #[wasm_bindgen]
            pub fn set_width(&mut self, width: usize) {
                self.0.set_width(width);
            }
        }
    };
}

lattice_impl!(RandomWalkLattice, 5, RandomWalk<5>);
lattice_impl!(RandomTeleportLattice, 7, RandomTeleport<7>);
lattice_impl!(HoldLeftLattice, 9, HoldLeft<9>);
lattice_impl!(TremauxLattice, 11, Tremaux<11>);
lattice_impl!(TimeTravelLattice, 13, TimeTravel<13>);
lattice_impl!(CloneLattice, 20, Clones<20>);
