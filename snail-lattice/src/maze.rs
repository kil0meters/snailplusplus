use std::mem::size_of;

use crate::{direction::Direction, image::Image, lfsr::LFSR, solvers::Solver, utils::Vec2};

pub const SNAIL_BG: [u8; 3] = [0x11, 0x0A, 0xEF];
pub const SNAIL_FG: [u8; 3] = [0x06, 0x8F, 0xEF];
pub const SNAIL_MOVEMENT_TIME: usize = 250000;
pub const ANIMATION_TIME: usize = 500000;

// each cell is 4 bits, so 2 cells per byte
pub const CELLS_PER_IDX: usize = size_of::<usize>() * 2;

pub struct MazeCell(pub usize);

impl MazeCell {
    pub fn has_wall(&self, dir: Direction) -> bool {
        self.0 & (1 << (3 - dir as usize)) != 0
    }

    pub fn valid_directions(&self) -> Vec<Direction> {
        vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .into_iter()
        .filter(|d| !self.has_wall(*d))
        .collect()
    }
}

pub struct AutoMaze<const S: usize, T: Solver<S>>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    solver: T,

    // stores time since start, in microseconds
    clock: usize,

    // time since last movement
    movement_timer: usize,

    pub maze: Maze<S>,
}

impl<const S: usize, T: Solver<S>> AutoMaze<S, T>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    pub fn new(solver: T) -> AutoMaze<S, T> {
        AutoMaze {
            solver,
            clock: 0,
            movement_timer: 0,

            maze: Maze::<S> {
                end_pos: Vec2 { x: S - 1, y: S - 1 },
                walls: [0; _],
                visited: [false; _],
            },
        }
    }

    // progresses time a certain number of microseconds
    // notably, no rendering happens when we tick the time
    // returns true if the tick results in a new maze to be generated
    pub fn tick(&mut self, dt: usize, lfsr: &mut LFSR) -> usize {
        let prev = self.clock;
        let now = self.clock + dt;
        self.clock = now;
        let movement_time = self.solver.movement_time();

        let mut num_movements = (now - prev) / movement_time;
        self.movement_timer += (now - prev) % movement_time;
        if self.movement_timer > movement_time {
            self.movement_timer -= movement_time;
            num_movements += 1;
        }

        let mut total = 0;

        for _ in 0..num_movements {
            if self.solver.step(&self.maze, lfsr) {
                total += S * S;
                self.maze.generate(lfsr);
                self.movement_timer = movement_time;
            }
        }

        total
    }

    pub fn draw(&mut self, lfsr: &mut LFSR, image: &mut Image, bx: usize, by: usize) {
        let animation_cycle = (self.clock / ANIMATION_TIME) % 2 == 0;

        // draw "snail"
        self.solver
            .draw(animation_cycle, self.movement_timer, lfsr, image, bx, by);
        self.maze.draw_foreground(animation_cycle, image, bx, by);
    }
}

// An SxS maze
pub struct Maze<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    pub end_pos: Vec2,

    // each cell is 4 bits
    pub walls: [usize; (S * S) / CELLS_PER_IDX + 1],
    visited: [bool; S * S],
}

