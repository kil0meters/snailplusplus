use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    image::Image,
    lfsr::LFSR,
    maze::AutoMaze,
    solvers::{RandomTeleport, RandomWalk, SolveStatus},
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

pub trait TilableMaze {
    fn size(&self) -> usize;
    fn tick(&mut self, dt: f32, lfsr: &mut LFSR) -> SolveStatus;
    fn set_upgrades(&mut self, upgrades: u32);
    fn generate_mesh(&self) -> MazeMesh;
    fn generate(&mut self, lfsr: &mut LFSR);
}

pub struct SnailLattice {
    mazes: Vec<Box<dyn TilableMaze>>,
    lfsr: LFSR,
    upgrades: u32,
    maze_size: usize,

    // stores the number of mazes solved by a given maze since the last query
    solve_count: Vec<u32>,

    // stores the mazes which need to have their maze meshes regenerated
    needs_new_mesh: Vec<bool>,
}

impl SnailLattice {
    pub fn new(maze_size: usize, seed: u16) -> SnailLattice {
        set_panic_hook();

        SnailLattice {
            maze_size,
            upgrades: 0,
            mazes: Vec::new(),
            lfsr: LFSR::new(seed),
            solve_count: Vec::new(),
            needs_new_mesh: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.mazes.len()
    }

    pub fn render(&mut self, targets: Vec<usize>) -> Vec<u8> {
        return vec![];
    }

    pub fn get_meshes(&mut self, targets: Vec<usize>) -> Vec<MazeMesh> {
        let mut meshes = Vec::new();

        for i in targets {
            if (self.needs_new_mesh[i]) {
                let mut new_mesh = self.mazes[i].generate_mesh();
                new_mesh.id = i;
                meshes.push(new_mesh);

                self.needs_new_mesh[i] = false;
            }
        }

        meshes
    }

    pub fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
        for maze in &mut self.mazes {
            maze.set_upgrades(self.upgrades);
        }
    }

    pub fn get_solve_count(&mut self) -> Vec<u32> {
        let solve_count_clone = self.solve_count.clone();

        for val in self.solve_count.iter_mut() {
            *val = 0;
        }

        solve_count_clone
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
                    self.needs_new_mesh[i] = true;
                }
                SolveStatus::Rerender => {
                    self.needs_new_mesh[i] = true;
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
                self.needs_new_mesh.pop();
            }
        } else {
            let mut time_offset = 0.0;

            for _ in 0..difference {
                // console_log!("hello world");

                let mut new_maze = new_maze_fn(self.maze_size);
                new_maze.set_upgrades(self.upgrades);
                new_maze.generate(&mut self.lfsr);

                // offset time slightly
                // new_maze.tick(time_offset, &mut self.lfsr);

                self.mazes.push(new_maze);
                self.solve_count.push(0);
                self.needs_new_mesh.push(true);

                time_offset += 100.0;
            }
        }
    }
}

