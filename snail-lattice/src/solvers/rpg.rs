use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE},
    solvers::Solver,
    utils::Vec2,
};

/// RPG Snail Upgrades:
/// - Comradery:   RPG Snail gets a 10% speed boost for each snail in its party.
/// - Sidequests:  Any snail RPG Snail runs into is automatically added to its party.
/// - Recruitment: The snails come to RPG snail on their own.

pub struct Rpg<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    party: Vec<Snail<S>>,
    lost: Vec<Snail<S>>,
    upgrades: u32,
    directions: [Option<Direction>; S * S],

    current_sequence: Vec<Direction>,
}

impl<const S: usize> Rpg<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn generate_lost_snails(&mut self, lfsr: &mut LFSR) {
        for _ in 0..(S / 2) {
            let mut x = 0;
            let mut y = 0;
            while (x == 0 && y == 0) || (x == S - 1 && y == S - 1) {
                x = lfsr.big() % S;
                y = lfsr.big() % S;
            }

            let mut new_snail = Snail::new();
            new_snail.pos = Vec2 { x, y };
            new_snail.prev_pos = new_snail.pos;

            self.lost.push(new_snail);
        }
    }
}

impl<const S: usize> Solver<S> for Rpg<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        Rpg {
            party: vec![],
            lost: vec![],

            directions: [None; S * S],
            current_sequence: vec![],
            upgrades: 0,
        }
    }

    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        _lfsr: &mut LFSR,
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        for snail in &self.party {
            snail.draw(
                DEFAULT_PALETTE,
                animation_cycle,
                movement_timer / self.movement_time(),
                image,
                bx,
                by,
            );
        }

        for snail in &self.lost {
            snail.draw(
                GRAYSCALE_PALETTE,
                animation_cycle,
                movement_timer / self.movement_time(),
                image,
                bx,
                by,
            );
        }
    }

    fn setup(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) {
        self.lost.clear();
        self.party.clear();

        self.party.push(Snail::new());
        self.generate_lost_snails(lfsr);

        if (self.upgrades & 0b100) != 0 {
            self.directions = maze.get_directions(Vec2 { x: 0, y: 0 });
        }
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
        // recruitment
        if (self.upgrades & 0b100) != 0 && !self.lost.is_empty() {
            if !(self.party[0].pos.x == 0 && self.party[0].pos.y == 0) {
                self.setup(maze, lfsr);
                return false;
            }

            for lost_snail in &mut self.lost {
                lost_snail.direction =
                    self.directions[lost_snail.pos.y * S + lost_snail.pos.x].unwrap();
                lost_snail.move_forward(maze);
            }

            let target_pos = self.party[0].pos;

            // add any lost snails that are in the party
            let new_snails = self.lost.drain_filter(|snail| snail.pos == target_pos);

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
                        let new_followers = self.lost.drain_filter(|lost| lost.pos == target_pos);
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
                return true;
            }
        }

        false
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
