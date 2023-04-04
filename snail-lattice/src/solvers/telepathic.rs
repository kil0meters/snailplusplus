use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, INVERTED_PALETTE, PHASE_2_PALETTE},
    solvers::Solver,
    utils::{lerpi, Vec2},
};

use super::SolveStatus;

struct TelepathyBall {
    prev_pos: Vec2,
    pos: Vec2,
    dir: Direction,
}

impl TelepathyBall {
    fn new(x: usize, y: usize) -> TelepathyBall {
        TelepathyBall {
            pos: Vec2 { x, y },
            prev_pos: Vec2 { x, y },
            dir: Direction::Right,
        }
    }

    fn step(&mut self, dir: Direction) {
        self.prev_pos = self.pos;
        self.dir = dir;

        match self.dir {
            Direction::Up => self.pos.y -= 1,
            Direction::Down => self.pos.y += 1,
            Direction::Left => self.pos.x -= 1,
            Direction::Right => self.pos.x += 1,
        }
    }

    fn draw(&self, image: &mut Image, progress: f32, lfsr: &mut LFSR) {
        let x = lerpi(
            10 * (self.prev_pos.x as i32),
            10 * (self.pos.x as i32),
            progress,
        ) as usize
            + 5;
        let y = lerpi(
            10 * (self.prev_pos.y as i32),
            10 * (self.pos.y as i32),
            progress,
        ) as usize
            + 5;

        let random_color = || {
            if lfsr.next() < 2 {
                [0x55, 0xaa, 0xff]
            } else {
                [0xff, 0xff, 0xff]
            }
        };

        image.draw_circle_with(random_color, x, y, 7);
    }
}

struct Goal {
    pos: Vec2,
    prev_pos: Vec2,
}

impl Goal {
    fn new(x: usize, y: usize) -> Goal {
        Goal {
            pos: Vec2 { x, y },
            prev_pos: Vec2 { x, y },
        }
    }

    fn set_pos(&mut self, x: usize, y: usize) {
        self.pos.x = x;
        self.pos.y = y;
        self.prev_pos = self.pos;
    }

    fn step(&mut self, dir: Direction) {
        self.prev_pos = self.pos;

        match dir {
            Direction::Up => self.pos.y -= 1,
            Direction::Down => self.pos.y += 1,
            Direction::Left => self.pos.x -= 1,
            Direction::Right => self.pos.x += 1,
        }
    }

    fn draw(&self, image: &mut Image, animation_cycle: bool, progress: f32) {
        let x = lerpi(
            10 * (self.prev_pos.x as i32),
            10 * (self.pos.x as i32),
            progress,
        ) as usize;
        let y = lerpi(
            10 * (self.prev_pos.y as i32),
            10 * (self.pos.y as i32),
            progress,
        ) as usize;

        if animation_cycle {
            image.draw_goal(DEFAULT_PALETTE[0], x, y);
        } else {
            image.draw_goal(INVERTED_PALETTE[0], x, y);
        }
    }
}

/// Telepathic Snail Upgrades:
/// - Untested Surgery: The Telepathic Snail undergoes an experimental surgery which allows it to move and use its telepathy at the same time.
/// - Kinesiology Degree: The Telepathic Snail goes to college to study kinesiology. With a newfound understanding of snail kinematics, it is able to use its telepathic abilities to move faster.
/// - Split Brain: The Telepathic Snail attracts the goal to it at the same rate.

pub struct Telepathic {
    snail: Snail,
    forward_ball: TelepathyBall,
    goal: Goal,
    upgrades: u32,
    snail_move_timer: usize,

    ball_sequence: Vec<Direction>,
    ball_sequence_index: usize,
    snail_sequence_index: usize,
    goal_sequence_index: usize,
}

impl Telepathic {
    pub fn new() -> Self {
        Telepathic {
            snail: Snail::new(),
            goal: Goal::new(0, 0),
            upgrades: 0,
            forward_ball: TelepathyBall::new(0, 0),
            ball_sequence: vec![],
            ball_sequence_index: 0,
            snail_sequence_index: 0,
            goal_sequence_index: 0,
            snail_move_timer: 0,
        }
    }