impl<const S: usize> Maze<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    // 4 bytes
    fn set_cell(&mut self, x: usize, y: usize, data: usize) {
        let offset = y * S + x;

        self.walls[offset / CELLS_PER_IDX] ^=
            data << (4 * (CELLS_PER_IDX - (offset % CELLS_PER_IDX) - 1));
    }

    pub fn get_cell(&self, x: usize, y: usize) -> MazeCell {
        let offset = y * S + x;

        MazeCell(
            self.walls[offset / CELLS_PER_IDX]
                >> (4 * (CELLS_PER_IDX - (offset % CELLS_PER_IDX) - 1))
                & 0b1111,
        )
    }

    fn set_cell_wall(&mut self, x: usize, y: usize, direction: Direction) {
        self.set_cell(x, y, 1 << (3 - direction as usize));
    }

    fn random_walk(&mut self, x: usize, y: usize, lfsr: &mut LFSR) {
        let mut next = Some((x, y));

        while let Some((x, y)) = next {
            self.visited[y * S + x] = true;
            next = None;

            for direction in lfsr.random_order() {
                // right
                if direction == 0 && x < S - 1 && !self.visited[y * S + x + 1] {
                    self.set_cell_wall(x, y, Direction::Right);
                    self.set_cell_wall(x + 1, y, Direction::Left);
                    next = Some((x + 1, y));
                }
                // left
                else if direction == 1 && x > 0 && !self.visited[y * S + x - 1] {
                    self.set_cell_wall(x, y, Direction::Left);
                    self.set_cell_wall(x - 1, y, Direction::Right);
                    next = Some((x - 1, y));
                }
                // up
                else if direction == 2 && y > 0 && !self.visited[(y - 1) * S + x] {
                    self.set_cell_wall(x, y, Direction::Up);
                    self.set_cell_wall(x, y - 1, Direction::Down);
                    next = Some((x, y - 1));
                }
                // down
                else if direction == 3 && y < S - 1 && !self.visited[(y + 1) * S + x] {
                    self.set_cell_wall(x, y, Direction::Down);
                    self.set_cell_wall(x, y + 1, Direction::Up);
                    next = Some((x, y + 1));
                }

                if next.is_some() {
                    break;
                }
            }
        }
    }

    pub fn generate(&mut self, lfsr: &mut LFSR) {
        // set all elements in vector to 1s
        self.walls = [!0usize; _];

        self.visited = [false; S * S];

        self.random_walk(0, 0, lfsr);

        for y in 0..S {
            for x in 0..S {
                if !self.visited[y * S + x] {
                    for direction in [0, 1, 2, 3] {
                        // right
                        if direction == 0 && x < S - 1 && self.visited[y * S + x + 1] {
                            self.set_cell_wall(x, y, Direction::Right);
                            self.set_cell_wall(x + 1, y, Direction::Left);
                            self.random_walk(x, y, lfsr);
                            break;
                        }
                        // left
                        else if direction == 1 && x > 0 && self.visited[y * S + x - 1] {
                            self.set_cell_wall(x, y, Direction::Left);
                            self.set_cell_wall(x - 1, y, Direction::Right);
                            self.random_walk(x, y, lfsr);
                            break;
                        }
                        // up
                        else if direction == 2 && y > 0 && self.visited[(y - 1) * S + x] {
                            self.set_cell_wall(x, y, Direction::Up);
                            self.set_cell_wall(x, y - 1, Direction::Down);
                            self.random_walk(x, y, lfsr);
                            break;
                        }
                        // down
                        else if direction == 3 && y < S - 1 && self.visited[(y + 1) * S + x] {
                            self.set_cell_wall(x, y, Direction::Down);
                            self.set_cell_wall(x, y + 1, Direction::Up);
                            self.random_walk(x, y, lfsr);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_background(&mut self, image: &mut Image, bx: usize, by: usize) {
        for y in 0..(S * 10) {
            for x in 0..S {
                let cell = self.get_cell(x, y / 10);
                let px = ((by + y) * image.buffer_width + bx + (x * 10)) * 4;

                // Checking the bottom wall is redundant
                if y % 10 == 0 && cell.has_wall(Direction::Up) {
                    for l in (px..(px + 4 * 10)).step_by(4) {
                        image.draw_pixel(l, SNAIL_FG);
                    }
                } else {
                    // if left wall, checking right wall is redundant
                    if cell.has_wall(Direction::Left) || y % 10 == 0 {
                        image.draw_pixel(px, SNAIL_FG);
                    } else {
                        image.draw_pixel(px, SNAIL_BG);
                    }

                    for l in ((px + 4)..(px + 4 * 10)).step_by(4) {
                        image.draw_pixel(l, SNAIL_BG);
                    }
                }
            }

            // fill end pixel
            let px = 4 * ((by + y) * image.buffer_width + bx + S * 10);
            image.draw_pixel(px, SNAIL_FG);
        }

        let px = 4 * ((by + S * 10) * image.buffer_width + bx);
        for l in (px..(px + 4 * (1 + 10 * S))).step_by(4) {
            image.draw_pixel(l, SNAIL_FG);
        }
    }

    fn draw_foreground(&self, animation_cycle: bool, image: &mut Image, bx: usize, by: usize) {
        // draw goal
        if animation_cycle {
            const GOAL_IMAGE_SIZE: usize = 7;

            let goal_image = include_bytes!("../../assets/goal_7x7.bin");

            for y in 0..GOAL_IMAGE_SIZE {
                for x in 0..GOAL_IMAGE_SIZE {
                    let goal_px = 4 * (y * GOAL_IMAGE_SIZE + x);
                    let px = 4
                        * ((by + x + self.end_pos.y * 10 + 2) * image.buffer_width
                            + bx
                            + y
                            + self.end_pos.x * 10
                            + 2);

                    if goal_image[goal_px + 3] != 0 {
                        image.buffer[px] = goal_image[goal_px];
                        image.buffer[px + 1] = goal_image[goal_px + 1];
                        image.buffer[px + 2] = goal_image[goal_px + 2];
                    }
                }
            }
        }
    }
}
