use std::collections::{BTreeMap, BTreeSet};

use wasm_bindgen::prelude::*;

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{AutoMaze, CELLS_PER_IDX},
    solvers::{
        Clones, HoldLeft, Inverted, Learning, RandomTeleport, RandomWalk, Rpg, Solver, TimeTravel,
        Tremaux,
    },
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

pub trait TilableMaze {
    const SIZE: usize;

    fn new() -> Self;
    fn tick(&mut self, dt: usize, lfsr: &mut LFSR) -> usize;
    fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image, bx: usize, by: usize);
    fn draw_background(&mut self, image: &mut Image, bx: usize, by: usize);
    fn generate(&mut self, lfsr: &mut LFSR);
}

pub struct SnailLattice<LatticeElement>
where
    LatticeElement: TilableMaze,
{
    width: usize,
    mazes: Vec<LatticeElement>,
    lfsr: LFSR,

    // assumes non-overlapping ranges, and assumes maxes out the index at 2^16.
    // should be fine for now. if not we can always change to a tuple later
    // we're also always going to be dealing with a very small amount of buffers so using a
    // b trees is more efficient than hashmaps here
    bg_buffers: BTreeMap<usize, Vec<u8>>,
    render_marked: BTreeSet<usize>,
}

impl<LatticeElement: TilableMaze> SnailLattice<LatticeElement> {
    pub fn new(width: usize, seed: u16) -> SnailLattice<LatticeElement> {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        let mut lattice = SnailLattice::<LatticeElement> {
            width,
            mazes: Vec::new(),
            lfsr: LFSR::new(seed),
            bg_buffers: BTreeMap::new(),
            render_marked: BTreeSet::new(),
        };

        for maze in lattice.mazes.iter_mut() {
            maze.generate(&mut lattice.lfsr);
        }

        lattice
    }

    pub fn count(&self) -> usize {
        self.mazes.len()
    }

    pub fn get_dimensions(&self, count: usize) -> Vec<usize> {
        // ceiling division -> count / width
        let height = (count + self.width - 1) / self.width;

        let height_px = (LatticeElement::SIZE * 10 + 1) * height;
        let width_px = (LatticeElement::SIZE * 10 + 1) * self.width;

        vec![width_px, height_px]
    }

    // renders to a buffer of size 4*self.get_dimensions()
    pub fn render(&mut self, buffer: &mut [u8], index: usize, count: usize) {
        let dimensions = self.get_dimensions(count);
        let buffer_size = 4 * dimensions[0] * dimensions[1];

        // just so we don't panic in case the javascript code messes up
        if buffer.len() != buffer_size {
            return;
        }

        let maze_size = LatticeElement::SIZE * 10 + 1;
        let width = maze_size * self.width;

        let bg_buffer = match self.bg_buffers.get_mut(&(index << 16 + count)) {
            Some(buffer) => {
                let mut bg_image = Image {
                    buffer,
                    buffer_width: dimensions[0],
                };

                let indexes = self
                    .render_marked
                    .range(index..(index + count))
                    .cloned()
                    .collect::<Vec<_>>();

                for i in indexes {
                    self.mazes[i].draw_background(
                        &mut bg_image,
                        maze_size * ((i - index) % self.width),
                        maze_size * ((i - index) / self.width),
                    );

                    self.render_marked.remove(&i);
                }

                buffer
            }
            None => {
                let mut bg_buffer = vec![0; buffer_size];

                let mut bg_image = Image {
                    buffer: &mut bg_buffer,
                    buffer_width: dimensions[0],
                };

                for (i, maze) in self.mazes.iter_mut().skip(index).take(count).enumerate() {
                    maze.draw_background(
                        &mut bg_image,
                        maze_size * (i % self.width),
                        maze_size * (i / self.width),
                    );
                }

                self.bg_buffers.insert(index << 16 + count, bg_buffer);
                self.bg_buffers.get_mut(&(index << 16 + count)).unwrap()
            }
        };

        buffer.copy_from_slice(bg_buffer);

        let mut cx = 0;
        let mut cy = 0;

        let mut image = Image {
            buffer,
            buffer_width: width,
        };

        for maze in self.mazes.iter_mut().skip(index).take(count) {
            maze.draw_foreground(&mut self.lfsr, &mut image, cx, cy);

            cx += maze_size;
            if cx >= width {
                cx = 0;
                cy += maze_size;
            }
        }
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;

        self.render_marked.clear();
        self.bg_buffers.clear();
    }

    // progresses all snails a certain number of microseconds
    // returns the number of maze framents accrued
    pub fn tick(&mut self, dt: usize) -> usize {
        let mut total = 0;

        for (i, maze) in self.mazes.iter_mut().enumerate() {
            let fragments = maze.tick(dt, &mut self.lfsr);
            if fragments != 0 {
                total += fragments;
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

            self.bg_buffers.clear();
            self.render_marked.clear();
        } else {
            let mut time_offset = 0;

            for _ in 0..difference {
                let mut new_maze = LatticeElement::new();
                new_maze.generate(&mut self.lfsr);

                // offset time slightly
                new_maze.tick(time_offset, &mut self.lfsr);

                self.render_marked.insert(self.mazes.len());
                self.mazes.push(new_maze);

                time_offset += 100000;
            }
        }
    }
}

#[wasm_bindgen]
pub struct MetaMaze {
    random_walk: AutoMaze<7, RandomWalk<7>>,
    random_teleport: AutoMaze<7, RandomTeleport<7>>,
    learning: AutoMaze<7, Learning<7>>,
    hold_left: AutoMaze<7, HoldLeft<7>>,
    inverted: AutoMaze<7, Inverted<7>>,
    tremaux: AutoMaze<7, Tremaux<7>>,
    time_travel: AutoMaze<7, TimeTravel<7>>,
    clone: AutoMaze<7, Clones<7>>,
    rpg: AutoMaze<7, Rpg<7>>,
}

impl TilableMaze for MetaMaze {
    const SIZE: usize = 7 * 3;

