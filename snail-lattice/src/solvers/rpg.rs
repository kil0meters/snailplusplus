use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE, INVERTED_PALETTE},
    solvers::Solver,
    utils::{console_log, Vec2},
};

pub struct Rpg<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    party: Vec<Snail<S>>,
    lost: Vec<Snail<S>>,

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

            current_sequence: vec![],
        }
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        _lfsr: &mut LFSR,
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        for snail in &self.party {
            snail.draw(
                DEFAULT_PALETTE,
                animation_cycle,
                movement_timer,
                self.movement_time(),
                image,
                bx,
                by,
            );
        }

        for snail in &self.lost {
            snail.draw(
                GRAYSCALE_PALETTE,
                animation_cycle,
                movement_timer,
                self.movement_time(),
                image,
                bx,
                by,
            );
        }
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
        if self.party.is_empty() && self.lost.is_empty() {
            self.party.push(Snail::new());
            self.generate_lost_snails(lfsr);

            return true;
        }

        if self.current_sequence.is_empty() {
            if let Some(last) = self.lost.last() {
                self.current_sequence =
                    maze.get_solve_sequence(self.party[0].pos.x, self.party[0].pos.y, last.pos);
            } else {
                self.current_sequence =
                    maze.get_solve_sequence(self.party[0].pos.x, self.party[0].pos.y, maze.end_pos);
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

                for (i, lost) in self.lost.iter().enumerate() {
                    if self.party[0].pos == lost.pos {
                        let mut new_follower = self.lost.remove(i);
                        new_follower.pos = self.party[0].prev_pos;
                        self.party.push(new_follower);
                        break;
                    }
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
        };

        false
    }

    fn movement_time(&self) -> usize {
        SNAIL_MOVEMENT_TIME
    }
}
