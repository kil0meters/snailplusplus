use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::PI,
};

use crate::{
    image::Image,
    lfsr::LFSR,
    snail::{DEFAULT_PALETTE, INVERTED_PALETTE},
    utils::{Vec2f, Vec2i},
};

#[derive(Debug, Clone, Copy)]
struct Ray {
    pos: Vec2f,
    dir: f32,
}

const FOV: f32 = PI / 2.0;
const DEFAULT_MAZE_SIZE: usize = 5;
const SHOOT_COOLDOWN: f32 = 500.0;
const SCREEN_W: usize = 240;
const SCREEN_H: usize = 240;
const LINE_HEIGHT: usize = 180;

fn generate_bg_buffer(width: usize, height: usize) -> Vec<u8> {
    let mut buffer = vec![0; width * height * 4];

    // upper half is bright like the sky
    for y in 0..(height / 2) {
        for x in 0..width {
            buffer[4 * (y * width + x)] = DEFAULT_PALETTE[4][0];
            buffer[4 * (y * width + x) + 1] = DEFAULT_PALETTE[4][1];
            buffer[4 * (y * width + x) + 2] = DEFAULT_PALETTE[4][2];
            buffer[4 * (y * width + x) + 3] = 0xFF;
        }
    }

    // lower half is a gradient
    for y in (height / 2)..height {
        let color = hsl_to_rgb(
            0.56857,
            0.95,
            (0.48 * ((y/* - height / 2 */) as f32 / height as f32)).min(0.48),
        );

        for x in 0..width {
            buffer[4 * (y * width + x)] = color[0];
            buffer[4 * (y * width + x) + 1] = color[1];
            buffer[4 * (y * width + x) + 2] = color[2];
            buffer[4 * (y * width + x) + 3] = 0xFF;
        }
    }

    buffer
}

fn generate_maze_random_walk(grid: &mut Vec<u8>, size: usize, lfsr: &mut LFSR, x: usize, y: usize) {
    let mut next = Some((x, y));
    let width = 2 * size + 1;

    while let Some((x, y)) = next {
        // set current tile to be visited
        grid[(2 * y + 1) * width + (2 * x + 1)] = 0;
        next = None;

        for direction in lfsr.random_order() {
            //right
            if direction == 0 && x < size - 1 && grid[(2 * y + 1) * width + (2 * x + 3)] == 1 {
                // unset right wall
                grid[(2 * y + 1) * width + (2 * x + 2)] = 0;
                next = Some((x + 1, y));
            }
            //left
            else if direction == 1 && x > 0 && grid[(2 * y + 1) * width + (2 * x - 1)] == 1 {
                // unset left wall
                grid[(2 * y + 1) * width + (2 * x)] = 0;
                next = Some((x - 1, y));
            }
            // down
            else if direction == 2 && y < size - 1 && grid[(2 * y + 3) * width + (2 * x + 1)] == 1
            {
                // unset bottom wall wall
                grid[(2 * y + 2) * width + (2 * x + 1)] = 0;
                next = Some((x, y + 1));
            }
            // up
            else if direction == 3 && y > 0 && grid[(2 * y - 1) * width + (2 * x + 1)] == 1 {
                // unset top wall
                grid[(2 * y) * width + (2 * x + 1)] = 0;
                next = Some((x, y - 1));
            }

            if next.is_some() {
                break;
            }
        }
    }
}

fn find_fartherst_point(grid: &[u8], size: usize, x: usize, y: usize) -> (usize, usize) {
    let width = 2 * size + 1;
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new(); // BTreeSet might be faster but idc
    let mut last_point = (x, y);

    queue.push_back((x, y));

    while let Some((x, y)) = queue.pop_front() {
        last_point = (x, y);
        visited.insert(last_point);

        // push all valid directions to the queue
        if x < size - 1
            && grid[(2 * y + 1) * width + (2 * x + 2)] == 0
            && !visited.contains(&(x + 1, y))
        {
            queue.push_back((x + 1, y));
        }
        //left
        if x > 0 && grid[(2 * y + 1) * width + (2 * x)] == 0 && !visited.contains(&(x - 1, y)) {
            queue.push_back((x - 1, y));
        }
        // down
        if y < size - 1
            && grid[(2 * y + 2) * width + (2 * x + 1)] == 0
            && !visited.contains(&(x, y + 1))
        {
            queue.push_back((x, y + 1));
        }
        // up
        if y > 0 && grid[(2 * y) * width + (2 * x + 1)] == 0 && !visited.contains(&(x, y - 1)) {
            queue.push_back((x, y - 1));
        }
    }

    last_point
}

