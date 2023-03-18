use std::collections::{BTreeMap, BTreeSet};

use wasm_bindgen::prelude::*;

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::AutoMaze,
    solvers::{
        Clones, Demolitionist, Flying, HoldLeft, Inverted, Learning, RandomTeleport, RandomWalk,
        Rpg, SolveStatus, TimeTravel, Tremaux,
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
    fn tick(&mut self, dt: f32, lfsr: &mut LFSR) -> SolveStatus;
    fn set_upgrades(&mut self, upgrades: u32);
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
    upgrades: u32,

    // stores the number of mazes solved by a given maze since the last query
    solve_count: Vec<u32>,

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
            upgrades: 0,
            mazes: Vec::new(),
            lfsr: LFSR::new(seed),
            solve_count: Vec::new(),
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
        for i in 0..buffer.len() {
            buffer[i] = 0xFF;
        }

        let dimensions = self.get_dimensions(count);
        let buffer_size = 4 * dimensions[0] * dimensions[1];

        // just so we don't panic in case the javascript code messes up
        if buffer.len() != buffer_size {
            return;
        }

        let maze_size = LatticeElement::SIZE * 10 + 1;

        let bg_buffer = match self.bg_buffers.get_mut(&((index << 16) + count)) {
            Some(buffer) => {
                let mut bg_image = Image {
                    buffer,
                    width: dimensions[0],
                    height: dimensions[1],
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
                    width: dimensions[0],
                    height: dimensions[1],
                };

                for (i, maze) in self.mazes.iter_mut().skip(index).take(count).enumerate() {
                    maze.draw_background(
                        &mut bg_image,
                        maze_size * (i % self.width),
                        maze_size * (i / self.width),
                    );
                }

                self.bg_buffers.insert((index << 16) + count, bg_buffer);
                self.bg_buffers.get_mut(&((index << 16) + count)).unwrap()
            }
        };

        buffer.copy_from_slice(bg_buffer);

        let mut cx = 0;
        let mut cy = 0;

        let mut image = Image {
            buffer,
            width: dimensions[0],
            height: dimensions[1],
        };

        for maze in self.mazes.iter_mut().skip(index).take(count) {
            maze.draw_foreground(&mut self.lfsr, &mut image, cx, cy);

            cx += maze_size;
            if cx >= dimensions[0] {
                cx = 0;
                cy += maze_size;
            }
        }
    }

    pub fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
        for maze in &mut self.mazes {
            maze.set_upgrades(self.upgrades);
        }
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;

        self.render_marked.clear();
        self.bg_buffers.clear();
    }

    // returns the index, then the number of solves for mazes this is better than the sparse
    // representation we store internally because it minimizes gc time in js land
    pub fn get_solve_count(&mut self) -> Vec<u32> {
        let mut solves = Vec::new();

        for (i, value) in self.solve_count.iter_mut().enumerate() {
            solves.push(i as u32);
            solves.push(*value);

            *value = 0;
        }

        solves
    }

    // progresses all snails a certain number of microseconds
    // returns the number of maze framents accrued
    pub fn tick(&mut self, dt: f32) -> usize {
        let mut total = 0;

        for (i, maze) in self.mazes.iter_mut().enumerate() {
            match maze.tick(dt, &mut self.lfsr) {
                SolveStatus::Solved(count) => {
                    total += count;
                    self.solve_count[i] += 1;
                    self.render_marked.insert(i);
                }
                SolveStatus::Rerender => {
                    self.render_marked.insert(i);
                }
                SolveStatus::None => {}
            }
        }

        total
    }

    pub fn alter(&mut self, difference: i32) {
        if difference < 0 {
            for _ in 0..difference.abs() {
                self.mazes.pop();
                self.solve_count.pop();
            }

            self.bg_buffers.clear();
            self.render_marked.clear();
        } else {
            let mut time_offset = 0.0;

            for _ in 0..difference {
                let mut new_maze = LatticeElement::new();
                new_maze.set_upgrades(self.upgrades);
                new_maze.generate(&mut self.lfsr);

                // offset time slightly
                new_maze.tick(time_offset, &mut self.lfsr);

                self.render_marked.insert(self.mazes.len());
                self.mazes.push(new_maze);
                self.solve_count.push(0);

                time_offset += 100.0;
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

    fn set_upgrades(&mut self, upgrades: u32) {
        self.random_walk.set_upgrades(upgrades & 0b111);
        self.random_teleport.set_upgrades((upgrades >> 3) & 0b111);
        self.learning.set_upgrades((upgrades >> 6) & 0b111);
        self.hold_left.set_upgrades((upgrades >> 9) & 0b111);
        self.inverted.set_upgrades((upgrades >> 12) & 0b111);
        self.tremaux.set_upgrades((upgrades >> 15) & 0b111);
        self.rpg.set_upgrades((upgrades >> 18) & 0b111);
        self.time_travel.set_upgrades((upgrades >> 21) & 0b111);
        self.clone.set_upgrades((upgrades >> 24) & 0b111);
    }

    fn tick(&mut self, dt: f32, lfsr: &mut LFSR) -> SolveStatus {
        let mut total = 0;

        total += self.random_walk.tick(dt, lfsr).get_count();
        total += self.random_teleport.tick(dt, lfsr).get_count();
        total += self.learning.tick(dt, lfsr).get_count();
        total += self.hold_left.tick(dt, lfsr).get_count();
        total += self.inverted.tick(dt, lfsr).get_count();
        total += self.tremaux.tick(dt, lfsr).get_count();
        total += self.time_travel.tick(dt, lfsr).get_count();
        total += self.clone.tick(dt, lfsr).get_count();
        total += self.rpg.tick(dt, lfsr).get_count();

        if total > 0 {
            SolveStatus::Solved(total)
        } else {
            SolveStatus::None
        }
    }

    fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image, bx: usize, by: usize) {
        self.random_walk.draw_foreground(lfsr, image, bx, by);
        self.random_teleport
            .draw_foreground(lfsr, image, bx + 70, by);
        self.learning.draw_foreground(lfsr, image, bx + 140, by);
        self.hold_left.draw_foreground(lfsr, image, bx, by + 70);
        self.inverted.draw_foreground(lfsr, image, bx + 70, by + 70);
        self.tremaux.draw_foreground(lfsr, image, bx + 140, by + 70);
        self.rpg.draw_foreground(lfsr, image, bx, by + 140);
        self.time_travel
            .draw_foreground(lfsr, image, bx + 70, by + 140);
        self.clone.draw_foreground(lfsr, image, bx + 140, by + 140);
    }

    fn draw_background(&mut self, image: &mut Image, bx: usize, by: usize) {
        self.random_walk.draw_background(image, bx, by);
        self.random_teleport.draw_background(image, bx + 70, by);
        self.learning.draw_background(image, bx + 140, by);
        self.hold_left.draw_background(image, bx, by + 70);
        self.inverted.draw_background(image, bx + 70, by + 70);
        self.tremaux.draw_background(image, bx + 140, by + 70);
        self.rpg.draw_background(image, bx, by + 140);
        self.time_travel.draw_background(image, bx + 70, by + 140);
        self.clone.draw_background(image, bx + 140, by + 140);
    }

    fn generate(&mut self, lfsr: &mut LFSR) {
        self.random_walk.generate(lfsr);
        self.random_teleport.generate(lfsr);
        self.learning.generate(lfsr);
        self.hold_left.generate(lfsr);
        self.inverted.generate(lfsr);
        self.tremaux.generate(lfsr);
        self.rpg.generate(lfsr);
        self.time_travel.generate(lfsr);
        self.clone.generate(lfsr);
    }
}

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
            pub fn get_solve_count(&mut self) -> Vec<u32> {
                self.0.get_solve_count()
            }

            #[wasm_bindgen]
            pub fn set_upgrades(&mut self, upgrades: u32) {
                self.0.set_upgrades(upgrades);
            }

            #[wasm_bindgen]
            pub fn render(&mut self, buffer: &mut [u8], index: usize, count: usize) {
                self.0.render(buffer, index, count);
            }

            #[wasm_bindgen]
            pub fn tick(&mut self, dt: f32) -> usize {
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
lattice_impl!(RpgLattice, AutoMaze<11, Rpg<11>>);
lattice_impl!(TimeTravelLattice, AutoMaze<13, TimeTravel<13>>);
lattice_impl!(CloneLattice, AutoMaze<20, Clones<20>>);
lattice_impl!(MetaLattice, MetaMaze);
lattice_impl!(DemolitionistLattice, AutoMaze<15, Demolitionist<15>>);
lattice_impl!(FlyingLattice, AutoMaze<15, Flying<15>>);
