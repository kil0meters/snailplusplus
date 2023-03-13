use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, ANIMATION_TIME, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, INVERTED_PALETTE},
    utils::Vec2,
};

const PACMAN_BOARD: &[u8] = concat!(
    "#####################",
    "#. . . . .#. . . . .#",
    "# ### ### # ### ### #",
    "#* . . . . . . . . *#",
    "# ### # ##### # ### #",
    "#. . .#. .#. .#. . .#",
    "##### ### # ### #####",
    "#   #.#. . . .#.#   #",
    "#   # # ##### # #   #",
    "#   #. .#   #. .#   #",
    "#   # # ##### # #   #",
    "#   #.#. . . .#.#   #",
    "##### # ##### # #####",
    "#. . . . .#. . . . .#",
    "# ### ### # ### ### #",
    "#* .#. . . . . .#. *#",
    "### # # ##### # # ###",
    "#. . .#. .#. .#. . .#",
    "# ####### # ####### #",
    "#. . . . . . . . . .#",
    "#####################",
)
.as_bytes();

#[derive(Clone, Copy)]
enum GhostStatus {
    Chase,
    Scatter,
    Frightened,
}

trait Ghost {
    fn draw(&self, status: GhostStatus, animation_cycle: bool, progress: f32, image: &mut Image);
    fn get_snail(&mut self) -> &mut Snail<10>;
    fn scatter_point(&self) -> Vec2;

    fn chase(
        &mut self,
        lfsr: &mut LFSR,
        maze: &Maze<10>,
        player_pos: Vec2,
        player_direction: Direction,
    );

    fn reset(&mut self) {
        let point = self.scatter_point();
        let snail = self.get_snail();
        snail.pos = point;
        snail.prev_pos = snail.pos;
    }

    fn step(
        &mut self,
        status: GhostStatus,
        lfsr: &mut LFSR,
        maze: &Maze<10>,
        player_pos: Vec2,
        player_direction: Direction,
    ) {
        match status {
            GhostStatus::Chase => {
                // we only actually chase if we have more than two options of where to go
                let snail = self.get_snail();
                let valid_directions = maze.get_cell(snail.pos.x, snail.pos.y).valid_directions();

                if valid_directions.len() > 2 {
                    self.chase(lfsr, maze, player_pos, player_direction)
                } else {
                    if snail.direction.flip() == valid_directions[0] {
                        snail.direction = valid_directions[1];
                    } else {
                        snail.direction = valid_directions[0]
                    }

                    snail.move_forward(maze);
                }
            }
            GhostStatus::Scatter => self.scatter(lfsr, maze),
            GhostStatus::Frightened => self.frightened(lfsr, maze),
        }
    }

    fn scatter(&mut self, lfsr: &mut LFSR, maze: &Maze<10>) {
        let scatter_point = self.scatter_point();
        let mut snail = self.get_snail();

        // is it bad that we have to re-compute this for every single iteration? yes.
        // does this realistically matter when we only have 4 ghosts and a 10x10 maze? hopefully not
        let directions = maze.get_solve_sequence(snail.pos.x, snail.pos.y, scatter_point);

        snail.direction = match directions.first() {
            Some(dir) => *dir,
            None => {
                let directions = maze.get_directions(snail.pos);
                directions[lfsr.next() as usize % directions.len()].unwrap()
            }
        };

        snail.move_forward(maze);
    }

    fn frightened(&mut self, lfsr: &mut LFSR, maze: &Maze<10>) {
        let mut snail = self.get_snail();

        let cell = maze.get_cell(snail.pos.x, snail.pos.y);
        let valid_directions = cell.valid_directions();

        if valid_directions.len() > 2 {
            snail.direction = valid_directions[lfsr.next() as usize % valid_directions.len()];
        } else if valid_directions.len() == 1 {
            snail.direction = valid_directions[0];
        } else {
            if snail.direction.flip() == valid_directions[0] {
                snail.direction = valid_directions[1];
            } else {
                snail.direction = valid_directions[0]
            }
        }

        snail.move_forward(maze);
    }
}

// Scatters to top right
// Targets player directly during chase
struct Blinky(Snail<10>);

pub const BLINKY_PALETTE: [[u8; 3]; 6] = [
    [0xff, 0x00, 0x00], // orange
    [0xff, 0x55, 0x00], // purple
    [0xff, 0x00, 0x00], // orange
    [0xff, 0xff, 0xff], // snail "eyes"
    [0x00, 0x00, 0x00],
    [0x00, 0x0A, 0x00],
];

impl Ghost for Blinky {
    fn draw(&self, status: GhostStatus, animation_cycle: bool, progress: f32, image: &mut Image) {
        let palette = match status {
            GhostStatus::Chase | GhostStatus::Scatter => BLINKY_PALETTE,
            GhostStatus::Frightened => INVERTED_PALETTE,
        };

        self.0.draw(palette, animation_cycle, progress, image, 0, 0);
    }

