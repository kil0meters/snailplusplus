use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::PHASE_2_PALETTE,
    solvers::Solver,
};

use super::SolveStatus;

/// Swarm Snail Upgrades:
/// - Carbon Fiber Exoskeleton: Moves 50% faster
/// - Singing Lessons: Larger swarm
/// - Microphone: Larger swarm

// chatgpt gave me this, surely it will work
// wtf it did thanks person who wrote similar code on github
fn interpolate_with_bezier(start: f32, end: f32, weight1: f32, weight2: f32, t: f32) -> f32 {
    let t1 = 1.0 - t;
    let p0 = start * t1 * t1 * t1;
    let p1 = start * 3.0 * t1 * t1 * t;
    let p2 = end * 3.0 * t1 * t * t;
    let p3 = end * t * t * t;

    p0 + p1 * weight1 + p2 * weight2 + p3
}

pub struct Flying {
    upgrades: u32,
    swarm_weights: Vec<(f32, f32, f32, f32)>,
}

impl Flying {
    pub fn new() -> Self {
        Flying {
            upgrades: 0,
            swarm_weights: vec![],
        }
    }
}

impl Solver for Flying {
    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        maze: &Maze,
        _lfsr: &mut LFSR,
        image: &mut Image,
    ) {
        let progress = movement_timer / self.movement_time();

        for &(weight1, weight2, weight3, weight4) in &self.swarm_weights {
            image.draw_snail(
                PHASE_2_PALETTE,
                animation_cycle,
                Direction::Right,
                interpolate_with_bezier(
                    0.0,
                    ((maze.size - 1) * 10) as f32,
                    weight1,
                    weight2,
                    progress,
                ) as usize,
                interpolate_with_bezier(
                    0.0,
                    ((maze.size - 1) * 10) as f32,
                    weight3,
                    weight4,
                    progress,
                ) as usize,
            );
        }
    }

    //
    fn setup(&mut self, _maze: &Maze, lfsr: &mut LFSR) {
        self.swarm_weights.clear();

        let mut swarm_count = 6;

        if self.upgrades & 0b10 != 0 {
            swarm_count += 3;
        }

        if self.upgrades & 0b100 != 0 {
            swarm_count += 5;
        }

        for _ in 0..swarm_count {
            let weight1 = (lfsr.big() % 101) as f32 / 100.0;
            let weight2 = (lfsr.big() % 101) as f32 / 100.0;
            let weight3 = (lfsr.big() % 101) as f32 / 100.0;
            let weight4 = (lfsr.big() % 101) as f32 / 100.0;

            self.swarm_weights
                .push((weight1, weight2, weight3, weight4));
        }
    }

    fn step(&mut self, _maze: &mut Maze, _lfsr: &mut LFSR) -> SolveStatus {
        SolveStatus::Solved(self.swarm_weights.len())
    }

    fn palette(&self) -> [[u8; 3]; 6] {
        PHASE_2_PALETTE
    }

    fn movement_time(&self) -> f32 {
        // Carbon Fiber Exoskeleton
        if self.upgrades & 0b1 != 0 {
            SNAIL_MOVEMENT_TIME * 5.0
        } else {
            SNAIL_MOVEMENT_TIME * 10.0
        }
    }
}
