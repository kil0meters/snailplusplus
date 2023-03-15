use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, ANIMATION_TIME, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE, INVERTED_PALETTE},
    utils::{console_log, lerp, Vec2, Vec2f, Vec2i},
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
    "#* .#. .     . .#. *#",
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
    // Locked,
}

trait Ghost {
    fn draw(&self, status: GhostStatus, animation_cycle: bool, progress: f32, image: &mut Image);
    fn get_snail(&mut self) -> &mut Snail<10>;
    fn scatter_point(&self) -> Vec2;

    fn pos(&mut self, fact: f32) -> Vec2f {
        let snail = self.get_snail();

        Vec2f {
            x: lerp(snail.prev_pos.x as i32 * 10, snail.pos.x as i32 * 10, fact) as f32,
            y: lerp(snail.prev_pos.y as i32 * 10, snail.pos.y as i32 * 10, fact) as f32,
        }
    }

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
        maze: &mut Maze<10>,
        player_pos: Vec2,
        player_direction: Direction,
    ) {
        let (pos, dir) = {
            let snail = self.get_snail();
            (snail.pos, snail.direction)
        };

        let behind_dir = dir.flip();

        let mut set_wall = false;

        if !maze.get_cell(pos.x, pos.y).has_wall(behind_dir) {
            maze.set_cell(pos.x, pos.y, behind_dir.to_wall());
            match behind_dir {
                Direction::Up if pos.y > 0 => maze.set_cell(pos.x, pos.y - 1, dir.to_wall()),
                Direction::Down if pos.y < 10 => maze.set_cell(pos.x, pos.y + 1, dir.to_wall()),
                Direction::Left if pos.x > 0 => maze.set_cell(pos.x - 1, pos.y, dir.to_wall()),
                Direction::Right if pos.x < 10 => maze.set_cell(pos.x + 1, pos.y, dir.to_wall()),
                _ => {}
            }

            set_wall = true;
        }

        let valid_directions = maze.get_cell(pos.x, pos.y).valid_directions();

        if valid_directions.len() > 1 {
            match status {
                GhostStatus::Chase => self.chase(lfsr, maze, player_pos, player_direction),
                GhostStatus::Scatter => self.scatter(lfsr, maze),
                GhostStatus::Frightened => self.frightened(lfsr, maze),
            }
        } else {
            let snail = self.get_snail();
            snail.direction = valid_directions[0];

            snail.move_forward(maze);
        }

        if set_wall {
            maze.set_cell(pos.x, pos.y, behind_dir.to_wall());
            match behind_dir {
                Direction::Up if pos.y > 0 => maze.set_cell(pos.x, pos.y - 1, dir.to_wall()),
                Direction::Down if pos.y < 10 => maze.set_cell(pos.x, pos.y + 1, dir.to_wall()),
                Direction::Left if pos.x > 0 => maze.set_cell(pos.x - 1, pos.y, dir.to_wall()),
                Direction::Right if pos.x < 10 => maze.set_cell(pos.x + 1, pos.y, dir.to_wall()),
                _ => {}
            }
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
                let directions = maze.get_cell(snail.pos.x, snail.pos.y).valid_directions();
                directions[lfsr.next() as usize % directions.len()]
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
        if let Some(dir) = directions.first() {
            self.0.direction = *dir;
            assert!(self.0.move_forward(maze));
        }
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

// Scatters to bottom left
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

// Scatters to bottom left
// should target an extended vector from blinky's line of sight to the snail, but that would be
// annoying to implenment which how I have things structured right now, so we instead target one
// space behind the snail.
struct Inky(Snail<10>);

pub const INKY_PALETTE: [[u8; 3]; 6] = [
    [0x55, 0xaa, 0xff], // light blue
    [0x55, 0xff, 0xff], // lighter lbue
    [0x55, 0xaa, 0xff], // light blue
    [0xff, 0xff, 0xff], // snail "eyes"
    [0x00, 0x00, 0x00],
    [0x00, 0x0A, 0x00],
];

impl Ghost for Inky {
    fn draw(&self, status: GhostStatus, animation_cycle: bool, progress: f32, image: &mut Image) {
        let palette = match status {
            GhostStatus::Chase | GhostStatus::Scatter => INKY_PALETTE,
            GhostStatus::Frightened => INVERTED_PALETTE,
        };

        self.0.draw(palette, animation_cycle, progress, image, 0, 0);
    }

    fn get_snail(&mut self) -> &mut Snail<10> {
        &mut self.0
    }

    fn scatter_point(&self) -> Vec2 {
        Vec2 { x: 8, y: 8 }
    }

    fn chase(
        &mut self,
        _lfsr: &mut LFSR,
        maze: &Maze<10>,
        player_pos: Vec2,
        player_direction: Direction,
    ) {
        let mut target_position = player_pos;

        match player_direction.flip() {
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
    let mut ghosts: Vec<Box<dyn Ghost>> = vec![
        Box::new(Blinky(Snail::new())),
        Box::new(Pinky(Snail::new())),
        Box::new(Clyde(Snail::new())),
        Box::new(Inky(Snail::new())),
    ];

    for ghost in &mut ghosts {
        ghost.reset();
    }

    ghosts
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

struct Player {
    pos: Vec2f,
    direction: Option<Direction>,
    just_moved: bool,
}

impl Player {
    fn new() -> Player {
        Player {
            pos: Vec2f::new(45.0, 70.0),
            direction: None,
            just_moved: false,
        }
    }

    fn reset(&mut self) {
        self.pos.x = 45.0;
        self.pos.y = 70.0;
        self.direction = None;
        self.just_moved = false;
    }

    fn movement(&mut self, maze: &Maze<10>, keys: &Vec<u32>, dt: f32) {
        let mut pos_tilewise = self.pos * 0.1;
        pos_tilewise.x = pos_tilewise.x.round();
        pos_tilewise.y = pos_tilewise.y.round();

        let pos_i_tilewise = pos_tilewise.to_vec2i();

        pos_tilewise = pos_tilewise * 10.0;
        let diff = self.pos - pos_tilewise;

        let cell = maze.get_cell(pos_i_tilewise.x as usize, pos_i_tilewise.y as usize);

        const MOVEMENT_SPEED: f32 = 0.03;
        const SNAPPINGS_SIZE: f32 = 2.0;

        match keys.first() {
            Some(1) if !cell.has_wall(Direction::Right) && diff.y.abs() < SNAPPINGS_SIZE => {
                self.direction = Some(Direction::Right);
            }

            Some(2) if !cell.has_wall(Direction::Left) && diff.y.abs() < SNAPPINGS_SIZE => {
                self.direction = Some(Direction::Left);
            }

            Some(4) if !cell.has_wall(Direction::Down) && diff.x.abs() < SNAPPINGS_SIZE => {
                self.direction = Some(Direction::Down);
            }

            Some(8) if !cell.has_wall(Direction::Up) && diff.x.abs() < SNAPPINGS_SIZE => {
                self.direction = Some(Direction::Up);
            }
            _ => {}
        }

        self.just_moved = false;

        // if we are properly aligned horizontally and the first key is, we can move up
        if diff.x.abs() < SNAPPINGS_SIZE {
            match self.direction {
                Some(Direction::Down)
                    if !cell.has_wall(Direction::Down) || self.pos.y < pos_tilewise.y =>
                {
                    self.pos.x = pos_tilewise.x;
                    self.pos.y = self.pos.y + MOVEMENT_SPEED * dt;
                    self.just_moved = true;
                }
                Some(Direction::Up)
                    if !cell.has_wall(Direction::Up) || self.pos.y > pos_tilewise.y =>
                {
                    self.pos.x = pos_tilewise.x;
                    self.pos.y = self.pos.y - MOVEMENT_SPEED * dt;
                    self.just_moved = true;
                }
                _ => {}
            }
        }

        if diff.y.abs() < SNAPPINGS_SIZE {
            match self.direction {
                Some(Direction::Right)
                    if !cell.has_wall(Direction::Right) || self.pos.x < pos_tilewise.x =>
                {
                    self.direction = Some(Direction::Right);
                    self.pos.y = pos_tilewise.y;
                    self.pos.x = self.pos.x + MOVEMENT_SPEED * dt;
                    self.just_moved = true;
                }
                Some(Direction::Left)
                    if !cell.has_wall(Direction::Left) || self.pos.x > pos_tilewise.x =>
                {
                    self.direction = Some(Direction::Left);
                    self.pos.y = pos_tilewise.y;
                    self.pos.x = self.pos.x - MOVEMENT_SPEED * dt;
                    self.just_moved = true;
                }
                _ => {}
            }
        }
    }

    fn draw(&self, buffer: &mut [u8], animation_cycle: bool) {
        let mut image = Image {
            buffer,
            buffer_width: 101,
        };

        let pos = self.pos.to_vec2i();

        image.draw_snail(
            DEFAULT_PALETTE,
            !self.just_moved || animation_cycle,
            self.direction.unwrap_or(Direction::Right),
            pos.x as usize,
            pos.y as usize,
        );
    }
}

pub struct PacSnail {
    bg_buffer: Vec<u8>,
    pellets: Vec<Pellet>,
    player: Player,
    ghosts: Vec<Box<dyn Ghost>>,
    locked_ghosts: Vec<Box<dyn Ghost>>,
    ghost_movement_timer: f32,
    powerup_timer: f32,
    powerup_streak: usize,
    maze: Maze<10>,
    pellet_count: usize,
    time: f32,
}

impl PacSnail {
    pub fn new() -> Self {
        let mut s = Self {
            maze: Maze::new(),
            pellets: vec![],
            pellet_count: 0,
            player: Player::new(),
            bg_buffer: vec![0; 4 * 101 * 101],
            ghost_movement_timer: 0.0,
            powerup_timer: 0.0,
            powerup_streak: 0,
            time: 0.0,
            ghosts: vec![],
            locked_ghosts: vec![],
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
        self.time = 0.0;

        self.ghosts = all_ghosts();
        self.locked_ghosts.clear();
        self.ghost_movement_timer = 0.0;
        self.powerup_timer = 0.0;
        self.powerup_streak = 0;

        self.player.reset();

        let mut image = Image {
            buffer: &mut self.bg_buffer,
            buffer_width: 101,
        };

        self.maze
            .draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image, 0, 0);

        self.draw_pellets();
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![101, 101]
    }

    pub fn tick(&mut self, lfsr: &mut LFSR, keys: &Vec<u32>, dt: f32) -> i32 {
        if self.pellet_count == 0 {
            self.reset();
            return -100;
        }

        self.powerup_timer = self.powerup_timer - dt;

        if self.powerup_timer < 0.0 {
            self.powerup_streak = 0;
            self.powerup_timer = 0.0;

            self.ghosts.extend(self.locked_ghosts.drain(..));
        }

        self.time += dt;
        self.ghost_movement_timer += dt;

        self.player.movement(&self.maze, &keys, dt);

        let mut score = 0;

        let player_pos = Vec2 {
            x: (self.player.pos.x * 0.1).round() as usize,
            y: (self.player.pos.y * 0.1).round() as usize,
        };

        match self.pellets[player_pos.y * 10 + player_pos.x] {
            Pellet::Pellet => {
                self.pellets[player_pos.y * 10 + player_pos.x] = Pellet::None;
                self.draw_pellet(player_pos.x, player_pos.y);
                self.pellet_count -= 1;
                score += 5;
            }
            Pellet::Powerup => {
                self.pellets[player_pos.y * 10 + player_pos.x] = Pellet::None;
                self.draw_pellet(player_pos.x, player_pos.y);
                self.powerup_timer = POWERUP_TIME;
                self.pellet_count -= 1;
                score += 10;
            }
            Pellet::None => {}
        };

        if self.ghost_movement_timer > GHOST_MOVEMENT_TIME {
            self.ghost_movement_timer -= GHOST_MOVEMENT_TIME;

            let current_status = if self.powerup_timer > 0.0 {
                GhostStatus::Frightened
            } else {
                SCATTER_SCHEUDLE
                    [((self.time / 2000.0).floor() as usize).min(SCATTER_SCHEUDLE.len() - 1)]
            };

            for ghost in &mut self.ghosts {
                ghost.step(
                    current_status,
                    lfsr,
                    &mut self.maze,
                    player_pos,
                    self.player.direction.unwrap_or(Direction::Right),
                );
            }
        }

        let mut i = 0;
        while i < self.ghosts.len() {
            // test if ghost collides with player
            let ghost_pos = self.ghosts[i].pos(self.ghost_movement_timer / GHOST_MOVEMENT_TIME);

            // test collision
            if (ghost_pos.x - self.player.pos.x).powi(2) + (ghost_pos.y - self.player.pos.y).powi(2)
                < 6.0 * 6.0
            {
                if self.powerup_timer > 0.0 {
                    score += 200 * 1 << self.powerup_streak;
                    self.powerup_streak += 1;

                    let mut ghost = self.ghosts.swap_remove(i);
                    ghost.reset();
                    self.locked_ghosts.push(ghost);

                    continue;
                } else {
                    self.reset();
                    return 0;
                }
            }

            i += 1;
        }

        score
    }

    pub fn render(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bg_buffer);

        let mut image = Image {
            buffer,
            buffer_width: 101,
        };

        // self.maze
        //     .draw_background(DEFAULT_PALETTE[4], DEFAULT_PALETTE[5], &mut image, 0, 0);

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

        // draw ghosts
        for ghost in &self.ghosts {
            ghost.draw(
                ghost_status_draw,
                animation_cycle,
                self.ghost_movement_timer / GHOST_MOVEMENT_TIME,
                &mut image,
            );
        }

        // draw player
        let animation_cycle = (self.time / (ANIMATION_TIME / 4.0)).floor() as usize % 2 == 0;
        self.player.draw(buffer, animation_cycle);
    }
}