    fn get_snail(&mut self) -> &mut Snail<10> {
        &mut self.0
    }

    fn scatter_point(&self) -> Vec2 {
        Vec2 { x: 8, y: 1 }
    }

    fn chase(
        &mut self,
        _lfsr: &mut LFSR,
        maze: &Maze<10>,
        player_pos: Vec2,
        _player_direction: Direction,
    ) {
        let directions = maze.get_solve_sequence(self.0.pos.x, self.0.pos.y, player_pos);
        self.0.direction = directions[0];
        assert!(self.0.move_forward(maze));
    }
}

// Scatters to top right
// Targets 1 tile in front of pacman during chase mode
struct Pinky(Snail<10>);

pub const PINKY_PALETTE: [[u8; 3]; 6] = [
    [0xff, 0x00, 0xff], // orange
    [0xff, 0x55, 0xff], // purple
    [0xff, 0x00, 0xff], // orange
    [0xff, 0xff, 0xff], // snail "eyes"
    [0x00, 0x00, 0x00],
    [0x00, 0x0A, 0x00],
];

impl Ghost for Pinky {
    fn draw(&self, status: GhostStatus, animation_cycle: bool, progress: f32, image: &mut Image) {
        let palette = match status {
            GhostStatus::Chase | GhostStatus::Scatter => PINKY_PALETTE,
            GhostStatus::Frightened => INVERTED_PALETTE,
        };

        self.0.draw(palette, animation_cycle, progress, image, 0, 0);
    }

    fn get_snail(&mut self) -> &mut Snail<10> {
        &mut self.0
    }

    fn scatter_point(&self) -> Vec2 {
        Vec2 { x: 1, y: 1 }
    }

    fn chase(
        &mut self,
        _lfsr: &mut LFSR,
        maze: &Maze<10>,
        player_pos: Vec2,
        player_direction: Direction,
    ) {
        let mut target_position = player_pos;

        match player_direction {
            Direction::Up => target_position.y = target_position.y.saturating_sub(1),
            Direction::Down => target_position.y = (target_position.y + 1).min(9),
            Direction::Left => target_position.x = target_position.x.saturating_sub(1),
            Direction::Right => target_position.x = (target_position.x + 1).min(9),
        }

        let directions = maze.get_solve_sequence(self.0.pos.x, self.0.pos.y, target_position);

        self.0.direction = match directions.first() {
            Some(dir) => *dir,
            None => maze.get_solve_sequence(self.0.pos.x, self.0.pos.y, player_pos)[0],
        };

        assert!(self.0.move_forward(maze));
    }
}

// Scatters to top right
// Targets pacman during chase mode, if closer than 1 tile, instead targets his scatter point
struct Clyde(Snail<10>);

pub const CLYDE_PALETTE: [[u8; 3]; 6] = [
    [0xaa, 0xaa, 0x00], // orange
    [0xff, 0xaa, 0x00], // purple
    [0xaa, 0xaa, 0x00], // orange
    [0xff, 0xff, 0xff], // snail "eyes"
    [0x00, 0x00, 0x00],
    [0x00, 0x0A, 0x00],
];

impl Ghost for Clyde {
    fn draw(&self, status: GhostStatus, animation_cycle: bool, progress: f32, image: &mut Image) {
        let palette = match status {
            GhostStatus::Chase | GhostStatus::Scatter => CLYDE_PALETTE,
            GhostStatus::Frightened => INVERTED_PALETTE,
        };

        self.0.draw(palette, animation_cycle, progress, image, 0, 0);
    }

    fn get_snail(&mut self) -> &mut Snail<10> {
        &mut self.0
    }

    fn scatter_point(&self) -> Vec2 {
        Vec2 { x: 1, y: 8 }
    }

    fn chase(
        &mut self,
        lfsr: &mut LFSR,
        maze: &Maze<10>,
        player_pos: Vec2,
        _player_direction: Direction,
    ) {
        let target_position = if player_pos.manhattan_dist(self.0.pos) < 5 {
            self.scatter_point()
        } else {
            player_pos
        };

        let directions = maze.get_solve_sequence(self.0.pos.x, self.0.pos.y, target_position);

        self.0.direction = match directions.first() {
            Some(dir) => *dir,
            None => {
                let directions = maze.get_cell(self.0.pos.x, self.0.pos.y).valid_directions();
                directions[lfsr.next() as usize % directions.len()]
            }
        };

        self.0.move_forward(maze);
    }
}

