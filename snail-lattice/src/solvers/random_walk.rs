use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
};

/// Random Walk Snail Upgrades:
/// - Four Leaf Clover: Gives 10% chance to go the right way
/// - Rabbit's Foot:    Gives an additional 20% to go the right way
/// - Horseshoe:        Gives an additional 30% to go the right way

pub struct RandomWalk<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    snail: Snail<S>,
    directions: [Direction; S * S],
    upgrades: u32,
}

impl<const S: usize> Solver<S> for RandomWalk<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        RandomWalk {
            snail: Snail::new(),
            directions: [Direction::Left; S * S],
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
        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer / self.movement_time(),
            image,
            bx,
            by,
        );
    }

    fn setup(&mut self, maze: &Maze<S>, _lfsr: &mut LFSR) {
        self.directions = maze.get_directions(maze.end_pos);
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
        // chance to move in the right direction based on the upgrades provided
        let chance = (self.upgrades & 0b1)
            + (self.upgrades & 0b10)
            + ((self.upgrades & 0b100) >> 1)
            + ((self.upgrades & 0b100) >> 2);

        if (lfsr.big() % 10) < chance as usize {
            self.snail.direction = self.directions[self.snail.pos.y * S + self.snail.pos.x];
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
            self.snail.reset();
            true
        } else {
            false
        }
    }

    fn movement_time(&self) -> f32 {
        SNAIL_MOVEMENT_TIME
    }
}
