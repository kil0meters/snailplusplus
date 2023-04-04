use std::collections::{BTreeMap, BTreeSet};

use wasm_bindgen::prelude::*;

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::AutoMaze,
    solvers::{
        Automaton, Clones, Demolitionist, Flying, HoldLeft, Inverted, Learning, RandomTeleport,
        RandomWalk, Rpg, SolveStatus, Telepathic, TimeTravel, Tremaux,
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
    fn size(&self) -> usize;
    fn tick(&mut self, dt: f32, lfsr: &mut LFSR) -> SolveStatus;
    fn set_upgrades(&mut self, upgrades: u32);
    fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image);
    fn draw_background(&mut self, image: &mut Image);
    fn generate(&mut self, lfsr: &mut LFSR);
}

pub struct SnailLattice {
    width: usize,
    mazes: Vec<Box<dyn TilableMaze>>,
    lfsr: LFSR,
    upgrades: u32,
    maze_size: usize,

    // stores the number of mazes solved by a given maze since the last query
    solve_count: Vec<u32>,

    // assumes non-overlapping ranges, and assumes maxes out the index at 2^16.
    // should be fine for now. if not we can always change to a tuple later
    // we're also always going to be dealing with a very small amount of buffers so using a
    // b trees is more efficient than hashmaps here
    bg_buffers: BTreeMap<usize, Vec<u8>>,
    render_marked: BTreeSet<usize>,
}

