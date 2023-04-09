use std::{cmp::Ordering, i8::MIN, mem};

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, INVERTED_PALETTE, PHASE_2_PALETTE},
    solvers::Solver,
    utils::console_log,
};

use super::SolveStatus;

/// Fluid Snail Upgrades:

pub struct Fluid {
    mass: Vec<f32>,
    swap_mass: Vec<f32>,
    upgrades: u32,
}

impl Fluid {
    pub fn new() -> Self {
        Fluid {
            upgrades: 0,
            mass: vec![],
            swap_mass: vec![],
        }
    }
}

const MAX_MASS: f32 = 1.0;
const MAX_COMPRESS: f32 = 0.02;
const MIN_MASS: f32 = 0.0001;
const MIN_FLOW: f32 = 0.001;
const FLOW_SPEED: f32 = 0.5;
const MAX_SPEED: f32 = 10.0;

fn get_stable_state(total_mass: f32) -> f32 {
    if total_mass <= MAX_MASS {
        MAX_MASS
    } else if total_mass < 2.0 * MAX_MASS + MAX_COMPRESS {
        return (MAX_MASS * MAX_MASS + total_mass * MAX_COMPRESS) / (MAX_MASS + MAX_COMPRESS);
    } else {
        return (total_mass + MAX_COMPRESS) / 2.0;
    }
}

