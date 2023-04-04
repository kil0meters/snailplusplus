use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
};

use super::SolveStatus;

/// Random Walk Snail Upgrades:
/// - Four Leaf Clover: Gives 10% chance to go the right way
/// - Rabbit's Foot:    Gives an additional 20% to go the right way
/// - Horseshoe:        Gives an additional 30% to go the right way

pub struct RandomWalk {
    snail: Snail,
    directions: Vec<Option<Direction>>,
    upgrades: u32,
}

impl RandomWalk {
    pub fn new() -> Self {
        RandomWalk {
            snail: Snail::new(),
            directions: vec![],
            upgrades: 0,
        }
    }
}

impl Solver for RandomWalk {
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
        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer / self.movement_time(),
            image,
        );
    }

    fn setup(&mut self, maze: &Maze, _lfsr: &mut LFSR) {
        self.snail.reset();
        maze.get_directions(maze.end_pos, &mut self.directions);
    }

    fn step(&mut self, maze: &mut Maze, lfsr: &mut LFSR) -> SolveStatus {
        // chance to move in the right direction based on the upgrades provided
        let chance = (self.upgrades & 0b1)
            + (self.upgrades & 0b10)
            + ((self.upgrades & 0b100) >> 1)
            + ((self.upgrades & 0b100) >> 2);

        if (lfsr.big() % 10) < chance as usize {
            self.snail.direction =
                self.directions[self.snail.pos.y * maze.size + self.snail.pos.x].unwrap();
            self.snail.move_forward(maze);
        } else {
            loop {
                match lfsr.next() {
                    0 => self.snail.direction = Direction::Up,
                    1 => self.snail.direction = Direction::Down,
                    2 => self.snail.direction = Direction::Left,
                    3 => self.snail.direction = Direction::Right,
                    _ => unreachable!(),
                }

                if self.snail.move_forward(maze) {
                    break;
                }
            }
        }

        if self.snail.pos == maze.end_pos {
            SolveStatus::Solved(1)
        } else {
            SolveStatus::None
        }
    }

    fn movement_time(&self) -> f32 {
        SNAIL_MOVEMENT_TIME
    }
}
