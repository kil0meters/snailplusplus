mod hold_left;
mod random_walk;
mod direction;

use bit_vec::BitVec;

use crate::{utils::Vec2, lfsr::LFSR, snail_lattice::MazeType};

use self::direction::Direction;

pub const SNAIL_BG: [u8; 3] = [0x11, 0x0A, 0xEF];
pub const SNAIL_FG: [u8; 3] = [0x68, 0x8F, 0xEF];
const SNAIL_MOVEMENT_TIME: usize = 250000;
const ANIMATION_TIME: usize = 500000;

#[derive(Clone)]
pub struct SnailMaze {
    maze_type: MazeType,

    // logical dimensions
    width: usize,
    height: usize,

    // snail coordinates
    snail_pos: Vec2,
    prev_snail_pos: Vec2,
    end_pos: Vec2,

    snail_direction: Direction,

    // stores time since start, used for animations in microseconds
    clock: usize,
    movement_timer: usize,

    // todo use bitset
    maze: BitVec,
}

impl SnailMaze {
    pub fn new(maze_type: MazeType, width: usize, height: usize) -> SnailMaze {
        SnailMaze {
            maze_type,

            width,
            height,

            snail_pos: Vec2 { x: 0, y: 0 },
            prev_snail_pos: Vec2 { x: 0, y: 0 },
            end_pos: Vec2 { x: width - 1, y: height - 1 },
            snail_direction: Direction::Right,

            clock: 0,
            movement_timer: 0,

            // prefill maze, 4 bits per tile, with a filled bit representing a wall
            // 0  0    0    0
            // up down left right
            // TODO: Consider using set_len since this gets redundantly filled with 1s on the first
            // generation
            maze: BitVec::from_elem(width * height * 4, true),
        }
    }

    // TODO: store visited as a bitvec
    fn random_walk(&mut self, x: usize, y: usize, visited: &mut BitVec, lfsr: &mut LFSR) {
        let mut next = Some((x, y));

        while let Some((x, y)) = next {
            visited.set(y * self.width + x, true);
            next = None;

            for direction in lfsr.random_order() {
                // right
                if direction == 0 && x < self.width - 1 && !visited[y * self.width + x + 1] {
                    self.maze.set((y * self.width + x) * 4 + 3, false);
                    self.maze.set((y * self.width + x + 1) * 4 + 2, false);
                    next = Some((x + 1, y));
                }
                // left
                else if direction == 1 && x > 0 && !visited[y * self.width + x - 1] {
                    self.maze.set((y * self.width + x) * 4 + 2, false);
                    self.maze.set((y * self.width + x - 1) * 4 + 3, false);
                    next = Some((x - 1, y));
                }
                // up
                else if direction == 2 && y > 0 && !visited[(y - 1) * self.width + x] {
                    self.maze.set((y * self.width + x) * 4, false);
                    self.maze.set(((y - 1) * self.width + x) * 4 + 1, false);
                    next = Some((x, y - 1));
                }
                // down
                else if direction == 3 && y < self.height - 1 && !visited[(y + 1) * self.width + x] {
                    self.maze.set((y * self.width + x) * 4 + 1, false);
                    self.maze.set(((y + 1) * self.width + x) * 4, false);
                    next = Some((x, y + 1));
                }

                if next.is_some() {
                    break;
                }
            }
        }
    }

