use std::{collections::VecDeque, mem::size_of};

use crate::{
    direction::Direction,
    image::Image,
    lattice::TilableMaze,
    lfsr::LFSR,
    solvers::Solver,
    utils::{console_log, Vec2},
};

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

impl<const S: usize, T: Solver<S>> TilableMaze for AutoMaze<S, T>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    const SIZE: usize = S;

    fn new() -> AutoMaze<S, T> {
        AutoMaze {
            solver: T::new(),
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
    fn tick(&mut self, dt: usize, lfsr: &mut LFSR) -> usize {
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
                self.generate(lfsr);
                self.movement_timer = movement_time;
            }
        }

        total
    }

    fn set_upgrades(&mut self, upgrades: u32) {
        self.solver.set_upgrades(upgrades);
    }

    fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image, bx: usize, by: usize) {
        let animation_cycle = (self.clock / ANIMATION_TIME) % 2 == 0;

        // draw "snail"
        self.solver
            .draw(animation_cycle, self.movement_timer, lfsr, image, bx, by);
        self.maze
            .draw_foreground(T::palette()[0], animation_cycle, image, bx, by);
    }

    fn draw_background(&mut self, image: &mut Image, bx: usize, by: usize) {
        self.maze
            .draw_background(T::palette()[4], T::palette()[5], image, bx, by);
    }

    fn generate(&mut self, lfsr: &mut LFSR) {
        self.maze.generate(lfsr);
        self.solver.setup(&self.maze, lfsr);
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

    pub fn get_distances(&self, x: usize, y: usize, distances: &mut [usize; S * S]) {
        let mut queue = VecDeque::new();
        *distances = [0; S * S];

        queue.push_back((x, y));

        while let Some((x, y)) = queue.pop_front() {
            let cell = self.get_cell(x, y);
            let distance = distances[y * S + x];

            if !cell.has_wall(Direction::Up) && distances[(y - 1) * S + x] == 0 {
                queue.push_back((x, y - 1));
                distances[(y - 1) * S + x] = distance + 1;
            }

            if !cell.has_wall(Direction::Down) && distances[(y + 1) * S + x] == 0 {
                queue.push_back((x, y + 1));
                distances[(y + 1) * S + x] = distance + 1;
            }

            if !cell.has_wall(Direction::Left) && distances[y * S + x - 1] == 0 {
                queue.push_back((x - 1, y));
                distances[y * S + x - 1] = distance + 1;
            }

            if !cell.has_wall(Direction::Right) && distances[y * S + x + 1] == 0 {
                queue.push_back((x + 1, y));
                distances[y * S + x + 1] = distance + 1;
            }
        }
    }

    pub fn get_directions(&self, source: Vec2) -> [Direction; S * S] {
        let mut visited = [false; S * S];
        let mut directions = [Direction::Left; S * S];

        let mut queue = VecDeque::new();
        queue.push_back((source.x, source.y));

        while let Some((x, y)) = queue.pop_front() {
            let cell = self.get_cell(x, y);
            if !cell.has_wall(Direction::Up) && !visited[(y - 1) * S + x] {
                queue.push_back((x, y - 1));
                visited[(y - 1) * S + x] = true;
                directions[(y - 1) * S + x] = Direction::Down;
            }

            if !cell.has_wall(Direction::Down) && !visited[(y + 1) * S + x] {
                queue.push_back((x, y + 1));
                visited[(y + 1) * S + x] = true;
                directions[(y + 1) * S + x] = Direction::Up;
            }

            if !cell.has_wall(Direction::Left) && !visited[y * S + x - 1] {
                queue.push_back((x - 1, y));
                visited[y * S + x - 1] = true;
                directions[y * S + x - 1] = Direction::Right;
            }

            if !cell.has_wall(Direction::Right) && !visited[y * S + x + 1] {
                queue.push_back((x + 1, y));
                visited[y * S + x + 1] = true;
                directions[y * S + x + 1] = Direction::Left;
            }
        }

        directions
    }

    pub fn get_solve_sequence(&self, x: usize, y: usize, target: Vec2) -> Vec<Direction> {
        let directions = self.get_directions(target);

        let mut pos = Vec2 { x, y };
        let mut moves = vec![];

        while pos != target {
            match directions[pos.y * S + pos.x] {
                Direction::Up => {
                    pos.y -= 1;
                    moves.push(Direction::Up);
                }
                Direction::Down => {
                    pos.y += 1;
                    moves.push(Direction::Down);
                }
                Direction::Left => {
                    pos.x -= 1;
                    moves.push(Direction::Left);
                }
                Direction::Right => {
                    pos.x += 1;
                    moves.push(Direction::Right);
                }
            }
        }

        moves
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

    pub fn draw_background(
        &mut self,
        fg_color: [u8; 3],
        bg_color: [u8; 3],
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        for y in 0..(S * 10) {
            for x in 0..S {
                let cell = self.get_cell(x, y / 10);
                let px = ((by + y) * image.buffer_width + bx + (x * 10)) * 4;

                // Checking the bottom wall is redundant
                if y % 10 == 0 && cell.has_wall(Direction::Up) {
                    for l in (px..(px + 4 * 10)).step_by(4) {
                        image.draw_pixel(l, fg_color);
                    }
                } else {
                    // if left wall, checking right wall is redundant
                    if cell.has_wall(Direction::Left) || y % 10 == 0 {
                        image.draw_pixel(px, fg_color);
                    } else {
                        image.draw_pixel(px, bg_color);
                    }

                    for l in ((px + 4)..(px + 4 * 10)).step_by(4) {
                        image.draw_pixel(l, bg_color);
                    }
                }
            }

            // fill end pixel
            let px = 4 * ((by + y) * image.buffer_width + bx + S * 10);
            image.draw_pixel(px, fg_color);
        }

        let px = 4 * ((by + S * 10) * image.buffer_width + bx);
        for l in (px..(px + 4 * (1 + 10 * S))).step_by(4) {
            image.draw_pixel(l, fg_color);
        }
    }

    fn draw_foreground(
        &self,
        goal_color: [u8; 3],
        animation_cycle: bool,
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        // draw goal
        if animation_cycle {
            const GOAL_IMAGE_SIZE: usize = 7;

            let goal_image = include_bytes!("../../assets/goal_7x7.bin");

            for y in 0..GOAL_IMAGE_SIZE {
                for x in 0..GOAL_IMAGE_SIZE {
                    let goal_px = y * GOAL_IMAGE_SIZE + x;
                    let px = 4
                        * ((by + x + self.end_pos.y * 10 + 2) * image.buffer_width
                            + bx
                            + y
                            + self.end_pos.x * 10
                            + 2);

                    // not transparent
                    if goal_image[goal_px] != 255 {
                        image.buffer[px] = goal_color[0];
                        image.buffer[px + 1] = goal_color[1];
                        image.buffer[px + 2] = goal_color[2];
                    }
                }
            }
        }
    }
}
