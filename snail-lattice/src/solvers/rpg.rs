use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
};

pub struct Rpg<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    leader: Snail<S>,
    followers: Vec<Snail<S>>,
    lost: Vec<Snail<S>>,

    current_target: Option<usize>,
}

impl<const S: usize> Solver<S> for Rpg<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        Rpg {
            leader: Snail::new(),
            followers: vec![],
            lost: vec![],
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
        self.leader.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer,
            self.movement_time(),
            image,
            bx,
            by,
        );

        for snail in &self.followers {
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
                DEFAULT_PALETTE,
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
        match self.current_target {
            Some(target) => {}
            None => {}
        };

        false
    }

    fn movement_time(&self) -> usize {
        SNAIL_MOVEMENT_TIME
    }
}
