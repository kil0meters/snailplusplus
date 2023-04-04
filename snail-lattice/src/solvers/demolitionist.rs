use std::{collections::HashSet, f32::consts::PI};

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, PHASE_2_PALETTE},
    solvers::Solver,
    utils::Vec2,
};

use super::SolveStatus;

struct Bomb {
    pos: Vec2,
    fuse: usize,
    explosion_progress: usize,
    radius: usize,
    exploded: bool,
}

const EXPLOSION_TIME: usize = 2;

impl Bomb {
    fn new(x: usize, y: usize, fuse_time: usize) -> Bomb {
        Bomb {
            pos: Vec2 { x, y },
            fuse: fuse_time,
            explosion_progress: 0,
            radius: 10,
            exploded: false,
        }
    }

    // whether or not the bomb is finished exploding
    fn step(&mut self) -> bool {
        if self.exploded {
            return true;
        }

        if self.fuse > 0 {
            self.fuse -= 1;
        } else if self.explosion_progress < EXPLOSION_TIME {
            self.explosion_progress += 1;
        } else {
            self.exploded = true;
        }

        false
    }

    fn draw(&self, image: &mut Image, movement_progress: f32) {
        if self.fuse > 0 {
            image.draw_circle(
                [0xff, 0x00, 0x00],
                self.pos.x * 10 + 5,
                self.pos.y * 10 + 5,
                ((movement_progress * PI / 2.0).cos() * 4.0) as i32,
            );
        } else if !self.exploded {
            let exploded_so_far =
                (self.explosion_progress as f32 / EXPLOSION_TIME as f32) * self.radius as f32;
            let explosion_per_time = self.radius as f32 / EXPLOSION_TIME as f32;

            image.draw_circle(
                [0xff, 0x00, 0x00],
                self.pos.x * 10 + 5,
                self.pos.y * 10 + 5,
                (exploded_so_far + (movement_progress * explosion_per_time)) as i32,
            );
        } else {
            image.draw_circle(
                [0xff, 0x00, 0x00],
                self.pos.x * 10 + 5,
                self.pos.y * 10 + 5,
                (self.radius as f32 - (movement_progress * self.radius as f32)) as i32,
            );
        }
    }
}

/// Demolitionist Snail Upgrades:
/// - Lax Regulatilns: More lax regulations allow the Demolitionist Snail to shorten its fuses.
/// - Nitogen Deposit: Place more bombs (5 -> 20) measured 25% throughput improvement
/// - Distructive Habits: Gets a bit faster for each solve. Roughly 65% improvement.

pub struct Demolitionist {
    snail: Snail,
    upgrades: u32,
    solve_sequence: Vec<Direction>,
    bombs: Vec<Bomb>,
    destroyed_squares: Vec<bool>,
    walked_tiles: f32,
}

impl Demolitionist {
    pub fn new() -> Self {
        Demolitionist {
            snail: Snail::new(),
            upgrades: 0,
            solve_sequence: Vec::new(),
            bombs: Vec::new(),
            destroyed_squares: Vec::new(),
            walked_tiles: 0.0,
        }
    }
}

impl Solver for Demolitionist {
    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        _maze: &Maze,
        _lfsr: &mut LFSR,
        image: &mut Image,
    ) {
        let progress = movement_timer / self.movement_time();

        self.snail
            .draw(DEFAULT_PALETTE, animation_cycle, progress, image);

        for bomb in &self.bombs {
            bomb.draw(image, progress);
        }
    }

    fn setup(&mut self, maze: &Maze, lfsr: &mut LFSR) {
        self.bombs.clear();
        self.snail.reset();
        self.destroyed_squares.fill(false);
        self.destroyed_squares.resize(maze.size * maze.size, false);

        self.walked_tiles = 0.0;

        let mut invalid_positions = HashSet::new();

        let bomb_count = if self.upgrades & 0b10 != 0 { 20 } else { 5 };
        let fuse_time = if self.upgrades & 0b1 != 0 { 5 } else { 10 };

        // generate some random enemies in random locations
        for _ in 0..bomb_count {
            let mut x = lfsr.big() % maze.size;
            let mut y = lfsr.big() % maze.size;

            while invalid_positions.contains(&(x, y)) {
                x = lfsr.big() % maze.size;
                y = lfsr.big() % maze.size;
            }

            invalid_positions.insert((x, y));

            self.bombs.push(Bomb::new(x, y, fuse_time));
        }
    }

    fn step(&mut self, maze: &mut Maze, _lfsr: &mut LFSR) -> SolveStatus {
        if !self.bombs.is_empty() {
            let mut bomb_exploded = false;

            let mut i = 0;
            while i < self.bombs.len() {
                let res = self.bombs[i].step();

                if self.bombs[i].exploded && !res {
                    let pos = self.bombs[i].pos;
                    maze.remove_wall(pos.x, pos.y, Direction::Up);
                    maze.remove_wall(pos.x, pos.y, Direction::Down);
                    maze.remove_wall(pos.x, pos.y, Direction::Left);
                    maze.remove_wall(pos.x, pos.y, Direction::Right);
                    bomb_exploded = true;

                    self.destroyed_squares[pos.y * maze.size + pos.x] = true;
                }

                if res {
                    self.bombs.swap_remove(i);
                } else {
                    i += 1;
                }
            }

            if bomb_exploded {
                SolveStatus::Rerender
            } else {
                SolveStatus::None
            }
        } else {
            if self.solve_sequence.is_empty() {
                self.solve_sequence =
                    maze.get_solve_sequence(self.snail.pos.x, self.snail.pos.y, maze.end_pos);
                self.solve_sequence.reverse();
            }

            self.snail.direction = self.solve_sequence.pop().unwrap();
            self.snail.move_forward(maze);

            if self.destroyed_squares[self.snail.pos.y * maze.size + self.snail.pos.x] {
                self.walked_tiles += 1.0;
            }

            if self.snail.pos == maze.end_pos {
                SolveStatus::Solved(1)
            } else {
                SolveStatus::None
            }
        }
    }

    fn palette(&self) -> [[u8; 3]; 6] {
        PHASE_2_PALETTE
    }

    fn movement_time(&self) -> f32 {
        if self.upgrades & 0b100 != 0 {
            (SNAIL_MOVEMENT_TIME - (50.0 * self.walked_tiles)).max(10.0)
        } else {
            SNAIL_MOVEMENT_TIME
        }
    }
}
