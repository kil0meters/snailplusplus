use std::{cmp::Ordering, collections::HashMap};

use crate::{
    direction::Direction,
    lfsr::LFSR,
    maze::Maze,
    snail::Snail,
    solvers::Solver,
    utils::{console_log, draw_pixel, Vec2},
};

struct Mark {
    // [up, down, left, right]
    directions: Vec<u8>,
}

impl Mark {
    fn add_mark(&mut self, direction: Direction) {
        self.directions[direction as usize] += 1;
    }

    fn draw(&self, pos: Vec2, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        let px = 4 * ((by + pos.y * 10) * buffer_width + bx + pos.x * 10);

        if self.directions[0] > 0 {
            let color = if self.directions[0] == 1 {
                [0x00, 0xFF, 0x00]
            } else {
                [0xFF, 0x00, 0x00]
            };

            for index in ((px + 4)..(px + 40)).step_by(8) {
                draw_pixel(buffer, index, color);
            }
        }

        if self.directions[1] > 0 {
            let color = if self.directions[1] == 1 {
                [0x00, 0xFF, 0x00]
            } else {
                [0xFF, 0x00, 0x00]
            };

            for index in ((px + 4 + 40 * buffer_width)..(px + 40 + 40 * buffer_width)).step_by(8) {
                draw_pixel(buffer, index, color);
            }
        }

        if self.directions[2] > 0 {
            let color = if self.directions[2] == 1 {
                [0x00, 0xFF, 0x00]
            } else {
                [0xFF, 0x00, 0x00]
            };

            for index in
                ((px + 4 * buffer_width)..(px + 40 * buffer_width)).step_by(8 * buffer_width)
            {
                draw_pixel(buffer, index, color);
            }
        }

        if self.directions[3] > 0 {
            let color = if self.directions[3] == 1 {
                [0x00, 0xFF, 0x00]
            } else {
                [0xFF, 0x00, 0x00]
            };

            for index in ((px + 4 * buffer_width + 40)..(px + 40 * buffer_width + 40))
                .step_by(8 * buffer_width)
            {
                draw_pixel(buffer, index, color);
            }
        }
    }
}

impl Default for Mark {
    fn default() -> Self {
        Self {
            directions: vec![0; 4],
        }
    }
}

pub struct Tremaux {
    snail: Snail,
    visited: HashMap<Vec2, Mark>,
}

impl Tremaux {
    pub fn new(_upgrades: usize) -> Self {
        Tremaux {
            snail: Snail::new(),
            visited: HashMap::new(),
        }
    }
}

impl Solver for Tremaux {
    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        buffer: &mut [u8],
        buffer_width: usize,
        bx: usize,
        by: usize,
    ) {
        for (pos, mark) in self.visited.iter() {
            mark.draw(*pos, buffer, buffer_width, bx, by);
        }

        self.snail.draw(
            animation_cycle,
            movement_timer,
            buffer,
            buffer_width,
            bx,
            by,
        );
    }

    fn step(&mut self, maze: &Maze, lfsr: &mut LFSR) -> bool {
        let coord = 4 * (self.snail.pos.y * maze.width + self.snail.pos.x);

        let mut valid_directions = vec![];

        if !maze.walls[coord] {
            valid_directions.push(Direction::Up);
        }

        if !maze.walls[coord + 1] {
            valid_directions.push(Direction::Down);
        }

        if !maze.walls[coord + 2] {
            valid_directions.push(Direction::Left);
        }

        if !maze.walls[coord + 3] {
            valid_directions.push(Direction::Right);
        }

        // if in junction
        if valid_directions.len() > 2 {
            let mark = self.visited.entry(self.snail.pos).or_default();
            let back_direction = self.snail.direction.flip();

            // mark square you came from
            mark.add_mark(back_direction);

            // 1. if only the direction we came from is visited, we pick an arbitrary unmarked
            //    entrance
            if mark.directions.iter().sum::<u8>() == 1 {
                let mut choices = vec![];

                for direction in valid_directions.iter() {
                    if *direction != back_direction {
                        choices.push(*direction);
                    }
                }

                self.snail.direction = choices[(lfsr.next() % choices.len() as u16) as usize];
            }
            // 2. Go back where we came from unless it's marked twice
            else if mark.directions[back_direction as usize] < 2 {
                self.snail.direction = back_direction;
            }
            // 3. Pick entrance with fewest marks
            else {
                let mut min_marks = u8::MAX;
                let mut choices = vec![];

                for (direction, num_marks) in mark.directions.iter().enumerate() {
                    if valid_directions.contains(&Direction::from_number(direction))
                        && *num_marks < min_marks
                    {
                        min_marks = *num_marks;
                    }
                }

                for (direction, num_marks) in mark.directions.iter().enumerate() {
                    if valid_directions.contains(&Direction::from_number(direction))
                        && *num_marks == min_marks
                    {
                        choices.push(Direction::from_number(direction));
                    }
                }

                self.snail.direction = choices[(lfsr.next() % choices.len() as u16) as usize];
            }

            // mark the direction the snail is now going
            mark.add_mark(self.snail.direction);
        } else if valid_directions.len() == 2 {
            // make the snail continue along the corridor
            if self.snail.direction.flip() == valid_directions[0] {
                self.snail.direction = valid_directions[1];
            } else {
                self.snail.direction = valid_directions[0]
            }
        }
        // if at dead end
        else if valid_directions.len() == 1 {
            self.snail.direction = valid_directions[0];
        }

        self.snail.move_forward(maze);

        if self.snail.pos == maze.end_pos {
            self.snail.reset();
            self.visited.clear();
            true
        } else {
            false
        }
    }
}
