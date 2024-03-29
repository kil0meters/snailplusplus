use crate::{
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
    utils::{lerpi, Vec2},
};

use super::SolveStatus;

/// Random Teleport Snail Upgrades:
/// - Fusion Reactor:         Random Teleport Snail uses a fusion reactor to charge up its teleportation 20% faster.
/// - Homing Beacon:          After every teleport, Random Teleport Snail shrinks its teleportation range by 1 tile
/// - Advanced Homing Beacon: After every teleport, Random Teleport Snail shrinks its teleportaiton range based on its new position

pub struct RandomTeleport<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    snail: Snail<S>,
    teleport_timer: f32,
    teleport_bounds: Vec2,
    prev_teleport_bounds: Vec2,
    upgrades: u32,
}

impl<const S: usize> RandomTeleport<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn teleportation_time(&self) -> f32 {
        // has fusion reactor upgrade
        if (self.upgrades & 1) != 0 {
            5.0 * SNAIL_MOVEMENT_TIME
        } else {
            6.0 * SNAIL_MOVEMENT_TIME
        }
    }
}

impl<const S: usize> Solver<S> for RandomTeleport<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        RandomTeleport {
            snail: Snail::new(),
            teleport_timer: 0.0,
            prev_teleport_bounds: Vec2 { x: S, y: S },
            teleport_bounds: Vec2 { x: S, y: S },
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
        let movement_time = self.movement_time();

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer / movement_time,
            image,
            bx,
            by,
        );

        let mut px =
            4 * ((by + self.snail.pos.y * 10 + 11) * image.width + bx + self.snail.pos.x * 10 + 1);

        if px > image.buffer.len() {
            px -= 44 * image.width;
        }

        let teleportation_progress =
            (self.teleport_timer + movement_timer) / self.teleportation_time();
        let progress = lerpi(0, 36, teleportation_progress) as usize;

        // draw progress bar under snail
        for index in (px..(px + progress)).step_by(4) {
            image.draw_pixel(index, [0x00, 0xFF, 0x00]);
        }

        // draw current teleportation bounds if homing beacon is enabled
        if (self.upgrades & 0b11) != 0 {
            let y_start = lerpi(
                10 * (S - self.prev_teleport_bounds.y) as i32,
                10 * (S - self.teleport_bounds.y) as i32,
                teleportation_progress,
            ) as usize;

            let x_start = lerpi(
                10 * (S - self.prev_teleport_bounds.x) as i32,
                10 * (S - self.teleport_bounds.x) as i32,
                teleportation_progress,
            ) as usize;

            let start_px = 4 * (((by + y_start) * image.width) + bx + x_start);

            for index in (start_px..(start_px + 4 * (S * 10 - x_start))).step_by(12) {
                image.draw_pixel(index, [0xFF, 0x00, 0x00]);
            }

            let start_px = 4 * (((by + y_start) * image.width) + bx + x_start);

            for index in (start_px..(start_px + (4 * (S * 10 - y_start) * image.width)))
                .step_by(12 * image.width)
            {
                image.draw_pixel(index, [0xFF, 0x00, 0x00]);
                image.draw_pixel(index + 4 * (S * 10 - x_start), [0xFF, 0x00, 0x00]);
            }

            let start_px = 4 * (((by + 10 * S) * image.width) + bx + x_start);

            for index in (start_px..(start_px + 4 * (S * 10 - x_start))).step_by(12) {
                image.draw_pixel(index, [0xFF, 0x00, 0x00]);
            }
        }
    }

    fn setup(&mut self, _maze: &Maze<S>, _lfsr: &mut LFSR) {
        self.snail.reset();
        self.teleport_bounds = Vec2 { y: S, x: S };
        self.prev_teleport_bounds = self.teleport_bounds;
    }

    fn step(&mut self, maze: &mut Maze<S>, lfsr: &mut LFSR) -> SolveStatus {
        self.snail.prev_pos.x = self.snail.pos.x;
        self.snail.prev_pos.y = self.snail.pos.y;
        self.teleport_timer += SNAIL_MOVEMENT_TIME;
        if self.teleport_timer >= self.teleportation_time() {
            self.teleport_timer = 0.0;
            self.snail.pos.x = S - (lfsr.big() % self.teleport_bounds.x) - 1;
            self.snail.pos.y = S - (lfsr.big() % self.teleport_bounds.y) - 1;

            self.prev_teleport_bounds = self.teleport_bounds;

            // if has advanced homing beacon
            if (self.upgrades & 0b100) != 0 {
                self.teleport_bounds.x = S - self.snail.pos.x;
                self.teleport_bounds.y = S - self.snail.pos.y;
            }

            // if has homing beacon
            if (self.upgrades & 0b10) != 0 {
                if self.teleport_bounds.y < self.teleport_bounds.x && self.teleport_bounds.x > 1 {
                    self.teleport_bounds.x -= 1;
                } else if self.teleport_bounds.y > 1 {
                    self.teleport_bounds.y -= 1;
                }
            }

            if self.snail.pos == maze.end_pos {
                SolveStatus::Solved(1)
            } else {
                SolveStatus::None
            }
        } else {
            SolveStatus::None
        }
    }

    fn movement_time(&self) -> f32 {
        SNAIL_MOVEMENT_TIME
    }
}