impl SnailLattice {
    pub fn new(maze_size: usize, width: usize, seed: u16) -> SnailLattice {
        set_panic_hook();

        SnailLattice {
            width,
            maze_size,
            upgrades: 0,
            mazes: Vec::new(),
            lfsr: LFSR::new(seed),
            solve_count: Vec::new(),
            bg_buffers: BTreeMap::new(),
            render_marked: BTreeSet::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.mazes.len()
    }

    pub fn get_dimensions(&self, count: usize) -> Vec<usize> {
        // ceiling division -> count / width
        let height = (count + self.width - 1) / self.width;

        let height_px = (self.maze_size * 10 + 1) * height;
        let width_px = (self.maze_size * 10 + 1) * self.width;

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

        let maze_size_px = self.maze_size * 10 + 1;

        let bg_buffer = match self.bg_buffers.get_mut(&((index << 16) + count)) {
            Some(buffer) => {
                let mut bg_image = Image::new(buffer, dimensions[0], dimensions[1]);

                let indexes = self
                    .render_marked
                    .range(index..(index + count))
                    .cloned()
                    .collect::<Vec<_>>();

                for i in indexes {
                    bg_image.set_offset(
                        maze_size_px * ((i - index) % self.width),
                        maze_size_px * ((i - index) / self.width),
                    );
                    self.mazes[i].draw_background(&mut bg_image);

                    self.render_marked.remove(&i);
                }

                buffer
            }
            None => {
                let mut bg_buffer = vec![0; buffer_size];

                let mut bg_image = Image::new(&mut bg_buffer, dimensions[0], dimensions[1]);

                for (i, maze) in self.mazes.iter_mut().skip(index).take(count).enumerate() {
                    bg_image.set_offset(
                        maze_size_px * (i % self.width),
                        maze_size_px * (i / self.width),
                    );
                    maze.draw_background(&mut bg_image);
                }

                self.bg_buffers.insert((index << 16) + count, bg_buffer);
                self.bg_buffers.get_mut(&((index << 16) + count)).unwrap()
            }
        };

        buffer.copy_from_slice(bg_buffer);

        let mut cx = 0;
        let mut cy = 0;
        let mut image = Image::new(buffer, dimensions[0], dimensions[1]);

        for maze in self.mazes.iter_mut().skip(index).take(count) {
            image.set_offset(cx, cy);
            maze.draw_foreground(&mut self.lfsr, &mut image);

            cx += maze_size_px;
            if cx >= dimensions[0] {
                cx = 0;
                cy += maze_size_px;
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
                    self.solve_count[i] += count as u32;
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

    pub fn alter(&mut self, difference: i32, new_maze_fn: fn(usize) -> Box<dyn TilableMaze>) {
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
                let mut new_maze = new_maze_fn(self.maze_size);
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
    size: usize,

    random_walk: AutoMaze,
    random_teleport: AutoMaze,
    learning: AutoMaze,
    hold_left: AutoMaze,
    inverted: AutoMaze,
    tremaux: AutoMaze,
    time_travel: AutoMaze,
    clone: AutoMaze,
    rpg: AutoMaze,
}

impl MetaMaze {
    fn new(size: usize) -> Self {
        let size = size / 3;

        MetaMaze {
            size,

            random_walk: AutoMaze::new(Box::new(RandomWalk::new()), size),
            random_teleport: AutoMaze::new(Box::new(RandomTeleport::new()), size),
            learning: AutoMaze::new(Box::new(Learning::new()), size),
            hold_left: AutoMaze::new(Box::new(HoldLeft::new()), size),
            inverted: AutoMaze::new(Box::new(Inverted::new()), size),
            tremaux: AutoMaze::new(Box::new(Tremaux::new()), size),
            time_travel: AutoMaze::new(Box::new(TimeTravel::new()), size),
            clone: AutoMaze::new(Box::new(Clones::new()), size),
            rpg: AutoMaze::new(Box::new(Rpg::new()), size),
        }
    }
}

impl TilableMaze for MetaMaze {
    fn size(&self) -> usize {
        self.size * 3
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

    fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image) {
        let bx = image.bx;
        let by = image.by;
        let size_px = self.size * 10;

        self.random_walk.draw_background(image);

        image.set_offset(bx + size_px, by);
        self.random_teleport.draw_foreground(lfsr, image);

        image.set_offset(bx + 2 * size_px, by);
        self.learning.draw_foreground(lfsr, image);

        image.set_offset(bx, by + size_px);
        self.hold_left.draw_foreground(lfsr, image);

        image.set_offset(bx + size_px, by + size_px);
        self.inverted.draw_foreground(lfsr, image);

        image.set_offset(bx + 2 * size_px, by + size_px);
        self.tremaux.draw_foreground(lfsr, image);

        image.set_offset(bx, by + 2 * size_px);
        self.rpg.draw_foreground(lfsr, image);

        image.set_offset(bx + size_px, by + 2 * size_px);
        self.time_travel.draw_foreground(lfsr, image);

        image.set_offset(bx + 2 * size_px, by + 2 * size_px);
        self.clone.draw_foreground(lfsr, image);

        image.set_offset(bx, by);
    }

    fn draw_background(&mut self, image: &mut Image) {
        let bx = image.bx;
        let by = image.by;
        let size_px = self.size * 10;

        self.random_walk.draw_background(image);

        image.set_offset(bx + size_px, by);
        self.random_teleport.draw_background(image);

        image.set_offset(bx + 2 * size_px, by);
        self.learning.draw_background(image);

        image.set_offset(bx, by + size_px);
        self.hold_left.draw_background(image);

        image.set_offset(bx + size_px, by + size_px);
        self.inverted.draw_background(image);

        image.set_offset(bx + 2 * size_px, by + size_px);
        self.tremaux.draw_background(image);

        image.set_offset(bx, by + 2 * size_px);
        self.rpg.draw_background(image);

        image.set_offset(bx + size_px, by + 2 * size_px);
        self.time_travel.draw_background(image);

        image.set_offset(bx + 2 * size_px, by + 2 * size_px);
        self.clone.draw_background(image);

        image.set_offset(bx, by);
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

enum LatticeType {
    RandomWalk,
    RandomTeleport,
    Learning,
    HoldLeft,
    Inverted,
    Tremaux,
    Rpg,
    TimeTravel,
    Clones,
    Meta,
    Demolitionist,
    Telepathic,
    Flying,
    Automaton,
}

#[wasm_bindgen]
pub struct WasmLattice {
    lattice: SnailLattice,
    lattice_type: LatticeType,
}

#[wasm_bindgen]
impl WasmLattice {
    #[wasm_bindgen(constructor)]
    pub fn new(shop_key: &str, seed: u16) -> Self {
        // see: ../../src/ShopProvider.tsx
        let (lattice_type, width, size) = match shop_key {
            "random-walk" => (LatticeType::RandomWalk, 4, 5),
            "random-teleport" => (LatticeType::RandomTeleport, 3, 7),
            "learning" => (LatticeType::Learning, 3, 9),
            "hold-left" => (LatticeType::HoldLeft, 3, 9),
            "inverted" => (LatticeType::Inverted, 3, 9),
            "tremaux" => (LatticeType::Tremaux, 2, 11),
            "rpg" => (LatticeType::Rpg, 2, 11),
            "time-travel" => (LatticeType::TimeTravel, 2, 13),
            "clone" => (LatticeType::Clones, 2, 20),
            "meta" => (LatticeType::Meta, 2, 21),
            "demolitionist" => (LatticeType::Demolitionist, 2, 15),
            "flying" => (LatticeType::Flying, 2, 15),
            "telepathic" => (LatticeType::Telepathic, 2, 11),
            "automaton" => (LatticeType::Automaton, 2, 20),
            _ => unreachable!(),
        };

        Self {
            lattice: SnailLattice::new(size, width, seed),
            lattice_type,
        }
    }

    #[wasm_bindgen]
    pub fn get_dimensions(&self, count: usize) -> Vec<usize> {
        self.lattice.get_dimensions(count)
    }

    #[wasm_bindgen]
    pub fn get_solve_count(&mut self) -> Vec<u32> {
        self.lattice.get_solve_count()
    }

    #[wasm_bindgen]
    pub fn set_upgrades(&mut self, upgrades: u32) {
        self.lattice.set_upgrades(upgrades);
    }

    #[wasm_bindgen]
    pub fn render(&mut self, buffer: &mut [u8], index: usize, count: usize) {
        self.lattice.render(buffer, index, count);
    }

    #[wasm_bindgen]
    pub fn tick(&mut self, dt: f32) -> usize {
        self.lattice.tick(dt)
    }

    #[wasm_bindgen]
    pub fn alter(&mut self, difference: i32) {
        macro_rules! lattice_type_match {
            ($($name:tt),+) => (
                match self.lattice_type {
                    $(
                        LatticeType::$name => self.lattice.alter(difference, |s| {
                            Box::new(AutoMaze::new(Box::new($name::new()), s))
                        }),
                    )+

                    LatticeType::Meta => self.lattice.alter(difference, |s| {
                        Box::new(MetaMaze::new(s))
                    }),
                }
            );
        }

        lattice_type_match!(
            RandomWalk,
            RandomTeleport,
            Learning,
            HoldLeft,
            Inverted,
            Tremaux,
            Rpg,
            TimeTravel,
            Clones,
            Demolitionist,
            Flying,
            Telepathic,
            Automaton
        );
    }

    #[wasm_bindgen]
    pub fn count(&self) -> usize {
        self.lattice.mazes.len()
    }

    #[wasm_bindgen]
    pub fn get_width(&self) -> usize {
        self.lattice.width
    }

    #[wasm_bindgen]
    pub fn set_width(&mut self, width: usize) {
        self.lattice.set_width(width);
    }
}