    pub fn generate_maze(&mut self, lfsr: &mut LFSR) {
        // prefill vector with
        self.maze.set_all();

        let mut visited = BitVec::from_elem(self.width * self.height, false);

        self.random_walk(0, 0, &mut visited, lfsr);

        for y in 0..self.height {
            for x in 0..self.width {
                if !visited[y * self.width + x] {
                    for direction in [0, 1, 2, 3] {// rng.random_order() {
                        // right
                        if direction == 0 && x < self.width - 1 && visited[y * self.width + x + 1] {
                            self.maze.set((y * self.width + x) * 4 + 3, false);
                            self.maze.set((y * self.width + x + 1) * 4 + 2, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                        // left
                        else if direction == 1 && x > 0 && visited[y * self.width + x - 1] {
                            self.maze.set((y * self.width + x) * 4 + 2, false);
                            self.maze.set((y * self.width + x - 1) * 4 + 3, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                        // up
                        else if direction == 2 && y > 0 && visited[(y - 1) * self.width + x] {
                            self.maze.set((y * self.width + x) * 4, false);
                            self.maze.set(((y - 1) * self.width + x) * 4 + 1, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                        // down
                        else if direction == 3 && y < self.height - 1 && visited[(y + 1) * self.width + x] {
                            self.maze.set((y * self.width + x) * 4 + 1, false);
                            self.maze.set(((y + 1) * self.width + x) * 4, false);
                            self.random_walk(x, y, &mut visited, lfsr);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn render_maze(&mut self, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        for y in 0..(self.height * 10) {
            for x in 0..self.width {
                let loc = 4*((y / 10) * self.width + x);
                let px = ((by + y) * buffer_width + bx + (x * 10)) * 4;

                // Checking the bottom wall is redundant
                if y % 10 == 0 && self.maze[loc] {
                    for l in (px..(px + 4 * 10)).step_by(4) {
                        draw_pixel(buffer, l, SNAIL_FG);
                    }
                }

                else {
                    // if left wall, checking right wall is redundant
                    if self.maze[loc + 2] || y % 10 == 0 {
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

    pub fn render_foreground(&mut self, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        let snail_image = if (self.clock / ANIMATION_TIME) % 2 == 0 {
            include_bytes!("../../../assets/snail1_8x8.bin")
        } else {
            include_bytes!("../../../assets/snail2_8x8.bin")
        };

        let offset_y = if self.prev_snail_pos.y != self.snail_pos.y {
            discrete_lerp(
                (self.prev_snail_pos.y * 10) as i32,
                (self.snail_pos.y * 10) as i32,
                self.movement_timer as i32,
                SNAIL_MOVEMENT_TIME as i32
            )
        } else {
            (self.snail_pos.y * 10) as i32
        };

        let offset_x = if self.prev_snail_pos.x != self.snail_pos.x {
            discrete_lerp(
                (self.prev_snail_pos.x * 10) as i32,
                (self.snail_pos.x * 10) as i32,
                self.movement_timer as i32,
                SNAIL_MOVEMENT_TIME as i32
            )
        } else {
            (self.snail_pos.x * 10) as i32
        };

        const SNAIL_IMAGE_SIZE: usize = 8;

        // draw goal
        for y in 0..SNAIL_IMAGE_SIZE {
            for x in 0..SNAIL_IMAGE_SIZE {
                let snail_px = 4*(y * SNAIL_IMAGE_SIZE + x);
                // only draw if not transparent
                if snail_image[snail_px + 3] != 0 {
                    // I'm so, so, sorry.
                    let px = match self.snail_direction {
                        Direction::Up => 4 * (((by + (SNAIL_IMAGE_SIZE - y)) as i32 + offset_y) as usize * buffer_width + bx + x + self.snail_pos.x * 10 + 2),
                        Direction::Down => 4 * (((by + y + 2) as i32 + offset_y) as usize * buffer_width + bx + (SNAIL_IMAGE_SIZE - x) + self.snail_pos.x * 10),
                        Direction::Left => 4 * ((by + x + self.snail_pos.y * 10 + 2) * buffer_width + ((bx + (SNAIL_IMAGE_SIZE - y)) as i32 + offset_x) as usize),
                        Direction::Right => 4 * ((by + x + self.snail_pos.y * 10 + 2) * buffer_width + ((bx + y + 2) as i32 + offset_x) as usize),
                    };

                    buffer[px] = snail_image[snail_px];
                    buffer[px+1] = snail_image[snail_px + 1];
                    buffer[px+2] = snail_image[snail_px + 2];
                }
            }
        }

        if (self.clock / ANIMATION_TIME) % 2 == 0 {
            const GOAL_IMAGE_SIZE: usize = 7;

            let goal_image = include_bytes!("../../../assets/goal_7x7.bin");

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

    fn move_forward(&mut self) -> bool {
        let coord = 4 * (self.snail_pos.y * self.width + self.snail_pos.x);
        self.prev_snail_pos = self.snail_pos;

        match self.snail_direction {
            Direction::Up => if !self.maze[coord] {
                self.snail_pos.y -= 1;
                return true;
            },
            Direction::Down => if !self.maze[coord + 1] {
                self.snail_pos.y += 1;
                return true;
            },
            Direction::Left => if !self.maze[coord + 2] {
                self.snail_pos.x -= 1;
                return true;
            },
            Direction::Right => if !self.maze[coord + 3] {
                self.snail_pos.x += 1;
                return true;
            }
        }

        return false;
    }

    fn move_snail(&mut self, lfsr: &mut LFSR) {
        match self.maze_type {
            MazeType::RandomWalk => self.ai_random_walk(lfsr),
            MazeType::HoldLeft => self.ai_hold_left(),
            MazeType::Tremaux => self.ai_random_walk(lfsr),
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
            self.move_snail(lfsr);

            if self.snail_pos == self.end_pos {
                total += self.width * self.height;
                self.generate_maze(lfsr);
                self.snail_pos.x = 0;
                self.snail_pos.y = 0;
                self.movement_timer = SNAIL_MOVEMENT_TIME;
                self.prev_snail_pos = self.snail_pos;
            }
        }

        total
    }
}

// discrete linear interpolation
// returns a linear intepolation between v1 and v2 baded on fact1/fact2
fn discrete_lerp(v1: i32, v2: i32, fact1: i32, fact2: i32) -> i32 {
    let difference = v2 - v1;
    let add = (fact1 * difference) / fact2;
    v1 + add
}

fn draw_pixel(buffer: &mut [u8], index: usize, pixel: [u8; 3]) {
    buffer[index] = pixel[0];
    buffer[index + 1] = pixel[1];
    buffer[index + 2] = pixel[2];
    buffer[index + 3] = 0xFF;
}
