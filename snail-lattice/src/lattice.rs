use std::collections::HashSet;

use wasm_bindgen::prelude::*;

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{AutoMaze, CELLS_PER_IDX},
    solvers::{Clones, HoldLeft, RandomTeleport, RandomWalk, Solver, TimeTravel, Tremaux},
    utils::{console_log, set_panic_hook},
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

    pub fn get_dimensions(&self, count: usize) -> Vec<usize> {
        // ceiling division -> count / width
        let height = (count + self.width - 1) / self.width;

        let height_px = (S * 10 + 1) * height;
        let width_px = (S * 10 + 1) * self.width;

        vec![width_px, height_px]
    }

    // renders to a buffer of size 4*self.get_dimensions()
    pub fn render(&mut self, buffer: &mut [u8], index: usize, count: usize) {
        let dimensions = self.get_dimensions(count);

        // just so we don't panic in case the javascript code messes up
        if buffer.len() != 4 * dimensions[0] * dimensions[1] {
            return;
        }

        let maze_size = S * 10 + 1;
        let width = maze_size * self.width;

        buffer.fill(0);

        let mut cx = 0;
        let mut cy = 0;

        let mut image = Image {
            buffer,
            buffer_width: width,
        };

        for maze in self.mazes.iter_mut().skip(index).take(count) {
            maze.maze.draw_background(&mut image, cx, cy);
            maze.draw(&mut self.lfsr, &mut image, cx, cy);

            cx += maze_size;
            if cx >= width {
                cx = 0;
                cy += maze_size;
            }
        }
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    // progresses all snails a certain number of microseconds
    // returns the number of maze framents accrued
    pub fn tick(&mut self, dt: usize) -> usize {
        let mut total = 0;

        for maze in self.mazes.iter_mut() {
            let fragments = maze.tick(dt, &mut self.lfsr);
            if fragments != 0 {
                total += fragments;
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
            pub fn get_dimensions(&self, count: usize) -> Vec<usize> {
                self.0.get_dimensions(count)
            }

            #[wasm_bindgen]
            pub fn render(&mut self, buffer: &mut [u8], index: usize, count: usize) {
                self.0.render(buffer, index, count);
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