    fn new() -> Self {
        MetaMaze {
            random_walk: AutoMaze::new(),
            random_teleport: AutoMaze::new(),
            learning: AutoMaze::new(),
            hold_left: AutoMaze::new(),
            inverted: AutoMaze::new(),
            tremaux: AutoMaze::new(),
            time_travel: AutoMaze::new(),
            clone: AutoMaze::new(),
            rpg: AutoMaze::new(),
        }
    }

    fn tick(&mut self, dt: usize, lfsr: &mut LFSR) -> usize {
        let mut total = 0;

        total += self.random_walk.tick(dt, lfsr);
        total += self.random_teleport.tick(dt, lfsr);
        total += self.learning.tick(dt, lfsr);
        total += self.hold_left.tick(dt, lfsr);
        total += self.inverted.tick(dt, lfsr);
        total += self.tremaux.tick(dt, lfsr);
        total += self.time_travel.tick(dt, lfsr);
        total += self.clone.tick(dt, lfsr);
        total += self.rpg.tick(dt, lfsr);

        total
    }

    fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image, bx: usize, by: usize) {
        self.random_walk.draw_foreground(lfsr, image, bx, by);
        self.random_teleport
            .draw_foreground(lfsr, image, bx + 70, by);
        self.learning.draw_foreground(lfsr, image, bx + 140, by);
        self.hold_left.draw_foreground(lfsr, image, bx, by + 70);
        self.inverted.draw_foreground(lfsr, image, bx + 70, by + 70);
        self.tremaux.draw_foreground(lfsr, image, bx + 140, by + 70);
        self.time_travel.draw_foreground(lfsr, image, bx, by + 140);
        self.clone.draw_foreground(lfsr, image, bx + 70, by + 140);
        self.rpg.draw_foreground(lfsr, image, bx + 140, by + 140);
    }

    fn draw_background(&mut self, image: &mut Image, bx: usize, by: usize) {
        self.random_walk.draw_background(image, bx, by);
        self.random_teleport.draw_background(image, bx + 70, by);
        self.learning.draw_background(image, bx + 140, by);
        self.hold_left.draw_background(image, bx, by + 70);
        self.inverted.draw_background(image, bx + 70, by + 70);
        self.tremaux.draw_background(image, bx + 140, by + 70);
        self.time_travel.draw_background(image, bx, by + 140);
        self.clone.draw_background(image, bx + 70, by + 140);
        self.rpg.draw_background(image, bx + 140, by + 140);
    }

    fn generate(&mut self, lfsr: &mut LFSR) {
        self.random_walk.generate(lfsr);
        self.random_teleport.generate(lfsr);
        self.learning.generate(lfsr);
        self.hold_left.generate(lfsr);
        self.inverted.generate(lfsr);
        self.tremaux.generate(lfsr);
        self.time_travel.generate(lfsr);
        self.clone.generate(lfsr);
        self.rpg.generate(lfsr);
    }
}

//
// #[wasm_bindgen]
// impl MetaLattice {
//     #[wasm_bindgen(constructor)]
//     pub fn new(width: usize, seed: u16) -> MetaLattice {
//         MetaLattice {
//             width,
//             lfsr: LFSR::new(seed),
//
//             random_walk_mazes: vec![],
//             random_teleport: vec![],
//             learning: vec![],
//             hold_left: vec![],
//             tremaux: vec![],
//             time_travel: vec![],
//             clone: vec![],
//             rpg: vec![],
//
//             bg_buffers: BTreeMap::new(),
//             render_marked: BTreeSet::new(),
//         }
//     }
//
//     #[wasm_bindgen]
//     pub fn get_dimensions(&self, count: usize) -> Vec<usize> {
//         self.0.get_dimensions(count)
//     }
//
//     #[wasm_bindgen]
//     pub fn render(&mut self, buffer: &mut [u8], index: usize, count: usize) {
//         self.0.render(buffer, index, count);
//     }
//
//     #[wasm_bindgen]
//     pub fn tick(&mut self, dt: usize) -> usize {
//         self.0.tick(dt)
//     }
//
//     #[wasm_bindgen]
//     pub fn alter(&mut self, difference: i32) {
//         self.0.alter(difference);
//     }
//
//     #[wasm_bindgen]
//     pub fn count(&self) -> usize {
//         self.0.mazes.len()
//     }
//
//     #[wasm_bindgen]
//     pub fn set_width(&mut self, width: usize) {
//         self.0.set_width(width);
//     }
// }

macro_rules! lattice_impl {
    ($name:tt, $tile:ty) => {
        #[wasm_bindgen]
        pub struct $name(SnailLattice<$tile>);

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

lattice_impl!(RandomWalkLattice, AutoMaze<5, RandomWalk<5>>);
lattice_impl!(RandomTeleportLattice, AutoMaze<7, RandomTeleport<7>>);
lattice_impl!(LearningLattice, AutoMaze<9, Learning<9>>);
lattice_impl!(HoldLeftLattice, AutoMaze<9, HoldLeft<9>>);
lattice_impl!(InvertedLattice, AutoMaze<9, Inverted<9>>);
lattice_impl!(TremauxLattice, AutoMaze<11, Tremaux<11>>);
lattice_impl!(TimeTravelLattice, AutoMaze<13, TimeTravel<13>>);
lattice_impl!(CloneLattice, AutoMaze<20, Clones<20>>);
lattice_impl!(RpgLattice, AutoMaze<7, Rpg<7>>);
lattice_impl!(MetaLattice, MetaMaze);