// assumes buffer is of size (2 * size + 1)^2
fn generate_maze(grid: &mut Vec<u8>, lfsr: &mut LFSR, size: usize) {
    // fill in entire grid with walls
    grid.fill(1);

    generate_maze_random_walk(grid, size, lfsr, 0, 0);

    let width = 2 * size + 1;

    for x in 0..size {
        for y in 0..size {
            // if not visited
            if grid[(2 * y + 1) * width + (2 * x + 1)] == 1 {
                for direction in lfsr.random_order() {
                    //right
                    if direction == 0
                        && x < size - 1
                        && grid[(2 * y + 1) * width + (2 * x + 3)] == 0
                    {
                        // unset right wall
                        grid[(2 * y + 1) * width + (2 * x + 2)] = 0;
                        generate_maze_random_walk(grid, size, lfsr, x, y);
                        break;
                    }
                    //left
                    else if direction == 1
                        && x > 0
                        && grid[(2 * y + 1) * width + (2 * x - 1)] == 0
                    {
                        // unset left wall
                        grid[(2 * y + 1) * width + (2 * x)] = 0;
                        generate_maze_random_walk(grid, size, lfsr, x, y);
                        break;
                    }
                    // down
                    else if direction == 2
                        && y < size - 1
                        && grid[(2 * y + 3) * width + (2 * x + 1)] == 0
                    {
                        // unset bottom wall wall
                        grid[(2 * y + 2) * width + (2 * x + 1)] = 0;
                        generate_maze_random_walk(grid, size, lfsr, x, y);
                        break;
                    }
                    // up
                    else if direction == 3
                        && y > 0
                        && grid[(2 * y - 1) * width + (2 * x + 1)] == 0
                    {
                        // unset top wall
                        grid[(2 * y) * width + (2 * x + 1)] = 0;
                        generate_maze_random_walk(grid, size, lfsr, x, y);
                        break;
                    }
                }
            }
        }
    }
}

struct Enemy {
    pos: Vec2f,
    health: f32,
    last_damaged: f32,
}

impl Enemy {
    fn new(x: f32, y: f32) -> Enemy {
        Enemy {
            pos: Vec2f::new(x, y),
            health: 100.0,
            last_damaged: -1000.0,
        }
    }

    fn damage(&mut self, amount: f32, time: f32) -> bool {
        self.health -= amount;
        self.last_damaged = time;

        self.health < 0.0
    }
}

pub struct WolfensteinGame {
    player_pos: Ray,
    goal_pos: Vec2f,
    size: usize,
    width: usize,
    grid: Vec<u8>,

    enemies: Vec<Enemy>,
    zbuffer: Vec<f32>,
    bg_buffer: Vec<u8>,

    shoot_cooldown: f32,
    time: f32,
}

impl WolfensteinGame {
    pub fn new(lfsr: &mut LFSR) -> WolfensteinGame {
        let mut game = WolfensteinGame {
            player_pos: Ray {
                pos: Vec2f::new(1.5, 1.5),
                dir: 0.0,
            },
            goal_pos: Vec2f::new(
                (2 * DEFAULT_MAZE_SIZE) as f32 - 0.5,
                (2 * DEFAULT_MAZE_SIZE) as f32 - 0.5,
            ),
            grid: vec![0; (2 * DEFAULT_MAZE_SIZE + 1).pow(2)],
            width: 2 * DEFAULT_MAZE_SIZE + 1,
            size: DEFAULT_MAZE_SIZE,

            bg_buffer: generate_bg_buffer(SCREEN_W, SCREEN_H),
            zbuffer: vec![0.0; SCREEN_W],
            enemies: vec![],

            shoot_cooldown: 0.0,
            time: 0.0,
        };

        game.reset(lfsr);

        game
    }

    fn cast_ray(&self, ray: Ray) -> f32 {
        let ray_dir = Vec2f::new(ray.dir.cos(), ray.dir.sin());

        let ray_step = Vec2f::new((1.0 / ray_dir.x).abs(), (1.0 / ray_dir.y).abs());

        let mut map_pos = Vec2i::new(ray.pos.x as i32, ray.pos.y as i32);
        let mut ray_length = Vec2f::new(0.0, 0.0);
        let mut step = Vec2i::new(0, 0);

        if ray_dir.x < 0.0 {
            step.x = -1;
            ray_length.x = (ray.pos.x - map_pos.x as f32) * ray_step.x;
        } else {
            step.x = 1;
            ray_length.x = ((map_pos.x + 1) as f32 - ray.pos.x) * ray_step.x;
        }

        if ray_dir.y < 0.0 {
            step.y = -1;
            ray_length.y = (ray.pos.y - map_pos.y as f32) * ray_step.y;
        } else {
            step.y = 1;
            ray_length.y = ((map_pos.y + 1) as f32 - ray.pos.y) * ray_step.y;
        }

        let mut tile_found = false;
        let mut side = 0;

        while !tile_found {
            if ray_length.x < ray_length.y {
                map_pos.x += step.x;
                ray_length.x += ray_step.x;
                side = 0;
            } else {
                map_pos.y += step.y;
                ray_length.y += ray_step.y;
                side = 1;
            }

            if self.grid[(map_pos.y * self.width as i32 + map_pos.x) as usize] != 0 {
                tile_found = true;
            }
        }

        if side == 0 {
            ray_length.x - ray_step.x
        } else {
            ray_length.y - ray_step.y
        }
    }

