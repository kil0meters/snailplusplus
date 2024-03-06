use std::collections::HashSet;

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE},
    solvers::Solver,
    utils::Vec2,
};

use super::SolveStatus;

/// RPG Snail Upgrades:
/// - Comradery:   RPG Snail gets a 10% speed boost for each snail in its party.
/// - Sidequests:  Any snail RPG Snail runs into is automatically added to its party.
/// - Recruitment: The snails come to RPG snail on their own.

pub struct Rpg {
    party: Vec<Snail>,
    lost: Vec<Snail>,
    upgrades: u32,
    directions: Vec<Option<Direction>>,

    current_sequence: Vec<Direction>,
}

impl Rpg {
    pub fn new() -> Self {
        Rpg {
            party: Vec::new(),
            lost: Vec::new(),

            directions: Vec::new(),
            current_sequence: Vec::new(),
            upgrades: 0,
        }
    }

    fn generate_lost_snails(&mut self, lfsr: &mut LFSR, size: usize) {
        let mut invalid_positions = HashSet::new();
        invalid_positions.insert((0, 0));

        for _ in 0..(size / 2) {
            let mut x;
            let mut y;

            loop {
                x = lfsr.big() % size;
                y = lfsr.big() % size;

                if !invalid_positions.contains(&(x, y)) {
                    break;
                }
            }

            invalid_positions.insert((x, y));

            let mut new_snail = Snail::new();
            new_snail.pos = Vec2 { x, y };
            new_snail.prev_pos = new_snail.pos;

            self.lost.push(new_snail);
        }
    }
}

impl Solver for Rpg {
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
        for snail in &self.party {
            snail.draw(
                DEFAULT_PALETTE,
                animation_cycle,
                movement_timer / self.movement_time(),
                image,
            );
        }

        for snail in &self.lost {
            snail.draw(
                GRAYSCALE_PALETTE,
                animation_cycle,
                movement_timer / self.movement_time(),
                image,
            );
        }
    }

    fn setup(&mut self, maze: &Maze, lfsr: &mut LFSR) {
        self.lost.clear();
        self.party.clear();

        self.party.push(Snail::new());
        self.generate_lost_snails(lfsr, maze.size);

        if (self.upgrades & 0b100) != 0 {
            maze.get_directions(Vec2 { x: 0, y: 0 }, &mut self.directions);
        }
    }

    fn step(&mut self, maze: &mut Maze, lfsr: &mut LFSR) -> SolveStatus {
        // recruitment
        if (self.upgrades & 0b100) != 0 && !self.lost.is_empty() {
            if !(self.party[0].pos.x == 0 && self.party[0].pos.y == 0) {
                self.setup(maze, lfsr);
                return SolveStatus::None;
            }

            for lost_snail in &mut self.lost {
                lost_snail.direction =
                    match self.directions[lost_snail.pos.y * maze.size + lost_snail.pos.x] {
                        Some(x) => x,
                        None => {
                            self.setup(maze, lfsr);
                            return SolveStatus::None;
                        }
                    };
                lost_snail.move_forward(maze);
            }

            let target_pos = self.party[0].pos;

            // add any lost snails that are in the party
            let new_snails = self.lost.extract_if(|snail| snail.pos == target_pos);

            for mut snail in new_snails {
                snail.prev_pos = target_pos;
                snail.pos = target_pos;
                snail.direction = Direction::Right;
                self.party.push(snail);
            }
        } else {
            if self.current_sequence.is_empty() {
                if let Some(last) = self.lost.last() {
                    self.current_sequence =
                        maze.get_solve_sequence(self.party[0].pos.x, self.party[0].pos.y, last.pos);
                } else {
                    self.current_sequence = maze.get_solve_sequence(
                        self.party[0].pos.x,
                        self.party[0].pos.y,
                        maze.end_pos,
                    );
                }

                self.current_sequence.reverse();
            }

            match self.current_sequence.pop() {
                Some(mut next_move) => {
                    let tmp = next_move;
                    next_move = self.party[0].direction;
                    self.party[0].direction = tmp;
                    self.party[0].move_forward(maze);
                    let mut next_pos = self.party[0].prev_pos;

                    for follower in self.party.iter_mut().skip(1) {
                        let tmp = follower.direction;
                        follower.direction = next_move;
                        follower.prev_pos = follower.pos;
                        follower.pos = next_pos;
                        next_pos = follower.prev_pos;
                        next_move = tmp;
                    }

                    if self.current_sequence.is_empty() {
                        if let Some(mut new_follower) = self.lost.pop() {
                            new_follower.pos = self.party[0].prev_pos;
                            self.party.push(new_follower);
                        } else if !self.party.is_empty() {
                            self.party.remove(0);
                        }
                    }

                    // sidequests
                    if (self.upgrades & 0b10) != 0 && !self.party.is_empty() {
                        let target_pos = self.party[0].pos;
                        let new_followers = self.lost.extract_if(|lost| lost.pos == target_pos);
                        self.party.extend(new_followers);
                    }
                }
                None => {
                    if let Some(mut new_follower) = self.lost.pop() {
                        new_follower.pos = self.party[0].prev_pos;
                        self.party.push(new_follower);
                    } else if !self.party.is_empty() {
                        self.party.remove(0);
                    }
                }
            }

            if self.party.is_empty() && self.lost.is_empty() {
                return SolveStatus::Solved(1);
            }
        }

        SolveStatus::None
    }

    fn movement_time(&self) -> f32 {
        // Comradery
        if (self.upgrades & 0b1) != 0 {
            SNAIL_MOVEMENT_TIME * 10.0 / (9.0 + self.party.len() as f32)
        } else {
            SNAIL_MOVEMENT_TIME
        }
    }
}
