use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
    utils::discrete_lerp,
};

const TELEPORTATION_TIME: usize = 6;

pub struct RandomTeleport<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    snail: Snail<S>,
    teleport_timer: usize,
}

impl<const S: usize> Solver<S> for RandomTeleport<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        RandomTeleport {
            snail: Snail::new(),
            teleport_timer: 0,
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
        let movement_time = self.movement_time();

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer,
            movement_time,
            image,
            bx,
            by,
        );

        let mut px = 4
            * ((by + self.snail.pos.y * 10 + 11) * image.buffer_width
                + bx
                + self.snail.pos.x * 10
                + 1);

        if px > image.buffer.len() {
            px -= 44 * image.buffer_width;
        }

        let progress = discrete_lerp(
            0,
            36,
            ((self.teleport_timer * movement_time + movement_timer)
                % (TELEPORTATION_TIME * movement_time)) as i32,
            (TELEPORTATION_TIME * movement_time) as i32,
        ) as usize;

        // draw progress bar under snail
        for index in (px..(px + progress)).step_by(4) {
            image.draw_pixel(index, [0x00, 0xFF, 0x00]);
        }
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
        self.snail.prev_pos.x = self.snail.pos.x;
        self.snail.prev_pos.y = self.snail.pos.y;
        self.teleport_timer += 1;
        if self.teleport_timer % TELEPORTATION_TIME == 0 {
            self.snail.pos.x = lfsr.big() % S;
            self.snail.pos.y = lfsr.big() % S;

            if self.snail.pos == maze.end_pos {
                self.snail.reset();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn movement_time(&self) -> usize {
        SNAIL_MOVEMENT_TIME
    }
}