    fn reset(&mut self, lfsr: &mut LFSR) {
        generate_maze(&mut self.grid, lfsr, self.size);

        self.player_pos.pos.x = 1.5;
        self.player_pos.pos.y = 1.5;

        // // print grid
        // let mut grid_string = String::new();
        //
        // for i in 0..self.grid.len() {
        //     if self.grid[i] == 0 {
        //         grid_string.push('.');
        //     } else {
        //         grid_string.push('#');
        //     }
        //
        //     if (i + 1) % (2 * DEFAULT_MAZE_SIZE + 1) == 0 {
        //         grid_string.push('\n');
        //     }
        // }
        // console_log!("{grid_string}");

        // find the farthest point and put the goal there
        let (goal_x, goal_y) = find_fartherst_point(&self.grid, self.size, 0, 0);

        self.goal_pos.x = (2 * goal_x) as f32 + 1.5;
        self.goal_pos.y = (2 * goal_y) as f32 + 1.5;

        // console_log!("{:?}", self.goal_pos);

        // we add the little constants here because it looks ugly before you turn otherwise.
        if self.grid[self.width + 2] != 0 {
            self.player_pos.dir = PI / 2.0 + 0.001;
        } else {
            self.player_pos.dir = 0.001;
        }

        self.enemies.clear();

        let mut invalid_positions = HashSet::new();
        invalid_positions.insert((goal_x, goal_y));
        invalid_positions.insert((0, 0));

        // generate some random enemies in random locations
        for _ in 0..self.size {
            let mut x = 0;
            let mut y = 0;

            while invalid_positions.contains(&(x, y)) {
                x = lfsr.big() % self.size;
                y = lfsr.big() % self.size;
            }

            invalid_positions.insert((x, y));

            self.enemies
                .push(Enemy::new((x * 2) as f32 + 1.5, (y * 2) as f32 + 1.5));
        }
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![240, 240]
    }

