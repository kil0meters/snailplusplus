use std::mem;

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{DEFAULT_PALETTE, GRAYSCALE_PALETTE, PHASE_2_PALETTE},
    solvers::Solver,
};

use super::SolveStatus;

const MAX_TIMEOUT: usize = 100;

/// Automaton Snail Upgrades:
/// - High Speed Connectivity: Automaton Snail installs a new 5G radio tower nearby to allow for faster communication between cells.
/// - Algorithmic Improvement: Automaton Snail changes its replication method to one that's more effective.

pub struct Automaton {
    upgrades: u32,
    grid: Vec<u8>,
    swap_grid: Vec<u8>,

    size: usize,
    spawned_count: usize,
    timeout: usize,
}

impl Automaton {
    pub fn new() -> Self {
        Automaton {
            grid: vec![],
            swap_grid: vec![],
            size: 0,
            upgrades: 0,
            spawned_count: 0,
            timeout: 0,
        }
    }

    #[rustfmt::skip]
    fn neighbor_count(&self, x: i32, y: i32) -> u8 {
        let mut neighbor_count = 0;
        let width = (self.size + 2) as i32;

        // SAFETY: This is guaranteed to be within bounds
        // This is a huge performance win in this case as well, like 25%
        unsafe {
            neighbor_count += self.grid.get_unchecked(((y - 1) * width + (x - 1)) as usize) & 1;
            neighbor_count += self.grid.get_unchecked(((y - 1) * width + x) as usize) & 1;
            neighbor_count += self.grid.get_unchecked(((y - 1) * width + (x + 1)) as usize) & 1;
            neighbor_count += self.grid.get_unchecked((y * width + x - 1) as usize) & 1;
            neighbor_count += self.grid.get_unchecked((y * width + (x + 1)) as usize) & 1;
            neighbor_count += self.grid.get_unchecked(((y + 1) * width + (x - 1)) as usize) & 1;
            neighbor_count += self.grid.get_unchecked(((y + 1) * width + x) as usize) & 1;
            neighbor_count += self.grid.get_unchecked(((y + 1) * width + (x + 1)) as usize) & 1;
        }

        neighbor_count
    }

    fn swap_edges(&mut self) {
        let width = self.size + 2;

        // swap vertical edges
        for x in 0..width {
            self.grid[0 * width + x] = self.grid[(width - 2) * width + x];
            self.grid[(width - 1) * width + x] = self.grid[1 * width + x];
        }

        // swap horizontal edges
        for y in 0..width {
            self.grid[y * width + 0] = self.grid[y * width + (width - 2)];
            self.grid[y * width + (width - 1)] = self.grid[y * width + 1];
        }
    }
}

impl Solver for Automaton {
    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        _movement_timer: f32,
        _maze: &Maze,
        _lfsr: &mut LFSR,
        image: &mut Image,
    ) {
        let width = self.size + 2;

        for y in 0..self.size {
            for x in 0..self.size {
                let cell = self.grid[(y + 1) * width + (x + 1)];

                if cell != 0 {
                    let dir = Direction::from_number((cell >> 6) as usize);
                    let palette = match cell & 0b11 {
                        1 => DEFAULT_PALETTE,
                        2 => GRAYSCALE_PALETTE,
                        _ => unreachable!(),
                    };

                    image.draw_snail(palette, animation_cycle, dir, x * 10, y * 10);
                }
            }
        }
    }

    fn setup(&mut self, maze: &Maze, lfsr: &mut LFSR) {
        self.timeout = 0;
        self.spawned_count = 0;
        self.size = maze.size;

        let size = (self.size + 2) * (self.size * 2);
        self.grid.resize(size, 0);
        self.swap_grid.resize(size, 0);

        if self.upgrades & 0b10 != 0 {
            self.grid.fill_with(|| {
                if lfsr.next() == 1 {
                    ((lfsr.next() << 6) | 1) as u8
                } else {
                    0
                }
            });
        } else {
            self.grid.fill_with(|| {
                if lfsr.next() == 1 {
                    ((lfsr.next() << 6) | ((lfsr.next() & 1) + 1)) as u8
                } else {
                    0
                }
            });
        }

        self.swap_edges();
    }

    fn step(&mut self, _maze: &mut Maze, lfsr: &mut LFSR) -> SolveStatus {
        let width = self.size + 2;

        // conway's game of life
        if self.upgrades & 0b10 != 0 {
            for y in 1..(self.size + 1) {
                for x in 1..(self.size + 1) {
                    let cell = self.grid[y * width + x];

                    // alive cells
                    if cell & 0b11 != 0 {
                        match self.neighbor_count(x as i32, y as i32) {
                            c if c < 2 => self.swap_grid[y * width + x] = 0,
                            c if c > 3 => self.swap_grid[y * width + x] = 0,
                            _ => self.swap_grid[y * width + x] = ((lfsr.next() << 6) | 1) as u8,
                        }
                    }
                    // dead cells
                    else {
                        if self.neighbor_count(x as i32, y as i32) == 3 {
                            self.spawned_count += 1;
                            self.swap_grid[y * width + x] = ((lfsr.next() << 6) | 1) as u8;
                        } else {
                            self.swap_grid[y * width + x] = 0;
                        }
                    }
                }
            }
        }
        // brian's brain
        else {
            for y in 1..(self.size + 1) {
                for x in 1..(self.size + 1) {
                    let cell = self.grid[y * width + x];

                    // alive cells
                    if cell & 0b11 == 1 {
                        self.swap_grid[y * width + x] = (cell & 0b11000000) | 2;
                    }
                    // dying cell
                    else if cell & 0b11 == 2 {
                        self.swap_grid[y * width + x] = 0;
                    }
                    // dead cell
                    else {
                        if self.neighbor_count(x as i32, y as i32) == 2 {
                            self.spawned_count += 1;
                            self.swap_grid[y * width + x] = ((lfsr.next() << 6) | 1) as u8;
                        } else {
                            self.swap_grid[y * width + x] = 0;
                        }
                    }
                }
            }
        }

        mem::swap(&mut self.grid, &mut self.swap_grid);
        self.swap_edges();

        self.timeout += 1;
        if self.timeout == MAX_TIMEOUT {
            return SolveStatus::Solved(self.spawned_count);
        }

        SolveStatus::None
    }

    fn custom_goal(&self) -> bool {
        true
    }

    fn palette(&self) -> [[u8; 3]; 6] {
        PHASE_2_PALETTE
    }

    fn movement_time(&self) -> f32 {
        if self.upgrades & 0b1 != 0 {
            SNAIL_MOVEMENT_TIME / 6.0
        } else {
            SNAIL_MOVEMENT_TIME / 4.0
        }
    }
}
