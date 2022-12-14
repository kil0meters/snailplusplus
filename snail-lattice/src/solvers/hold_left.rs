use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::Snail,
    solvers::Solver,
};

pub struct HoldLeft<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    snail: Snail<S>,
}

impl<const S: usize> Solver<S> for HoldLeft<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        HoldLeft {
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
            animation_cycle,
            movement_timer,
            self.movement_time(),
            image,
            bx,
            by,
        );
    }

    fn step(&mut self, maze: &Maze<S>, _lfsr: &mut LFSR) -> bool {
        let cell = maze.get_cell(self.snail.pos.x, self.snail.pos.y);
        let left = self.snail.direction.rotate_counter();

        // if we can move left, do so
        if !cell.has_wall(left) {
            self.snail.direction = left;
        }
        // otherwise, if there's a wall blocking the front, rotate clockwise until we face an empty
        // wall
        else {
            while cell.has_wall(self.snail.direction) {
                self.snail.direction = self.snail.direction.rotate();
            }
        }

        self.snail.move_forward(maze);

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
