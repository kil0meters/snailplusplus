use std::collections::HashMap;

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
    utils::Vec2,
};

pub struct Mark {
    // [up, down, left, right]
    pub directions: Vec<u8>,
}

impl Mark {
    pub fn add_mark(&mut self, direction: Direction) {
        self.directions[direction as usize] += 1;
    }

    fn get_color(mark_value: u8) -> [u8; 3] {
        if mark_value == 1 {
            [0x00, 0xFF, 0x00]
        } else {
            [0xFF, 0x00, 0x00]
        }
    }

    fn draw(&self, pos: Vec2, image: &mut Image, bx: usize, by: usize) {
        let px = 4 * ((by + pos.y * 10) * image.buffer_width + bx + pos.x * 10);

        if self.directions[0] > 0 {
            for index in ((px + 4)..(px + 40)).step_by(8) {
                image.draw_pixel(index, Mark::get_color(self.directions[0]));
            }
        }

        if self.directions[1] > 0 {
            for index in
                ((px + 4 + 40 * image.buffer_width)..(px + 40 + 40 * image.buffer_width)).step_by(8)
            {
                image.draw_pixel(index, Mark::get_color(self.directions[1]));
            }
        }

        if self.directions[2] > 0 {
            for index in ((px + 4 * image.buffer_width)..(px + 40 * image.buffer_width))
                .step_by(8 * image.buffer_width)
            {
                image.draw_pixel(index, Mark::get_color(self.directions[2]));
            }
        }

        if self.directions[3] > 0 {
            for index in ((px + 4 * image.buffer_width + 40)..(px + 40 * image.buffer_width + 40))
                .step_by(8 * image.buffer_width)
            {
                image.draw_pixel(index, Mark::get_color(self.directions[3]));
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

/// Segment Snail Upgrades:
/// - Compass:       Using a compass, the segment snail can sometimes make smarter decisions about where to turn.
/// - Electromagnet: Installs an electromagnet near the goal to make Segment Snails compass more accurate.
/// - Breadcrumbs:   Segment Snail is twice as fast while backtracking.

pub struct Tremaux<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    pub snail: Snail<S>,
    pub visited: HashMap<Vec2, Mark>,
    is_backtracking: bool,
    upgrades: u32,
    directions: [Option<Direction>; S * S],
    movement_time: f32,
}

impl<const S: usize> Tremaux<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    pub fn set_movement_time(&mut self, movement_time: f32) {
        self.movement_time = movement_time;
    }
}

impl<const S: usize> Solver<S> for Tremaux<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        Tremaux {
            snail: Snail::new(),
            visited: HashMap::new(),
            upgrades: 0,
            directions: [None; S * S],
            is_backtracking: false,
            movement_time: SNAIL_MOVEMENT_TIME,
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
        for (pos, mark) in self.visited.iter() {
            mark.draw(*pos, image, bx, by);
        }

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            movement_timer / self.movement_time(),
            image,
            bx,
            by,
        );
    }

    fn setup(&mut self, maze: &Maze<S>, _lfsr: &mut LFSR) {
        self.snail.reset();
        self.visited.clear();
        self.directions = maze.get_directions(maze.end_pos);
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
        let cell = maze.get_cell(self.snail.pos.x, self.snail.pos.y);
        let valid_directions = cell.valid_directions();

        // if in junction
        if valid_directions.len() > 2 {
            self.is_backtracking = false;

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

                for (direction, num_marks) in mark.directions.iter().enumerate() {
                    if !cell.has_wall(Direction::from_number(direction)) && *num_marks < min_marks {
                        min_marks = *num_marks;
                    }
                }

                let choices: Vec<Direction> = mark
                    .directions
                    .iter()
                    .enumerate()
                    .filter_map(|(direction, num_marks)| {
                        (!cell.has_wall(Direction::from_number(direction))
                            && *num_marks == min_marks)
                            .then(|| Direction::from_number(direction))
                    })
                    .collect();

                if choices.len() == 1 && mark.directions[choices[0] as usize] == 1 {
                    self.is_backtracking = true;
                }

                let odds = (self.upgrades & 0b11) << 1;
                if odds > 0 && lfsr.big() % 12 < odds as usize {
                    self.snail.direction =
                        self.directions[self.snail.pos.y * S + self.snail.pos.x].unwrap();
                } else {
                    self.snail.direction = choices[(lfsr.next() % choices.len() as u16) as usize];
                }
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
            self.is_backtracking = true;
        }

        self.snail.move_forward(maze);

        self.snail.pos == maze.end_pos
    }

    fn movement_time(&self) -> f32 {
        if self.is_backtracking && (self.upgrades & 0b100) != 0 {
            self.movement_time / 2.0
        } else {
            self.movement_time
        }
    }
}
