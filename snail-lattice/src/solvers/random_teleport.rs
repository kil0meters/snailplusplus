use crate::{
    direction::Direction,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::Snail,
    solvers::Solver,
    utils::{console_log, discrete_lerp, draw_pixel},
};

const TELEPORTATION_TIME: usize = 6;

pub struct RandomTeleport {
    snail: Snail,
    teleport_timer: usize,
}

impl RandomTeleport {
    pub fn new(_upgrades: usize) -> Self {
        RandomTeleport {
            snail: Snail::new(),
            teleport_timer: 0,
        }
    }
}

impl Solver for RandomTeleport {
    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        buffer: &mut [u8],
        buffer_width: usize,
        bx: usize,
        by: usize,
    ) {
        self.snail.draw(
            animation_cycle,
            movement_timer,
            buffer,
            buffer_width,
            bx,
            by,
        );

        let mut px =
            4 * ((by + self.snail.pos.y * 10 + 11) * buffer_width + bx + self.snail.pos.x * 10 + 1);

        if px > buffer.len() {
            px -= 44 * buffer_width;
        }

        let progress = discrete_lerp(
            -2,
            36,
            ((self.teleport_timer * SNAIL_MOVEMENT_TIME + movement_timer)
                % (TELEPORTATION_TIME * SNAIL_MOVEMENT_TIME)) as i32,
            (TELEPORTATION_TIME * SNAIL_MOVEMENT_TIME) as i32,
        ) as usize;

        // draw progress bar under snail
        for index in (px..(px + progress)).step_by(4) {
            draw_pixel(buffer, index, [0x00, 0xFF, 0x00]);
        }
    }

    fn step(&mut self, maze: &Maze, lfsr: &mut LFSR) -> bool {
        self.snail.prev_pos.x = self.snail.pos.x;
        self.snail.prev_pos.y = self.snail.pos.y;
        self.teleport_timer += 1;
        if self.teleport_timer % TELEPORTATION_TIME == 0 {
            self.snail.pos.x = lfsr.big() % maze.width;
            self.snail.pos.y = lfsr.big() % maze.height;

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
}
