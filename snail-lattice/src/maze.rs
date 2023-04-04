use std::{collections::VecDeque, mem::size_of};

use crate::{
    direction::Direction,
    image::Image,
    lattice::TilableMaze,
    lfsr::LFSR,
    solvers::{SolveStatus, Solver},
    utils::Vec2,
};

pub const SNAIL_MOVEMENT_TIME: f32 = 250.0;
pub const ANIMATION_TIME: f32 = 500.0;

// each cell is 4 bits, so 2 cells per byte
pub const CELLS_PER_IDX: usize = size_of::<u32>() * 2;

pub struct MazeCell(pub u32);

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

pub struct AutoMaze {
    solver: Box<dyn Solver>,

    // stores time since start, in milliseconds
    clock: f32,

    // time since last movement
    movement_timer: f32,

    pub maze: Maze,
}

impl AutoMaze {
    pub fn new(solver: Box<dyn Solver>, size: usize) -> AutoMaze {
        AutoMaze {
            solver,
            clock: 0.0,
            movement_timer: 0.0,

            maze: Maze::new(size),
        }
    }
}

impl TilableMaze for AutoMaze {
    fn size(&self) -> usize {
        self.maze.size
    }

    // progresses time a certain number of microseconds
    // notably, no rendering happens when we tick the time
    // returns true if the tick results in a new maze to be generated
    fn tick(&mut self, mut dt: f32, lfsr: &mut LFSR) -> SolveStatus {
        self.clock += dt;
        let mut total = 0;
        let mut rerender = false;

        dt += self.movement_timer;

        while dt > self.solver.movement_time() {
            let movement_time = self.solver.movement_time();
            dt -= movement_time;

            match self.solver.step(&mut self.maze, lfsr) {
                SolveStatus::Solved(count) => {
                    total += count;
                    self.movement_timer = movement_time;
                    self.generate(lfsr);
                }
                SolveStatus::Rerender => rerender = true,
                SolveStatus::None => {}
            }
        }

        self.movement_timer = dt;

        match (total, rerender) {
            (0, true) => SolveStatus::Rerender,
            (0, false) => SolveStatus::None,
            (num, _) => SolveStatus::Solved(num),
        }
    }

    fn set_upgrades(&mut self, upgrades: u32) {
        self.solver.set_upgrades(upgrades);
    }

    fn draw_foreground(&mut self, lfsr: &mut LFSR, image: &mut Image) {
        let animation_cycle = (self.clock / ANIMATION_TIME).round() as usize % 2 == 0;

        // draw "snail"
        self.solver.draw(
            animation_cycle,
            self.movement_timer,
            &self.maze,
            lfsr,
            image,
        );

        if !self.solver.custom_goal() {
            self.maze
                .draw_foreground(self.solver.palette()[0], animation_cycle, image);
        }
    }

    fn draw_background(&mut self, image: &mut Image) {
        self.maze
            .draw_background(self.solver.palette()[4], self.solver.palette()[5], image);
    }

    fn generate(&mut self, lfsr: &mut LFSR) {
        self.maze.generate(lfsr);
        self.solver.setup(&self.maze, lfsr);
    }
}

// An SxS maze
pub struct Maze {
    pub end_pos: Vec2,
    pub size: usize,

    // each cell is 4 bits
    pub walls: Vec<u32>,
}

impl Maze {
    pub fn new(size: usize) -> Self {
        Maze {
            size,
            end_pos: Vec2 {
                x: size - 1,
                y: size - 1,
            },
            walls: vec![],
        }
    }

    pub fn remove_wall(&mut self, x: usize, y: usize, direction: Direction) {
        let cell = self.get_cell(x, y);

        if cell.has_wall(direction)
            && !(direction == Direction::Left && x == 0)
            && !(direction == Direction::Up && y == 0)
            && !(direction == Direction::Right && x == self.size - 1)
            && !(direction == Direction::Down && y == self.size - 1)
        {
            self.set_wall(x, y, direction);
        }
    }

    pub fn add_wall(&mut self, x: usize, y: usize, direction: Direction) {
        let cell = self.get_cell(x, y);

        if !cell.has_wall(direction) {
            self.set_wall(x, y, direction);
        }
    }

    pub fn set_wall(&mut self, x: usize, y: usize, direction: Direction) {
        self.xor_cell(x, y, direction.to_wall());
        match direction {
            Direction::Up if y > 0 => self.xor_cell(x, y - 1, direction.flip().to_wall()),
            Direction::Down if y < self.size - 1 => {
                self.xor_cell(x, y + 1, direction.flip().to_wall())
            }
            Direction::Left if x > 0 => self.xor_cell(x - 1, y, direction.flip().to_wall()),
            Direction::Right if x < self.size - 1 => {
                self.xor_cell(x + 1, y, direction.flip().to_wall())
            }
            _ => {}
        }
    }