    fn move_cooldown(&self) -> usize {
        if self.upgrades & 0b10 != 0 {
            2
        } else {
            3
        }
    }
}

impl Solver for Telepathic {
    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        _maze: &Maze,
        lfsr: &mut LFSR,
        image: &mut Image,
    ) {
        let progress = movement_timer / self.movement_time();
        let cooldown = self.move_cooldown() as f32;

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            progress / cooldown + self.snail_move_timer as f32 / cooldown,
            image,
        );

        if self.ball_sequence_index < self.ball_sequence.len() {
            self.forward_ball.draw(image, progress, lfsr);
        }

        if self.upgrades & 0b100 != 0 {
            self.goal.draw(image, animation_cycle, progress);
        }
    }

    fn setup(&mut self, maze: &Maze, lfsr: &mut LFSR) {
        self.snail.reset();
        self.forward_ball.pos.x = 0;
        self.forward_ball.pos.y = 0;
        self.goal.set_pos(maze.size - 1, maze.size - 1);
        self.goal.prev_pos = self.goal.pos;
        self.ball_sequence.clear();
        self.ball_sequence_index = 0;
        self.snail_sequence_index = 0;
        self.goal_sequence_index = 0;

        let mut right_moves = 0;
        let mut down_moves = 0;

        while self.ball_sequence.len() < maze.size * 2 - 2 {
            if right_moves == maze.size - 1 {
                self.ball_sequence.push(Direction::Down);
            } else if down_moves == maze.size - 1 {
                self.ball_sequence.push(Direction::Right)
            } else {
                if lfsr.next() < 2 {
                    self.ball_sequence.push(Direction::Right);
                    right_moves += 1;
                } else {
                    self.ball_sequence.push(Direction::Down);
                    down_moves += 1;
                }
            }
        }
    }

    fn step(&mut self, maze: &mut Maze, _lfsr: &mut LFSR) -> SolveStatus {
        let mut rerender = false;

        if self.ball_sequence_index < self.ball_sequence.len() {
            match (
                self.forward_ball.dir,
                self.ball_sequence[self.ball_sequence_index],
            ) {
                (Direction::Down, Direction::Right) => {
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Down,
                    );
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Left,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Right,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Up,
                    );
                }
                (Direction::Down, Direction::Down) => {
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Right,
                    );
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Left,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Up,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Down,
                    );
                }
                (Direction::Right, Direction::Down) => {
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Right,
                    );
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Up,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Left,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Down,
                    );
                }
                (Direction::Right, Direction::Right) => {
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Up,
                    );
                    maze.add_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Down,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Left,
                    );
                    maze.remove_wall(
                        self.forward_ball.pos.x,
                        self.forward_ball.pos.y,
                        Direction::Right,
                    );
                }

                _ => {}
            }

            self.forward_ball
                .step(self.ball_sequence[self.ball_sequence_index]);
            self.ball_sequence_index += 1;

            rerender = true;
        }

        if self.upgrades & 0b100 != 0 && !(self.ball_sequence_index < self.ball_sequence.len()) {
            let dir =
                self.ball_sequence[self.ball_sequence.len() - 1 - self.goal_sequence_index].flip();
            self.goal.step(dir);

            self.goal_sequence_index += 1;
        }

        if self.upgrades & 0b1 != 0 || !(self.ball_sequence_index < self.ball_sequence.len()) {
            self.snail_move_timer += 1;

            if self.snail_move_timer >= self.move_cooldown() {
                self.snail.direction = self.ball_sequence[self.snail_sequence_index];
                self.snail.move_forward(maze);

                self.snail_sequence_index += 1;
                self.snail_move_timer = 0;
            }

            if self.snail.pos == self.goal.prev_pos || self.snail.prev_pos == self.goal.prev_pos {
                return SolveStatus::Solved(1);
            }
        }

        if rerender {
            SolveStatus::Rerender
        } else {
            SolveStatus::None
        }
    }

    fn custom_goal(&self) -> bool {
        self.upgrades & 0b100 != 0
    }

    fn palette(&self) -> [[u8; 3]; 6] {
        PHASE_2_PALETTE
    }

    fn movement_time(&self) -> f32 {
        SNAIL_MOVEMENT_TIME / 3.0
    }
}
