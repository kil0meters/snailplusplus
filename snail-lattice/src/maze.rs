use bit_vec::BitVec;

use crate::{utils::{Vec2, draw_pixel}, lfsr::LFSR, solvers::Solver};

pub const SNAIL_BG: [u8; 3] = [0x11, 0x0A, 0xEF];
pub const SNAIL_FG: [u8; 3] = [0x68, 0x8F, 0xEF];
pub const SNAIL_MOVEMENT_TIME: usize = 250000;
pub const ANIMATION_TIME: usize = 500000;

pub struct AutoMaze {
    // due to wasm bindgen limitations we can't use template structs yet so we have to deal with
    // some overhead here unfortunately
    solver: Box<dyn Solver>,

    // stores time since start, in microseconds
    clock: usize,

    // time since last movement
    movement_timer: usize,

    pub maze: Maze,
}

impl AutoMaze {
    pub fn new(solver: Box<dyn Solver>, width: usize, height: usize) -> AutoMaze {
        AutoMaze {
            solver,

            clock: 0,
            movement_timer: 0,

            maze: Maze {
                width,
                height,
                end_pos: Vec2 { x: width - 1, y: height - 1 },
                walls: BitVec::from_elem(width * height * 4, true),
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

        let mut num_movements = (now - prev) / SNAIL_MOVEMENT_TIME;
        self.movement_timer += (now - prev) % SNAIL_MOVEMENT_TIME;
        if self.movement_timer > SNAIL_MOVEMENT_TIME {
            self.movement_timer -= SNAIL_MOVEMENT_TIME;
            num_movements += 1;
        }

        let mut total = 0;

        for _ in 0..num_movements {
            if self.solver.step(&self.maze, lfsr) {
                total += self.maze.width * self.maze.height;
                self.maze.generate(lfsr);
                self.movement_timer = SNAIL_MOVEMENT_TIME;
            }
        }

        total
    }

    pub fn draw(&self, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        let animation_cycle = (self.clock / ANIMATION_TIME) % 2 == 0;

        // draw "snail"
        self.solver.draw(animation_cycle, self.movement_timer, buffer, buffer_width, bx, by);
        self.maze.draw_foreground(animation_cycle, buffer, buffer_width, bx, by);
    }
}

pub struct Maze {
    // logical dimensions
    pub width: usize,
    pub height: usize,

    pub end_pos: Vec2,

    pub walls: BitVec,
}

impl Maze {
    fn random_walk(&mut self, x: usize, y: usize, visited: &mut BitVec, lfsr: &mut LFSR) {
        let mut next = Some((x, y));

        while let Some((x, y)) = next {
            visited.set(y * self.width + x, true);
            next = None;

            for direction in lfsr.random_order() {
                // right
                if direction == 0 && x < self.width - 1 && !visited[y * self.width + x + 1] {
                    self.walls.set((y * self.width + x) * 4 + 3, false);
                    self.walls.set((y * self.width + x + 1) * 4 + 2, false);
                    next = Some((x + 1, y));
                }
                // left
                else if direction == 1 && x > 0 && !visited[y * self.width + x - 1] {
                    self.walls.set((y * self.width + x) * 4 + 2, false);
                    self.walls.set((y * self.width + x - 1) * 4 + 3, false);
                    next = Some((x - 1, y));
                }
                // up
                else if direction == 2 && y > 0 && !visited[(y - 1) * self.width + x] {
                    self.walls.set((y * self.width + x) * 4, false);
                    self.walls.set(((y - 1) * self.width + x) * 4 + 1, false);
                    next = Some((x, y - 1));
                }
                // down
                else if direction == 3 && y < self.height - 1 && !visited[(y + 1) * self.width + x] {
                    self.walls.set((y * self.width + x) * 4 + 1, false);
                    self.walls.set(((y + 1) * self.width + x) * 4, false);
                    next = Some((x, y + 1));
                }

                if next.is_some() {
                    break;
                }
            }
        }
    }

    pub fn generate(&mut self, lfsr: &mut LFSR) {
        // prefill vector with
        self.walls.set_all();

        let mut visited = BitVec::from_elem(self.width * self.height, false);

        self.random_walk(0, 0, &mut visited, lfsr);

        for y in 0..self.height {
            for x in 0..self.width {
                if !visited[y * self.width + x] {
                    for direction in [0, 1, 2, 3] {// rng.random_order() {
                        // right
                        if direction == 0 && x < self.width - 1 && visited[y * self.width + x + 1] {
                            self.walls.set((y * self.width + x) * 4 + 3, false);
                            self.walls.set((y * self.width + x + 1) * 4 + 2, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                        // left
                        else if direction == 1 && x > 0 && visited[y * self.width + x - 1] {
                            self.walls.set((y * self.width + x) * 4 + 2, false);
                            self.walls.set((y * self.width + x - 1) * 4 + 3, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                        // up
                        else if direction == 2 && y > 0 && visited[(y - 1) * self.width + x] {
                            self.walls.set((y * self.width + x) * 4, false);
                            self.walls.set(((y - 1) * self.width + x) * 4 + 1, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                        // down
                        else if direction == 3 && y < self.height - 1 && visited[(y + 1) * self.width + x] {
                            self.walls.set((y * self.width + x) * 4 + 1, false);
                            self.walls.set(((y + 1) * self.width + x) * 4, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_background(&self, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        for y in 0..(self.height * 10) {
            for x in 0..self.width {
                let loc = 4*((y / 10) * self.width + x);
                let px = ((by + y) * buffer_width + bx + (x * 10)) * 4;

                // Checking the bottom wall is redundant
                if y % 10 == 0 && self.walls[loc] {
                    for l in (px..(px + 4 * 10)).step_by(4) {
                        draw_pixel(buffer, l, SNAIL_FG);
                    }
                }

                else {
                    // if left wall, checking right wall is redundant
                    if self.walls[loc + 2] || y % 10 == 0 {
                        draw_pixel(buffer, px, SNAIL_FG);
                    }
                    else {
                        draw_pixel(buffer, px, SNAIL_BG);
                    }

                    for l in ((px + 4)..(px + 4 * 10)).step_by(4) {
                        draw_pixel(buffer, l, SNAIL_BG);
                    }
                }
            }

            // fill end pixel
            let px = 4 * ((by + y) * buffer_width + bx + self.width * 10);
            draw_pixel(buffer, px, SNAIL_FG);
        }

        let px = 4 * ((by + self.height * 10) * buffer_width + bx);
        for l in (px..(px + 4 * (1 + 10 * self.width))).step_by(4) {
            draw_pixel(buffer, l, SNAIL_FG);
        }
    }

    fn draw_foreground(&self, animation_cycle: bool, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        // draw goal
        if animation_cycle {
            const GOAL_IMAGE_SIZE: usize = 7;

            let goal_image = include_bytes!("../../assets/goal_7x7.bin");

            for y in 0..GOAL_IMAGE_SIZE {
                for x in 0..GOAL_IMAGE_SIZE {
                    let goal_px = 4 * (y * GOAL_IMAGE_SIZE + x);
                    let px = 4 * ((by + x + self.end_pos.y * 10 + 2) * buffer_width + bx + y + self.end_pos.x * 10 + 2);

                    if goal_image[goal_px + 3] != 0 {
                        buffer[px] = goal_image[goal_px];
                        buffer[px+1] = goal_image[goal_px + 1];
                        buffer[px+2] = goal_image[goal_px + 2];
                    }
                }
            }
        }
    }
}