    // pub fn set_cell_checked(&mut self, x: usize, y: usize, data: usize) {
    //     if x >= S || y >= S {
    //         return;
    //     }
    //
    //     let offset = y * S + x;
    //     let shift_amount = 4 * (CELLS_PER_IDX - (offset % CELLS_PER_IDX) - 1);
    //
    //     let mask = usize::MAX & !(0b1111 << shift_amount);
    //
    //     // and + or has the desired effect here
    //     self.walls[offset / CELLS_PER_IDX] &= mask;
    //     self.walls[offset / CELLS_PER_IDX] |= data << shift_amount;
    // }

    // 4 bytes
    pub fn xor_cell(&mut self, x: usize, y: usize, data: u8) {
        let offset = y * self.size + x;

        self.walls[offset / CELLS_PER_IDX] ^=
            (data as u32) << (4 * (CELLS_PER_IDX - (offset % CELLS_PER_IDX) - 1));
    }

    pub fn get_cell(&self, x: usize, y: usize) -> MazeCell {
        let offset = y * self.size + x;

        MazeCell(
            self.walls[offset / CELLS_PER_IDX]
                >> (4 * (CELLS_PER_IDX - (offset % CELLS_PER_IDX) - 1))
                & 0b1111,
        )
    }

    fn set_cell_wall(&mut self, x: usize, y: usize, direction: Direction) {
        self.xor_cell(x, y, 1 << (3 - direction as usize));
    }