// #[wasm_bindgen]
// pub struct MetaMaze {
//     size: usize,
//
//     random_walk: AutoMaze,
//     random_teleport: AutoMaze,
//     learning: AutoMaze,
//     hold_left: AutoMaze,
//     inverted: AutoMaze,
//     tremaux: AutoMaze,
//     time_travel: AutoMaze,
//     clone: AutoMaze,
//     rpg: AutoMaze,
// }
//
// impl MetaMaze {
//     fn new(size: usize) -> Self {
//         let size = size / 3;
//
//         MetaMaze {
//             size,
//
//             random_walk: AutoMaze::new(Box::new(RandomWalk::new()), size),
//             random_teleport: AutoMaze::new(Box::new(RandomTeleport::new()), size),
//             learning: AutoMaze::new(Box::new(Learning::new()), size),
//             hold_left: AutoMaze::new(Box::new(HoldLeft::new()), size),
//             inverted: AutoMaze::new(Box::new(Inverted::new()), size),
//             tremaux: AutoMaze::new(Box::new(Tremaux::new()), size),
//             time_travel: AutoMaze::new(Box::new(TimeTravel::new()), size),
//             clone: AutoMaze::new(Box::new(Clones::new()), size),
//             rpg: AutoMaze::new(Box::new(Rpg::new()), size),
//         }
//     }
// }
//
// impl TilableMaze for MetaMaze {
//     fn size(&self) -> usize {
//         self.size * 3
//     }
//
//     fn set_upgrades(&mut self, upgrades: u32) {
//         self.random_walk.set_upgrades(upgrades & 0b111);
//         self.random_teleport.set_upgrades((upgrades >> 3) & 0b111);
//         self.learning.set_upgrades((upgrades >> 6) & 0b111);
//         self.hold_left.set_upgrades((upgrades >> 9) & 0b111);
//         self.inverted.set_upgrades((upgrades >> 12) & 0b111);
//         self.tremaux.set_upgrades((upgrades >> 15) & 0b111);
//         self.rpg.set_upgrades((upgrades >> 18) & 0b111);
//         self.time_travel.set_upgrades((upgrades >> 21) & 0b111);
//         self.clone.set_upgrades((upgrades >> 24) & 0b111);
//     }
//
//     fn tick(&mut self, dt: f32, lfsr: &mut LFSR) -> SolveStatus {
//         let mut total = 0;
//
//         total += self.random_walk.tick(dt, lfsr).get_count();
//         total += self.random_teleport.tick(dt, lfsr).get_count();
//         total += self.learning.tick(dt, lfsr).get_count();
//         total += self.hold_left.tick(dt, lfsr).get_count();
//         total += self.inverted.tick(dt, lfsr).get_count();
//         total += self.tremaux.tick(dt, lfsr).get_count();
//         total += self.time_travel.tick(dt, lfsr).get_count();
//         total += self.clone.tick(dt, lfsr).get_count();
//         total += self.rpg.tick(dt, lfsr).get_count();
//
//         if total > 0 {
//             SolveStatus::Solved(total)
//         } else {
//             SolveStatus::None
//         }
//     }
//
//     fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image) {
//         let bx = image.bx;
//         let by = image.by;
//         let size_px = self.size * 10;
//
//         self.random_walk.draw_background(image);
//
//         image.set_offset(bx + size_px, by);
//         self.random_teleport.draw_foreground(lfsr, image);
//
//         image.set_offset(bx + 2 * size_px, by);
//         self.learning.draw_foreground(lfsr, image);
//
//         image.set_offset(bx, by + size_px);
//         self.hold_left.draw_foreground(lfsr, image);
//
//         image.set_offset(bx + size_px, by + size_px);
//         self.inverted.draw_foreground(lfsr, image);
//
//         image.set_offset(bx + 2 * size_px, by + size_px);
//         self.tremaux.draw_foreground(lfsr, image);
//
//         image.set_offset(bx, by + 2 * size_px);
//         self.rpg.draw_foreground(lfsr, image);
//
//         image.set_offset(bx + size_px, by + 2 * size_px);
//         self.time_travel.draw_foreground(lfsr, image);
//
//         image.set_offset(bx + 2 * size_px, by + 2 * size_px);
//         self.clone.draw_foreground(lfsr, image);
//
//         image.set_offset(bx, by);
//     }
//
//     fn draw_background(&mut self, image: &mut Image) {
//         let bx = image.bx;
//         let by = image.by;
//         let size_px = self.size * 10;
//
//         self.random_walk.draw_background(image);
//
//         image.set_offset(bx + size_px, by);
//         self.random_teleport.draw_background(image);
//
//         image.set_offset(bx + 2 * size_px, by);
//         self.learning.draw_background(image);
//
//         image.set_offset(bx, by + size_px);
//         self.hold_left.draw_background(image);
//
//         image.set_offset(bx + size_px, by + size_px);
//         self.inverted.draw_background(image);
//
//         image.set_offset(bx + 2 * size_px, by + size_px);
//         self.tremaux.draw_background(image);
//
//         image.set_offset(bx, by + 2 * size_px);
//         self.rpg.draw_background(image);
//
//         image.set_offset(bx + size_px, by + 2 * size_px);
//         self.time_travel.draw_background(image);
//
//         image.set_offset(bx + 2 * size_px, by + 2 * size_px);
//         self.clone.draw_background(image);
//
//         image.set_offset(bx, by);
//     }
//
//     fn generate(&mut self, lfsr: &mut LFSR) {
//         self.random_walk.generate(lfsr);
//         self.random_teleport.generate(lfsr);
//         self.learning.generate(lfsr);
//         self.hold_left.generate(lfsr);
//         self.inverted.generate(lfsr);
//         self.tremaux.generate(lfsr);
//         self.rpg.generate(lfsr);
//         self.time_travel.generate(lfsr);
//         self.clone.generate(lfsr);
//     }
// }

enum LatticeType {
    RandomWalk,
    RandomTeleport,
    // Learning,
    // HoldLeft,
    // Inverted,
    // Tremaux,
    // Rpg,
    // TimeTravel,
    // Clones,
    // Meta,
    // Demolitionist,
    // Telepathic,
    // Flying,
    // Automaton,
    // Fluid,
}

#[derive(Serialize, Deserialize)]
pub struct MazeMesh {
    pub id: usize,
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
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
            // "learning" => (LatticeType::Learning, 3, 9),
            // "hold-left" => (LatticeType::HoldLeft, 3, 9),
            // "inverted" => (LatticeType::Inverted, 3, 9),
            // "tremaux" => (LatticeType::Tremaux, 2, 11),
            // "rpg" => (LatticeType::Rpg, 2, 11),
            // "time-travel" => (LatticeType::TimeTravel, 2, 13),
            // "clone" => (LatticeType::Clones, 2, 20),
            // "meta" => (LatticeType::Meta, 2, 21),
            // "demolitionist" => (LatticeType::Demolitionist, 2, 15),
            // "flying" => (LatticeType::Flying, 2, 15),
            // "telepathic" => (LatticeType::Telepathic, 2, 11),
            // "automaton" => (LatticeType::Automaton, 2, 20),
            // "fluid" => (LatticeType::Fluid, 2, 10),
            _ => unreachable!(),
        };

        Self {
            lattice: SnailLattice::new(size, seed),
            lattice_type,
        }
    }

    // using JsValue is probably very slow, but it might not matter
    #[wasm_bindgen]
    pub fn get_meshes(&mut self, visible: Vec<usize>) -> JsValue {
        let meshes = self.lattice.get_meshes(visible);
        serde_wasm_bindgen::to_value(&meshes).unwrap()
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

                    // LatticeType::Meta => self.lattice.alter(difference, |s| {
                    //     Box::new(MetaMaze::new(s))
                    // }),
                }
            );
        }

        lattice_type_match!(
            RandomWalk,
            RandomTeleport // Learning,
                           // HoldLeft,
                           // Inverted,
                           // Tremaux,
                           // Rpg,
                           // TimeTravel,
                           // Clones,
                           // Demolitionist,
                           // Flying,
                           // Telepathic,
                           // Automaton,
                           // Fluid
        );
    }

    #[wasm_bindgen]
    pub fn count(&self) -> usize {
        self.lattice.mazes.len()
    }
}