fn pacman_maze() -> (Maze<10>, Vec<Pellet>, usize) {
    let mut maze = Maze::new();
    let mut pellets = vec![Pellet::None; 10 * 10];
    let mut pellet_count = 0;
    let width = 21;

    for x in 0..10 {
        for y in 0..10 {
            let pos = (y * 2 + 1) * width + (x * 2 + 1);
            let mut cell = 0;

            if PACMAN_BOARD[pos] == b'.' {
                pellets[y * 10 + x] = Pellet::Pellet;
                pellet_count += 1;
            } else if PACMAN_BOARD[pos] == b'*' {
                pellets[y * 10 + x] = Pellet::Powerup;
                pellet_count += 1;
            }

            // right
            if PACMAN_BOARD[pos + 1] == b'#' {
                cell |= 1;
            }

            // left
            if PACMAN_BOARD[pos - 1] == b'#' {
                cell |= 2;
            }

            // up
            if PACMAN_BOARD[pos + width] == b'#' {
                cell |= 4;
            }

            // down
            if PACMAN_BOARD[pos - width] == b'#' {
                cell |= 8;
            }

            maze.set_cell(x, y, cell);
        }
    }

    (maze, pellets, pellet_count)
}

fn all_ghosts() -> Vec<Box<dyn Ghost>> {
    vec![
        Box::new(Blinky(Snail::new())),
        Box::new(Pinky(Snail::new())),
        Box::new(Clyde(Snail::new())),
    ]
}

const PACMAN_MOVEMENT_TIME: f32 = SNAIL_MOVEMENT_TIME * 1.5;
const GHOST_MOVEMENT_TIME: f32 = SNAIL_MOVEMENT_TIME * 1.6;
const POWERUP_TIME: f32 = SNAIL_MOVEMENT_TIME * 30.0;

// Each entry represents 2 seconds
const SCATTER_SCHEUDLE: &'static [GhostStatus] = &[
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Chase,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Scatter,
    GhostStatus::Chase,
];

#[derive(Clone, Copy, PartialEq)]
enum Pellet {
    Pellet,
    Powerup,
    None,
}

pub struct PacSnail {
    bg_buffer: Vec<u8>,
    pellets: Vec<Pellet>,
    snail: Snail<10>,
    ghosts: Vec<Box<dyn Ghost>>,
    maze: Maze<10>,
    stuck: bool,
    pellet_count: usize,
    next_direction: Option<Direction>,
    time: f32,
    powerup_timer: f32,
    player_movement_timer: f32,
    ghost_movement_timer: f32,
}

impl PacSnail {
    pub fn new() -> Self {
        let mut s = Self {
            maze: Maze::new(),
            pellets: vec![],
            pellet_count: 0,
            bg_buffer: vec![0; 4 * 101 * 101],
            snail: Snail::new(),
            stuck: true,
            next_direction: None,
            powerup_timer: 0.0,
            time: 0.0,
            ghosts: all_ghosts(),
            player_movement_timer: PACMAN_MOVEMENT_TIME,
            ghost_movement_timer: GHOST_MOVEMENT_TIME,
        };
        s.reset();
        s
    }

    fn draw_pellet(&mut self, x: usize, y: usize) {
        match self.pellets[y * 10 + x] {
            Pellet::Pellet => {
                let px = 4 * ((y * 10 + 5) * 101 + x * 10 + 5);
                self.bg_buffer[px] = 0xFF;
                self.bg_buffer[px + 1] = 0xFF;
                self.bg_buffer[px + 2] = 0x00;
            }
            Pellet::Powerup => {
                let mut px = 4 * ((y * 10 + 4) * 101 + x * 10 + 4);

                for _ in 0..3 {
                    for x in 0..3 {
                        self.bg_buffer[px + 4 * x] = 0xFF;
                        self.bg_buffer[px + 4 * x + 1] = 0xFF;
                        self.bg_buffer[px + 4 * x + 2] = 0x00;
                    }
                    px += 4 * 101;
                }
            }
            Pellet::None => {
                let mut px = 4 * ((y * 10 + 4) * 101 + x * 10 + 4);

                for _ in 0..3 {
                    for x in 0..3 {
                        self.bg_buffer[px + 4 * x] = DEFAULT_PALETTE[5][0];
                        self.bg_buffer[px + 4 * x + 1] = DEFAULT_PALETTE[5][1];
                        self.bg_buffer[px + 4 * x + 2] = DEFAULT_PALETTE[5][2];
                    }
                    px += 4 * 101;
                }
            }
        }
    }

    fn draw_pellets(&mut self) {
        for x in 0..10 {
            for y in 0..10 {
                self.draw_pellet(x, y);
            }
        }
    }

    fn reset(&mut self) {
        let (maze, pellets, pellet_count) = pacman_maze();

        self.maze = maze;
        self.pellets = pellets;
        self.pellet_count = pellet_count;

        self.ghosts = all_ghosts();

        let mut image = Image {
            buffer: &mut self.bg_buffer,
            buffer_width: 101,
        };

        self.maze
            .draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image, 0, 0);

