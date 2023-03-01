use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
};

pub struct RandomWalk<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    snail: Snail<S>,
}

impl<const S: usize> Solver<S> for RandomWalk<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        RandomWalk {
            snail: Snail::new(),
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
        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer,
            self.movement_time(),
            image,
            bx,
            by,
        );
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
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

        if self.snail.pos == maze.end_pos {
            self.snail.reset();
            true
        } else {
            false
        }
    }

    fn movement_time(&self) -> usize {
        SNAIL_MOVEMENT_TIME
    }
}