impl Solver for Fluid {
    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        maze: &Maze,
        _lfsr: &mut LFSR,
        image: &mut Image,
    ) {
        let w = maze.size * 10 + 5;

        let max = *self
            .mass
            .iter()
            .max_by(|a, b| {
                if a.is_nan() {
                    Ordering::Less
                } else if b.is_nan() {
                    Ordering::Greater
                } else {
                    a.partial_cmp(b).unwrap()
                }
            })
            .unwrap();

        for x in 0..(maze.size * 10 + 1) {
            for y in 0..(maze.size * 10 + 1) {
                let cell = self.mass[(y + 2) * w + (x + 2)];
                if cell >= MIN_MASS {
                    image.draw_pixel_xy(
                        [
                            0x00,
                            0x00,
                            (0xff as f32 * (cell.log2() / max.log2()).max(0.3)) as u8,
                        ],
                        x,
                        y,
                    );
                }

                // if cell.is_nan() {
                //     image.draw_pixel_xy([0xff, 0xff, 0xff], x, y);
                // }
            }
        }
    }

    fn setup(&mut self, maze: &Maze, _lfsr: &mut LFSR) {
        // 2 padding spaces on each side
        let w = (maze.size * 10) + 5;

        self.mass.fill(0.0);
        self.mass.resize(w * w, 0.0);
        self.swap_mass.resize(w * w, 0.0);

        // draw walls to our buffer
        for y in 0..maze.size {
            for x in 0..maze.size {
                let cell = maze.get_cell(x, y);

                if cell.has_wall(Direction::Up) {
                    for i in 0..11 {
                        self.mass[(y * 10 + 2) * w + (x * 10 + 2 + i)] = f32::NAN;
                    }
                }

                if cell.has_wall(Direction::Down) {
                    for i in 0..11 {
                        self.mass[(y * 10 + 12) * w + (x * 10 + 2 + i)] = f32::NAN;
                    }
                }

                if cell.has_wall(Direction::Left) {
                    for i in 0..11 {
                        self.mass[(y * 10 + 2 + i) * w + (x * 10 + 2)] = f32::NAN;
                    }
                }

                if cell.has_wall(Direction::Right) {
                    for i in 0..11 {
                        self.mass[(y * 10 + 2 + i) * w + (x * 10 + 12)] = f32::NAN;
                    }
                }
            }
        }

        self.swap_mass.copy_from_slice(&self.mass);
    }

    // https://w-shadow.com/blog/2009/09/01/simple-fluid-simulation/
    fn step(&mut self, maze: &mut Maze, lfsr: &mut LFSR) -> SolveStatus {
        let s = maze.size * 10 + 1;
        let w = s + 4;

        for _ in 0..10 {
            // spawn water
            self.mass[5 * w + 6] += 2.0;
            self.mass[5 * w + 7] += 2.0;
            self.mass[5 * w + 8] += 2.0;
            self.swap_mass[5 * w + 6] += 2.0;
            self.swap_mass[5 * w + 7] += 2.0;
            self.swap_mass[5 * w + 8] += 2.0;

            // console_log!("2: {}", self.mass[5 * w + 8]);

            for y in (2..(s + 2)) {
                for x in 2..(s + 3) {
                    if self.mass[y * w + x].is_nan() {
                        continue;
                    }

                    let mut flow = 0.0;
                    let mut remaining_mass = self.mass[y * w + x];

                    if remaining_mass <= MIN_MASS {
                        // self.swap_mass[y * w + x] -= remaining_mass;
                        continue;
                    }

                    // DOWN
                    if !self.mass[(y + 1) * w + x].is_nan() {
                        flow = get_stable_state(remaining_mass + self.mass[(y + 1) * w + x])
                            - self.mass[(y + 1) * w + x];

                        if flow > MIN_FLOW {
                            flow *= FLOW_SPEED;
                        }

                        flow = flow.max(0.0).min(MAX_SPEED.min(remaining_mass));

                        // console_log!(
                        //     "2: {}, {}, {flow}",
                        //     self.swap_mass[5 * w + 8],
                        //     self.mass[5 * w + 8]
                        // );

                        self.swap_mass[y * w + x] -= flow;
                        self.swap_mass[(y + 1) * w + x] += flow;

                        // console_log!("3: {}, {flow}", self.swap_mass[5 * w + 8]);
                        // console_log!("3: {}", self.mass[5 * w + 8]);
                        remaining_mass -= flow;
                    }

                    if remaining_mass <= MIN_MASS {
                        // self.swap_mass[y * w + x] -= remaining_mass;
                        continue;
                    }

                    // LEFT
                    if !self.mass[y * w + x - 1].is_nan() && self.mass[y * w + x - 1] >= 0.0 {
                        // && self.mass[y * w + x - 1] <= 0.0 {
                        flow = (self.mass[y * w + x] - self.mass[y * w + x - 1]) / 2.0;

                        if flow > MIN_FLOW {
                            flow *= FLOW_SPEED;
                        }

                        flow = flow.max(0.0).min(remaining_mass);

                        self.swap_mass[y * w + x] -= flow;
                        self.swap_mass[y * w + x - 1] += flow;

                        // console_log!("4: {}", self.mass[5 * w + 8]);
                        remaining_mass -= flow;
                    }

                    if remaining_mass <= MIN_MASS {
                        self.swap_mass[y * w + x] -= remaining_mass;
                        continue;
                    }

                    // RIGHT
                    if !self.mass[y * w + x + 1].is_nan() && self.mass[y * w + x + 1] >= 0.0 {
                        // && self.mass[y * w + x + 1] <= 0.0 {
                        flow = (self.mass[y * w + x] - self.mass[y * w + x + 1]) / 2.0;

                        if flow > MIN_FLOW {
                            flow *= FLOW_SPEED;
                        }

                        flow = flow.max(0.0).min(remaining_mass);

                        self.swap_mass[y * w + x] -= flow;
                        self.swap_mass[y * w + x + 1] += flow;

                        remaining_mass -= flow;
                    }

                    if remaining_mass <= MIN_MASS {
                        // self.swap_mass[y * w + x] -= remaining_mass;
                        continue;
                    }

                    // UP
                    if !self.mass[(y - 1) * w + x].is_nan() {
                        flow = remaining_mass
                            - get_stable_state(remaining_mass + self.mass[(y - 1) * w + x]);

                        if flow > MIN_FLOW {
                            flow *= FLOW_SPEED;
                        }

                        flow = flow.max(0.0).min(MAX_SPEED.min(remaining_mass));

                        self.swap_mass[y * w + x] -= flow;
                        self.swap_mass[(y - 1) * w + x] += flow;

                        remaining_mass -= flow;
                    }
                }
            }

            self.mass.copy_from_slice(&self.swap_mass);
        }

        let goal_px = maze.size * 10 + 2 - 4;
        if self.mass[goal_px * w + goal_px] > MIN_MASS {
            SolveStatus::Solved(1)
        } else {
            SolveStatus::None
        }
    }

    fn palette(&self) -> [[u8; 3]; 6] {
        PHASE_2_PALETTE
    }

    fn movement_time(&self) -> f32 {
        1000.0 / 60.0 // 30fps simulation
    }
}