        self.draw_pellets();

        self.snail.direction = Direction::Right;
        self.snail.pos = Vec2 { x: 4, y: 7 };
        self.snail.prev_pos = self.snail.pos;

        self.stuck = true;

        self.powerup_timer = 0.0;

        for ghost in &mut self.ghosts {
            ghost.reset();
        }
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![101, 101]
    }

    pub fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: f32) -> i32 {
        if self.pellet_count == 0 {
            self.reset();
            return -1;
        }

        self.powerup_timer = (self.powerup_timer - dt).max(0.0);
        self.ghost_movement_timer += dt;
        let can_move = self.ghost_movement_timer / GHOST_MOVEMENT_TIME >= 1.0;

        let mut score = 0;

        if can_move {
            self.ghost_movement_timer %= GHOST_MOVEMENT_TIME;
            let mut i = 0;
            while i < self.ghosts.len() {
                let ghost_snail = self.ghosts[i].get_snail();
                if ghost_snail.pos == self.snail.pos
                    || ghost_snail.prev_pos == self.snail.pos
                    || ghost_snail.pos == self.snail.prev_pos
                    || ghost_snail.prev_pos == self.snail.prev_pos
                {
                    if self.powerup_timer > 0.0 {
                        self.ghosts.swap_remove(i);

                        score += 200 * (1 << (3 - self.ghosts.len()));

                        continue;
                    } else {
                        self.reset();
                        return 0;
                    }
                }

                let ghost_status = if self.powerup_timer > 0.0 {
                    GhostStatus::Frightened
                } else {
                    SCATTER_SCHEUDLE
                        [((self.time / 1000.0).floor() as usize).min(SCATTER_SCHEUDLE.len() - 1)]
                };

                self.ghosts[i].step(
                    ghost_status,
                    lfsr,
                    &self.maze,
                    self.snail.pos,
                    self.snail.direction,
                );

                i += 1;
            }
        }

        self.time += dt;

        self.player_movement_timer += dt;
        let can_move = self.player_movement_timer / PACMAN_MOVEMENT_TIME >= 1.0;

        match keys.first() {
            Some(1) => {
                self.next_direction = Some(Direction::Right);
                self.stuck = false;
            }
            Some(2) => {
                self.next_direction = Some(Direction::Left);
                self.stuck = false;
            }
            Some(4) => {
                self.next_direction = Some(Direction::Down);
                self.stuck = false;
            }
            Some(8) => {
                self.next_direction = Some(Direction::Up);
                self.stuck = false;
            }
            _ => {}
        }

        if !self.stuck && can_move {
            if let Some(next_direction) = self.next_direction {
                // only move if that direction is currently available
                if self
                    .maze
                    .get_cell(self.snail.pos.x, self.snail.pos.y)
                    .valid_directions()
                    .contains(&next_direction)
                {
                    self.snail.direction = next_direction;
                }
                self.next_direction = None;
            }

            let pellet = self
                .pellets
                .get_mut(self.snail.pos.y * 10 + self.snail.pos.x)
                .unwrap();

            match *pellet {
                Pellet::Pellet => score = 5,
                Pellet::Powerup => {
                    score = 10;
                    self.powerup_timer = POWERUP_TIME;
                }
                Pellet::None => score = 0,
            }

            if *pellet != Pellet::None {
                self.pellet_count -= 1;
                *pellet = Pellet::None;
                self.draw_pellet(self.snail.pos.x, self.snail.pos.y);
            }

            self.next_direction = None;
            if !self.snail.move_forward(&self.maze) {
                self.stuck = true;
            } else {
                self.player_movement_timer = 0.0;
            }
        }

        score
    }

    pub fn render(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bg_buffer);

        let animation_cycle =
            self.stuck || (self.time / (ANIMATION_TIME / 4.0)).floor() as usize % 2 == 0;

        let mut image = Image {
            buffer,
            buffer_width: 101,
        };

        self.snail.draw(
            DEFAULT_PALETTE,
            animation_cycle,
            self.player_movement_timer / PACMAN_MOVEMENT_TIME,
            &mut image,
            0,
            0,
        );

        let animation_cycle = (self.time / ANIMATION_TIME).floor() as usize % 2 == 0;
        let ghost_status_draw = if self.powerup_timer > 3000.0 {
            GhostStatus::Frightened
        } else if self.powerup_timer > 0.0 {
            if (self.powerup_timer / 200.0).floor() as usize % 2 == 0 {
                GhostStatus::Chase
            } else {
                GhostStatus::Frightened
            }
        } else {
            GhostStatus::Chase
        };

        for ghost in &self.ghosts {
            ghost.draw(
                ghost_status_draw,
                animation_cycle,
                self.ghost_movement_timer / GHOST_MOVEMENT_TIME,
                &mut image,
            );
        }
    }
}
