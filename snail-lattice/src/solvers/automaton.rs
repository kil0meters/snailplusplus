use std::mem;

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE, PHASE_2_PALETTE},
    solvers::Solver,
};

use super::SolveStatus;

const MAX_TIMEOUT: usize = 100;

/// Automaton Snail Upgrades:
/// - High Speed Connectivity: Automaton Snail installs a new 5G radio tower nearby to allow for faster communication between cells.
/// - Algorithmic Improvement: Automaton Snail changes its replication method to one that's more effective.

pub struct Automaton<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    upgrades: u32,
    grid: [u8; S * S],
    swap_grid: [u8; S * S],

    spawned_count: usize,
    timeout: usize,
}

impl<const S: usize> Automaton<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn neighbor_count(&self, dx: usize, dy: usize) -> usize {
        let mut neighbor_count = 0;

        for_each_neighbor::<S>(dx as i32, dy as i32, |x, y| {
            if self.grid[y * S + x] & 1 != 0 {
                neighbor_count += 1;
            }
        });

        neighbor_count
    }
}

impl<const S: usize> Solver<S> for Automaton<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        Automaton {
            grid: [0; S * S],
            swap_grid: [0; S * S],
            upgrades: 0,
            spawned_count: 0,
            timeout: 0,
        }
    }

    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        _movement_timer: f32,
        _lfsr: &mut LFSR,
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        for y in 0..S {
            for x in 0..S {
                let cell = self.grid[y * S + x];

                if cell != 0 {
                    let dir = Direction::from_number((cell >> 6) as usize);
                    let palette = match cell & 0b11 {
                        1 => DEFAULT_PALETTE,
                        2 => GRAYSCALE_PALETTE,
                        _ => unreachable!(),
                    };

                    image.draw_snail(palette, animation_cycle, dir, x * 10 + bx, y * 10 + by);
                }
            }
        }
    }

    fn setup(&mut self, _maze: &Maze<S>, lfsr: &mut LFSR) {
        self.timeout = 0;
        self.spawned_count = 0;

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
    }

    fn step(&mut self, _maze: &mut Maze<S>, lfsr: &mut LFSR) -> SolveStatus {
        // conway's game of life
        if self.upgrades & 0b10 != 0 {
            for y in 0..S {
                for x in 0..S {
                    let cell = self.grid[y * S + x];

                    // alive cells
                    if cell & 0b11 != 0 {
                        match self.neighbor_count(x, y) {
                            c if c < 2 => self.swap_grid[y * S + x] = 0,
                            c if c > 3 => self.swap_grid[y * S + x] = 0,
                            _ => self.swap_grid[y * S + x] = ((lfsr.next() << 6) | 1) as u8,
                        }
                    }
                    // dead cells
                    else {
                        if self.neighbor_count(x, y) == 3 {
                            self.spawned_count += 1;
                            self.swap_grid[y * S + x] = ((lfsr.next() << 6) | 1) as u8;
                        } else {
                            self.swap_grid[y * S + x] = 0;
                        }
                    }
                }
            }
        }
        // brian's brain
        else {
            for y in 0..S {
                for x in 0..S {
                    let cell = self.grid[y * S + x];

                    // alive cells
                    if cell & 0b11 == 1 {
                        self.swap_grid[y * S + x] = (cell & 0b11000000) | 2;
                    }
                    // dying cell
                    else if cell & 0b11 == 2 {
                        self.swap_grid[y * S + x] = 0;
                    }
                    // dead cell
                    else {
                        if self.neighbor_count(x, y) == 2 {
                            self.spawned_count += 1;
                            self.swap_grid[y * S + x] = ((lfsr.next() << 6) | 1) as u8;
                        } else {
                            self.swap_grid[y * S + x] = 0;
                        }
                    }
                }
            }
        }

        mem::swap(&mut self.grid, &mut self.swap_grid);

        self.timeout += 1;
        if self.timeout == MAX_TIMEOUT {
            return SolveStatus::Solved(self.spawned_count);
        }

        SolveStatus::None
    }

    fn custom_goal() -> bool {
        true
    }

    fn palette() -> [[u8; 3]; 6] {
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

fn for_each_neighbor<const S: usize>(x: i32, y: i32, mut callbackfn: impl FnMut(usize, usize)) {
    callbackfn((x - 1).rem_euclid(S as i32) as usize, y as usize);
    callbackfn(x as usize, (y - 1).rem_euclid(S as i32) as usize);
    callbackfn((x + 1).rem_euclid(S as i32) as usize, y as usize);
    callbackfn(x as usize, (y + 1).rem_euclid(S as i32) as usize);
    callbackfn(
        (x - 1).rem_euclid(S as i32) as usize,
        (y - 1).rem_euclid(S as i32) as usize,
    );
    callbackfn(
        (x + 1).rem_euclid(S as i32) as usize,
        (y - 1).rem_euclid(S as i32) as usize,
    );
    callbackfn(
        (x + 1).rem_euclid(S as i32) as usize,
        (y + 1).rem_euclid(S as i32) as usize,
    );
    callbackfn(
        (x - 1).rem_euclid(S as i32) as usize,
        (y + 1).rem_euclid(S as i32) as usize,
    );
}
