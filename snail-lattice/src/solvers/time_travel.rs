use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, GRAYSCALE_PALETTE},
    solvers::Solver,
    utils::Vec2,
};

use super::{SolveStatus, Tremaux};

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
    fn draw(&self, lfsr: &mut LFSR, image: &mut Image) {
        match self.directions {
            // up down
            [true, true, false, false] => {
                image.draw_rectangle_with(self.pos.x * 10 + 4, self.pos.y * 10, 3, 10, || {
                    random_color(lfsr)
                });
            }
            // left right
            [false, false, true, true] => {
                image.draw_rectangle_with(self.pos.x * 10, self.pos.y * 10 + 4, 10, 3, || {
                    random_color(lfsr)
                });
            }

            // up right
            [true, false, false, true] => {
                image.draw_rectangle_with(self.pos.x * 10 + 4, self.pos.y * 10, 3, 7, || {
                    random_color(lfsr)
                });

                image.draw_rectangle_with(self.pos.x * 10 + 7, self.pos.y * 10 + 4, 4, 3, || {
                    random_color(lfsr)
                });
            }

            // up left
            [true, false, true, false] => {
                image.draw_rectangle_with(self.pos.x * 10 + 4, self.pos.y * 10, 3, 7, || {
                    random_color(lfsr)
                });

                image.draw_rectangle_with(self.pos.x * 10, self.pos.y * 10 + 4, 4, 3, || {
                    random_color(lfsr)
                });
            }

            // down right
            [false, true, false, true] => {
                image.draw_rectangle_with(self.pos.x * 10 + 4, self.pos.y * 10 + 4, 3, 7, || {
                    random_color(lfsr)
                });

                image.draw_rectangle_with(self.pos.x * 10 + 7, self.pos.y * 10 + 4, 4, 3, || {
                    random_color(lfsr)
                });
            }

            // down left
            [false, true, true, false] => {
                image.draw_rectangle_with(self.pos.x * 10 + 4, self.pos.y * 10 + 4, 3, 7, || {
                    random_color(lfsr)
                });

                image.draw_rectangle_with(self.pos.x * 10, self.pos.y * 10 + 4, 4, 3, || {
                    random_color(lfsr)
                });
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

/// Time Travel Snail Upgrades:
/// - Forward Time Travel: Move 50% faster in the present
/// - Improved Time Relay: Move 50% faster in the past
/// - Time Warp:           Backtrack Instnatly
pub struct TimeTravel {
    snail: Snail,
    state: TimeTravelState,
    path: Vec<PathTile>,
    upgrades: u32,

    path_drawer: Snail,
    time_traveler: Tremaux,
}

impl TimeTravel {
    pub fn new() -> Self {
        let mut path_drawer = Snail::new();
        path_drawer.active = false;

        let mut time_traveler = Tremaux::new();
        time_traveler.snail.active = false;

        TimeTravel {
            snail: Snail::new(),
            state: TimeTravelState::TimeTraveling,
            path: vec![],
            upgrades: 0,

            path_drawer,
            time_traveler,
        }
    }
}

impl Solver for TimeTravel {
    fn set_upgrades(&mut self, upgrades: u32) {
        self.upgrades = upgrades;
    }

    fn setup(&mut self, _maze: &Maze, _lfsr: &mut LFSR) {
        self.state = TimeTravelState::TimeTraveling;
        self.time_traveler.set_movement_time(self.movement_time());
        self.snail.reset();
        self.path.clear();
        self.path_drawer.reset();
        self.time_traveler.setup(_maze, _lfsr);
    }

    fn step(&mut self, maze: &mut Maze, lfsr: &mut LFSR) -> SolveStatus {
        match self.state {
            TimeTravelState::TimeTraveling => match self.time_traveler.step(maze, lfsr) {
                SolveStatus::Solved(_) => {
                    self.state = TimeTravelState::DrawingPath;
                    self.path_drawer.pos = maze.end_pos;
                    self.path_drawer.prev_pos = maze.end_pos;
                    self.path_drawer.direction = self.time_traveler.snail.direction.flip();
                }
                _ => {}
            },
            TimeTravelState::DrawingPath => {
                loop {
                    let cell = maze.get_cell(self.path_drawer.pos.x, self.path_drawer.pos.y);
                    let valid_directions = cell.valid_directions();

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
                    else if valid_directions.len() == 2
                        && cell.has_wall(self.path_drawer.direction)
                    {
                        // make the path_drawer continue along the corridor
                        if self.path_drawer.direction.flip() == valid_directions[0] {
                            self.path_drawer.direction = valid_directions[1];
                        } else {
                            self.path_drawer.direction = valid_directions[0]
                        }
                    }

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
                        break;
                    }

                    if (self.upgrades & 0b100) == 0 {
                        break;
                    }
                }
            }
            TimeTravelState::Normal => {
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
                    return SolveStatus::Solved(1);
                }
            }
        }

        SolveStatus::None
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: f32,
        maze: &Maze,
        lfsr: &mut LFSR,
        image: &mut Image,
    ) {
        match self.state {
            TimeTravelState::TimeTraveling => {
                self.snail.draw(GRAYSCALE_PALETTE, true, 0.0, image);

                self.time_traveler
                    .draw(animation_cycle, movement_timer, maze, lfsr, image);
            }
            TimeTravelState::DrawingPath => {
                for tile in &self.path {
                    tile.draw(lfsr, image);
                }

                self.snail.draw(GRAYSCALE_PALETTE, true, 0.0, image);

                self.path_drawer.draw(
                    DEFAULT_PALETTE,
                    animation_cycle,
                    movement_timer / self.movement_time(),
                    image,
                );
            }
            TimeTravelState::Normal => {
                for tile in &self.path {
                    tile.draw(lfsr, image);
                }

                self.snail.draw(
                    DEFAULT_PALETTE,
                    animation_cycle,
                    movement_timer / self.movement_time(),
                    image,
                );
            }
        }
    }

    fn movement_time(&self) -> f32 {
        match self.state {
            TimeTravelState::Normal => {
                // forward time travel
                if (self.upgrades & 0b1) != 0 {
                    SNAIL_MOVEMENT_TIME / 1.5
                } else {
                    SNAIL_MOVEMENT_TIME
                }
            }
            TimeTravelState::TimeTraveling => {
                // improved time relay
                if (self.upgrades & 0b10) != 0 {
                    (SNAIL_MOVEMENT_TIME / 8.0) / 1.5
                } else {
                    SNAIL_MOVEMENT_TIME / 8.0
                }
            }
            TimeTravelState::DrawingPath => SNAIL_MOVEMENT_TIME / 8.0,
        }
    }
}