    pub fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: f32) -> i32 {
        self.time += dt;
        self.shoot_cooldown = (self.shoot_cooldown - dt).max(0.0);

        let keys_bits = keys.iter().fold(0, |acc, x| acc + x);

        // right
        if keys_bits & 1 != 0 {
            self.player_pos.dir += 0.003 * dt;
        }

        // left
        if keys_bits & 2 != 0 {
            self.player_pos.dir -= 0.003 * dt;
        }

        // up
        if keys_bits & 8 != 0 {
            let movement_mag = (0.001 * dt).min(self.cast_ray(self.player_pos) - 0.1);
            //

            self.player_pos.pos += Vec2f::new(movement_mag, 0.0).rot(self.player_pos.dir);
        }

        self.player_pos.dir %= PI * 2.0;

        // test collision
        if self.player_pos.pos.dist2(self.goal_pos) < 0.5 * 0.5 {
            self.reset(lfsr);

            return -10_00_000;
        }

        // down
        if (keys_bits & 4) != 0 {
            if self.shoot_cooldown <= 0.0 {
                self.shoot_cooldown = SHOOT_COOLDOWN;

                if self.fire_bullet() {
                    return 1_000_000;
                }
            }
        } else {
            self.shoot_cooldown = 0.0;
        }
        if keys_bits & 4 != 0 {}

        0
    }

    // returns true if an enemy was killed
    pub fn fire_bullet(&mut self) -> bool {
        let inc_dist = 0.1;
        let inc = Vec2f::new(self.player_pos.dir.cos(), self.player_pos.dir.sin()) * inc_dist;
        let mut cur_pos = self.player_pos.pos;
        let max_dist = 10.0;
        let mut dist = 0.0;

        while dist < max_dist {
            dist += inc_dist;
            cur_pos += inc;

            let map_pos = Vec2i::new(cur_pos.x.floor() as i32, cur_pos.y.floor() as i32);

            // we stop when we run into in a wall
            if map_pos.x > 0
                && map_pos.x < self.width as i32
                && map_pos.y > 0
                && map_pos.y < self.width as i32
            {
                if self.grid[(map_pos.y * self.width as i32 + map_pos.x) as usize] != 0 {
                    break;
                }
            }

            // test if we collide with any enemies
            for (i, enemy) in self.enemies.iter_mut().enumerate() {
                if enemy.pos.dist2(cur_pos) < 0.25 * 0.25 {
                    if enemy.damage(35.0, self.time) {
                        self.enemies.swap_remove(i);
                        return true;
                    }

                    return false;
                }
            }
        }

        false
    }

    pub fn render(&mut self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bg_buffer);

        let mut image = Image {
            buffer,
            buffer_width: 240,
        };

        let dist_to_plane = (SCREEN_W as f32 / 2.0) / (FOV / 2.0).tan();
        let mut ray = self.player_pos;
        ray.dir -= FOV / 2.0;

        for x in 0..240 {
            let angle = (x as f32 - SCREEN_W as f32 / 2.0).atan2(dist_to_plane);
            ray.dir = angle + self.player_pos.dir;

            let dist = self.cast_ray(ray);
            self.zbuffer[x] = dist;

            let height =
                ((dist_to_plane / 2.0) / dist / (self.player_pos.dir - ray.dir).cos()) as usize;

            let color = hsl_to_rgb(
                0.6716667,
                0.92,
                (0.488 * (height as f32 / LINE_HEIGHT as f32).sqrt()).min(0.488),
            );

            for y in 0..height.min(SCREEN_H / 2) {
                image.draw_pixel_xy(color, x, 120 + y);
                image.draw_pixel_xy(color, x, 120 - y);
            }
        }

        self.draw_sprite(
            &mut image,
            DEFAULT_PALETTE,
            self.goal_pos,
            (self.time / 1000.0).cos() * 0.25 + 0.5,
            include_bytes!("../../../assets/goal_7x7.bin"),
            7,
        );

        // sort enemies by distance to player
        let player_pos = self.player_pos.pos;
        self.enemies.sort_unstable_by(|e1, e2| {
            ((e2.pos.x - player_pos.x).powi(2) + (e2.pos.y - player_pos.y).powi(2))
                .total_cmp(&((e1.pos.x - player_pos.x).powi(2) + (e1.pos.y - player_pos.y).powi(2)))
        });

        for enemy in &self.enemies {
            // if damaged recently, draw with the damaged color palette
            if (enemy.last_damaged - self.time).abs() < 100.0 {
                self.draw_sprite(
                    &mut image,
                    INVERTED_PALETTE,
                    enemy.pos,
                    0.0,
                    include_bytes!("../../../assets/snail1_8x8.bin"),
                    8,
                );
            } else {
                self.draw_sprite(
                    &mut image,
                    DEFAULT_PALETTE,
                    enemy.pos,
                    0.0,
                    include_bytes!("../../../assets/snail1_8x8.bin"),
                    8,
                );
            }
        }

        // crosshair
        image.draw_line(
            DEFAULT_PALETTE[4],
            Vec2i::new(SCREEN_W as i32 / 2 - 4, SCREEN_H as i32 / 2),
            Vec2i::new(SCREEN_W as i32 / 2 + 5, SCREEN_H as i32 / 2),
        );
        image.draw_line(
            DEFAULT_PALETTE[4],
            Vec2i::new(SCREEN_W as i32 / 2, SCREEN_H as i32 / 2 - 4),
            Vec2i::new(SCREEN_W as i32 / 2, SCREEN_H as i32 / 2 + 5),
        );
    }

    fn draw_sprite(
        &self,
        image: &mut Image,
        palette: [[u8; 3]; 6],
        pos: Vec2f,
        height: f32,
        texture: &[u8],
        texture_size: usize,
    ) {
        let dist_to_plane = (SCREEN_W as f32 / 2.0) / (FOV / 2.0).tan();
        let adj_pos = (pos - self.player_pos.pos).rot(-self.player_pos.dir);
        let distance = adj_pos.x;

        // something goes really bad if the distance is close and i don't want to investigate
        // further so we just do an early return here
        if distance < 0.2 {
            return;
        }

        let size = (dist_to_plane / 2.0) / distance;

        let start_x = (SCREEN_W as f32 / 2.0) - size / 2.0 + adj_pos.y * (dist_to_plane / distance);
        let start_y = dist_to_plane - size * height;

        for x in start_x as usize..(start_x + size) as usize {
            let texture_x = (((x as f32 - start_x) * texture_size as f32) / size) as usize;

            if x < SCREEN_W && distance < self.zbuffer[x] {
                for y in start_y as usize..(start_y + size) as usize {
                    let texture_y = (((y as f32 - start_y) * texture_size as f32) / size) as usize;

                    let px = texture[texture_x * texture_size + texture_y];
                    if px != 255 && x < SCREEN_W && y < SCREEN_H {
                        image.draw_pixel_xy(palette[px as usize], x, y);
                    }
                }
            }
        }
    }
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0
    }
    if t > 1.0 {
        t -= 1.0
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    return p;
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    if s == 0.0 {
        [(l * 255.0) as u8; 3]
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };

        let p = 2.0 * l - q;
        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
    }
}