    fn random_walk(&mut self, x: usize, y: usize, lfsr: &mut LFSR, visited: &mut Vec<bool>) {
        let mut next = Some((x, y));

        let s = self.size;

        while let Some((x, y)) = next {
            visited[y * s + x] = true;
            next = None;

            for direction in lfsr.random_order() {
                // right
                if direction == 0 && x < s - 1 && !visited[y * s + x + 1] {
                    self.set_cell_wall(x, y, Direction::Right);
                    self.set_cell_wall(x + 1, y, Direction::Left);
                    next = Some((x + 1, y));
                }
                // left
                else if direction == 1 && x > 0 && !visited[y * s + x - 1] {
                    self.set_cell_wall(x, y, Direction::Left);
                    self.set_cell_wall(x - 1, y, Direction::Right);
                    next = Some((x - 1, y));
                }
                // up
                else if direction == 2 && y > 0 && !visited[(y - 1) * s + x] {
                    self.set_cell_wall(x, y, Direction::Up);
                    self.set_cell_wall(x, y - 1, Direction::Down);
                    next = Some((x, y - 1));
                }
                // down
                else if direction == 3 && y < s - 1 && !visited[(y + 1) * s + x] {
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

    /// distances assumed to be of length self.size * self.size
    pub fn get_distances(&self, x: usize, y: usize, distances: &mut Vec<usize>) {
        let s = self.size;

        let mut queue = VecDeque::new();
        distances.fill(0);
        distances.resize(s * s, 0);

        queue.push_back((x, y));

        while let Some((x, y)) = queue.pop_front() {
            let cell = self.get_cell(x, y);
            let distance = distances[y * s + x];

            if !cell.has_wall(Direction::Up) && distances[(y - 1) * s + x] == 0 {
                queue.push_back((x, y - 1));
                distances[(y - 1) * s + x] = distance + 1;
            }

            if !cell.has_wall(Direction::Down) && distances[(y + 1) * s + x] == 0 {
                queue.push_back((x, y + 1));
                distances[(y + 1) * s + x] = distance + 1;
            }

            if !cell.has_wall(Direction::Left) && distances[y * s + x - 1] == 0 {
                queue.push_back((x - 1, y));
                distances[y * s + x - 1] = distance + 1;
            }

            if !cell.has_wall(Direction::Right) && distances[y * s + x + 1] == 0 {
                queue.push_back((x + 1, y));
                distances[y * s + x + 1] = distance + 1;
            }
        }
    }

    pub fn get_directions(&self, source: Vec2, directions: &mut Vec<Option<Direction>>) {
        let s = self.size;
        let mut visited = vec![false; s * s];

        directions.fill(None);
        directions.resize(s * s, None);

        let mut queue = VecDeque::new();
        queue.push_back((source.x, source.y));

        while let Some((x, y)) = queue.pop_front() {
            let cell = self.get_cell(x, y);
            if !cell.has_wall(Direction::Up) && !visited[(y - 1) * s + x] {
                queue.push_back((x, y - 1));
                visited[(y - 1) * s + x] = true;
                directions[(y - 1) * s + x] = Some(Direction::Down);
            }

            if !cell.has_wall(Direction::Down) && !visited[(y + 1) * s + x] {
                queue.push_back((x, y + 1));
                visited[(y + 1) * s + x] = true;
                directions[(y + 1) * s + x] = Some(Direction::Up);
            }

            if !cell.has_wall(Direction::Left) && !visited[y * s + x - 1] {
                queue.push_back((x - 1, y));
                visited[y * s + x - 1] = true;
                directions[y * s + x - 1] = Some(Direction::Right);
            }

            if !cell.has_wall(Direction::Right) && !visited[y * s + x + 1] {
                queue.push_back((x + 1, y));
                visited[y * s + x + 1] = true;
                directions[y * s + x + 1] = Some(Direction::Left);
            }
        }
    }

    pub fn get_solve_sequence(&self, x: usize, y: usize, target: Vec2) -> Vec<Direction> {
        let mut directions = vec![];
        self.get_directions(target, &mut directions);

        let mut pos = Vec2 { x, y };
        let mut moves = vec![];

        while pos != target {
            match directions[pos.y * self.size + pos.x] {
                Some(Direction::Up) => {
                    pos.y -= 1;
                    moves.push(Direction::Up);
                }
                Some(Direction::Down) => {
                    pos.y += 1;
                    moves.push(Direction::Down);
                }
                Some(Direction::Left) => {
                    pos.x -= 1;
                    moves.push(Direction::Left);
                }
                Some(Direction::Right) => {
                    pos.x += 1;
                    moves.push(Direction::Right);
                }

                None => {
                    return vec![];
                }
            }
        }

        moves
    }

    pub fn generate(&mut self, lfsr: &mut LFSR) {
        let s = self.size;

        // set all elements in vector to 1s
        self.walls.fill(!0u32);
        self.walls.resize(s * s, !0u32);

        let mut visited = vec![false; s * s];

        self.random_walk(0, 0, lfsr, &mut visited);

        for y in 0..s {
            for x in 0..s {
                if !visited[y * s + x] {
                    for direction in lfsr.random_order() {
                        // right
                        if direction == 0 && x < s - 1 && visited[y * s + x + 1] {
                            self.set_cell_wall(x, y, Direction::Right);
                            self.set_cell_wall(x + 1, y, Direction::Left);
                            self.random_walk(x, y, lfsr, &mut visited);
                            break;
                        }
                        // left
                        else if direction == 1 && x > 0 && visited[y * s + x - 1] {
                            self.set_cell_wall(x, y, Direction::Left);
                            self.set_cell_wall(x - 1, y, Direction::Right);
                            self.random_walk(x, y, lfsr, &mut visited);
                            break;
                        }
                        // up
                        else if direction == 2 && y > 0 && visited[(y - 1) * s + x] {
                            self.set_cell_wall(x, y, Direction::Up);
                            self.set_cell_wall(x, y - 1, Direction::Down);
                            self.random_walk(x, y, lfsr, &mut visited);
                            break;
                        }
                        // down
                        else if direction == 3 && y < s - 1 && visited[(y + 1) * s + x] {
                            self.set_cell_wall(x, y, Direction::Down);
                            self.set_cell_wall(x, y + 1, Direction::Up);
                            self.random_walk(x, y, lfsr, &mut visited);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_background(&self, fg_color: [u8; 3], bg_color: [u8; 3], image: &mut Image) {
        let s = self.size;
        let by = image.by;
        let bx = image.bx;

        for y in 0..(s * 10) {
            for x in 0..s {
                let cell = self.get_cell(x, y / 10);
                let px = ((by + y) * image.width + bx + (x * 10)) * 4;

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
            let px = 4 * ((by + y) * image.width + bx + s * 10);
            image.draw_pixel(px, fg_color);
        }

        let px = 4 * ((by + s * 10) * image.width + bx);
        for l in (px..(px + 4 * (1 + 10 * s))).step_by(4) {
            image.draw_pixel(l, fg_color);
        }
    }

    fn draw_foreground(&self, goal_color: [u8; 3], animation_cycle: bool, image: &mut Image) {
        // draw goal
        if animation_cycle {
            image.draw_goal(goal_color, self.end_pos.x * 10, self.end_pos.y * 10);
        }
    }
}
