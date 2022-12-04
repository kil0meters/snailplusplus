use crate::{
    direction::Direction,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::Snail,
    solvers::Solver,
    utils::{draw_rectangle_with, Vec2},
};

use super::Tremaux;

const TIME_TRAVEL_SPEED_FACTOR: usize = 8;
const TIME_TRAVEL_MOVEMENT_TIME: usize = SNAIL_MOVEMENT_TIME / TIME_TRAVEL_SPEED_FACTOR;

fn random_color(lfsr: &mut LFSR) -> [u8; 3] {
    if lfsr.next() == 3 {
        [0xFF, 0x00, 0x00]
    } else {
        [0x00, 0x00, 0x00]
    }
}

struct PathTile {
    pos: Vec2,
    directions: [bool; 4],
}

impl PathTile {
    fn draw(&self, lfsr: &mut LFSR, buffer: &mut [u8], buffer_width: usize, bx: usize, by: usize) {
        match self.directions {
            // up down
            [true, true, false, false] => {
                draw_rectangle_with(
                    self.pos.x * 10 + 4,
                    self.pos.y * 10,
                    3,
                    10,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }
            // left right
            [false, false, true, true] => {
                draw_rectangle_with(
                    self.pos.x * 10,
                    self.pos.y * 10 + 4,
                    10,
                    3,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }

            // up right
            [true, false, false, true] => {
                draw_rectangle_with(
                    self.pos.x * 10 + 4,
                    self.pos.y * 10,
                    3,
                    7,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );

                draw_rectangle_with(
                    self.pos.x * 10 + 7,
                    self.pos.y * 10 + 4,
                    4,
                    3,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }

            // up left
            [true, false, true, false] => {
                draw_rectangle_with(
                    self.pos.x * 10 + 4,
                    self.pos.y * 10,
                    3,
                    7,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );

                draw_rectangle_with(
                    self.pos.x * 10,
                    self.pos.y * 10 + 4,
                    4,
                    3,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }

            // down right
            [false, true, false, true] => {
                draw_rectangle_with(
                    self.pos.x * 10 + 4,
                    self.pos.y * 10 + 4,
                    3,
                    7,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );

                draw_rectangle_with(
                    self.pos.x * 10 + 7,
                    self.pos.y * 10 + 4,
                    4,
                    3,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }

            // down left
            [false, true, true, false] => {
                draw_rectangle_with(
                    self.pos.x * 10 + 4,
                    self.pos.y * 10 + 4,
                    3,
                    7,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );

                draw_rectangle_with(
                    self.pos.x * 10,
                    self.pos.y * 10 + 4,
                    4,
                    3,
                    || random_color(lfsr),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }
            _ => {}
        }
    }

    fn new(pos: Vec2, dir: Direction) -> PathTile {
        let mut directions = [false; 4];
        directions[dir as usize] = true;

        Self { pos, directions }
    }
}

enum TimeTravelState {
    TimeTraveling,
    DrawingPath,
    Normal,
}

pub struct TimeTravel {
    snail: Snail,
    state: TimeTravelState,
    path: Vec<PathTile>,

    path_drawer: Snail,
    time_traveler: Tremaux,
    time_dilation_timer: usize,
}

impl TimeTravel {
    pub fn new(_upgrades: usize) -> Self {
        let mut path_drawer = Snail::new();
        path_drawer.active = false;

        let mut time_traveler = Tremaux::new(0).set_movement_time(TIME_TRAVEL_MOVEMENT_TIME);
        time_traveler.snail.active = false;

        TimeTravel {
            snail: Snail::new(),
            state: TimeTravelState::TimeTraveling,
            path: vec![],

            path_drawer,
            time_traveler,
            time_dilation_timer: 0,
        }
    }
}

impl Solver for TimeTravel {
    fn step(&mut self, maze: &Maze, lfsr: &mut LFSR) -> bool {
        match self.state {
            TimeTravelState::TimeTraveling => {
                if self.time_traveler.step(maze, lfsr) {
                    self.state = TimeTravelState::DrawingPath;
                    self.path_drawer.pos = maze.end_pos;
                    self.path_drawer.prev_pos = maze.end_pos;
                    self.path_drawer.direction = self.time_traveler.snail.direction.flip();
                }
            }
            TimeTravelState::DrawingPath => {
                let coord = 4 * (self.path_drawer.pos.y * maze.width + self.path_drawer.pos.x);

                let walls = &maze.walls[coord..(coord + 4)];

                let valid_directions: Vec<Direction> = walls
                    .iter()
                    .enumerate()
                    .filter_map(|(i, has_wall)| (!has_wall).then(|| Direction::from_number(i)))
                    .collect();

                // if in junction
                if valid_directions.len() > 2 {
                    let mark = self
                        .time_traveler
                        .visited
                        .entry(self.path_drawer.pos)
                        .or_default();
                    let back_direction = self.path_drawer.direction.flip();

                    let direction = valid_directions
                        .iter()
                        .filter(|direction| {
                            **direction != back_direction
                                && mark.directions[**direction as usize] == 1
                        })
                        .next()
                        .unwrap();

                    self.path_drawer.direction = *direction;
                }
                // if in corridor, keep along
                else if valid_directions.len() == 2 && walls[self.path_drawer.direction as usize]
                {
                    // make the path_drawer continue along the corridor
                    if self.path_drawer.direction.flip() == valid_directions[0] {
                        self.path_drawer.direction = valid_directions[1];
                    } else {
                        self.path_drawer.direction = valid_directions[0]
                    }
                }
                // in dead end, turn around
                /* else {
                    self.path_drawer.direction = self.path_drawer.direction.flip();
                } */

                // draw path
                if let Some(prev) = self.path.last_mut() {
                    prev.directions[self.path_drawer.direction as usize] = true;
                }

                assert!(self.path_drawer.move_forward(maze));

                self.path.push(PathTile::new(
                    self.path_drawer.pos,
                    self.path_drawer.direction.flip(),
                ));

                if self.path_drawer.pos.x == 0 && self.path_drawer.pos.y == 0 {
                    self.state = TimeTravelState::Normal;
                }
            }
            TimeTravelState::Normal => {
                self.time_dilation_timer += 1;

                if self.time_dilation_timer % TIME_TRAVEL_SPEED_FACTOR == 0 {
                    self.time_dilation_timer = 0;

                    // just follow the path corridor
                    if let Some(tile) = self.path.pop() {
                        if !tile.directions[self.snail.direction as usize] {
                            let right_rotate = self.snail.direction.rotate();
                            let left_rotate = self.snail.direction.rotate_counter();
                            if tile.directions[right_rotate as usize] {
                                self.snail.direction = right_rotate;
                            } else if tile.directions[left_rotate as usize] {
                                self.snail.direction = left_rotate;
                            }
                        }
                    }

                    assert!(self.snail.move_forward(maze));

                    if self.snail.pos == maze.end_pos {
                        self.state = TimeTravelState::TimeTraveling;
                        self.time_traveler.visited.clear();
                        self.snail.reset();
                        self.path_drawer.reset();

                        return true;
                    }
                }
            }
        }

        false
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        maze: &Maze,
        lfsr: &mut LFSR,
        buffer: &mut [u8],
        buffer_width: usize,
        bx: usize,
        by: usize,
    ) {
        match self.state {
            TimeTravelState::TimeTraveling => {
                self.snail
                    .draw(true, 0, SNAIL_MOVEMENT_TIME, buffer, buffer_width, bx, by);

                self.time_traveler.draw(
                    animation_cycle,
                    movement_timer,
                    maze,
                    lfsr,
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }
            TimeTravelState::DrawingPath => {
                for tile in &self.path {
                    tile.draw(lfsr, buffer, buffer_width, bx, by);
                }

                self.snail
                    .draw(true, 0, SNAIL_MOVEMENT_TIME, buffer, buffer_width, bx, by);

                self.path_drawer.draw(
                    animation_cycle,
                    movement_timer,
                    self.movement_time(),
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }
            TimeTravelState::Normal => {
                for tile in &self.path {
                    tile.draw(lfsr, buffer, buffer_width, bx, by);
                }

                self.snail.draw(
                    animation_cycle,
                    movement_timer + TIME_TRAVEL_MOVEMENT_TIME * self.time_dilation_timer,
                    SNAIL_MOVEMENT_TIME,
                    buffer,
                    buffer_width,
                    bx,
                    by,
                );
            }
        }
    }

    fn movement_time(&self) -> usize {
        TIME_TRAVEL_MOVEMENT_TIME
    }
}
