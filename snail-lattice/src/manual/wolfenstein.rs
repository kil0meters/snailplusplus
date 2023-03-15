use std::f32::consts::PI;

use crate::{
    image::Image,
    lfsr::LFSR,
    snail::DEFAULT_PALETTE,
    utils::{console_log, Vec2f, Vec2i},
};

#[derive(Debug, Clone, Copy)]
struct Ray {
    pos: Vec2f,
    dir: f32,
}

const FOV: f32 = PI / 2.0;
const SCREEN_W: usize = 240;
const SCREEN_H: usize = 240;
const LINE_HEIGHT: usize = 180;

fn generate_bg_buffer(width: usize, height: usize) -> Vec<u8> {
    let mut buffer = vec![0; width * height * 4];

    // upper half is bright like the sky
    for y in 0..(height / 2) {
        for x in (width / 2)..width {
            buffer[4 * (y * width + x)] = DEFAULT_PALETTE[4][0];
            buffer[4 * (y * width + x) + 1] = DEFAULT_PALETTE[4][1];
            buffer[4 * (y * width + x) + 2] = DEFAULT_PALETTE[4][2];
            buffer[4 * (y * width + x) + 3] = 0xFF;
        }
    }

    // lower half is our favorite blue
    for y in (height / 2)..height {
        for x in (width / 2)..width {
            buffer[4 * (y * width + x)] = DEFAULT_PALETTE[4][0];
            buffer[4 * (y * width + x) + 1] = DEFAULT_PALETTE[4][1];
            buffer[4 * (y * width + x) + 2] = DEFAULT_PALETTE[4][2];
            buffer[4 * (y * width + x) + 3] = 0xFF;
        }
    }

    buffer
}

fn generate_maze_random_walk(grid: &mut Vec<u8>, size: usize, lfsr: &mut LFSR, x: usize, y: usize) {
    let mut next = Some((x, y));
    let width = 2 * size + 1;

    while let Some((x, y)) = next {
        console_log!("visiting: {x},{y}");
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

pub struct WolfensteinGame {
    player_pos: Ray,
    goal_pos: Vec2f,
    size: usize,
    width: usize,
    grid: Vec<u8>,

    zbuffer: Vec<f32>,
    bg_buffer: Vec<u8>,
}

const DEFAULT_MAZE_SIZE: usize = 9;

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
            size: DEFAULT_MAZE_SIZE,
            width: 2 * DEFAULT_MAZE_SIZE + 1,
            bg_buffer: generate_bg_buffer(SCREEN_W, SCREEN_H),
            zbuffer: vec![0.0; SCREEN_W],
            grid: vec![0; (2 * DEFAULT_MAZE_SIZE + 1).pow(2)],
        };

        // print grid
        // let mut grid_string = String::new();
        //
        // for i in 0..game.grid.len() {
        //     if game.grid[i] == 0 {
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

        // we add the little constants here because it looks ugly before you turn otherwise.
        if self.grid[self.width + 2] != 0 {
            self.player_pos.dir = PI / 2.0 + 0.001;
        } else {
            self.player_pos.dir = 0.001;
        }
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![240, 240]
    }

    pub fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: f32) -> i32 {
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
        if (self.player_pos.pos.x - self.goal_pos.x).powi(2)
            + (self.player_pos.pos.y - self.goal_pos.y).powi(2)
            < 0.5 * 0.5
        {
            self.reset(lfsr);

            -100000
        } else {
            0
        }
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

        // draw goal
        let adj_goal_pos = (self.goal_pos - self.player_pos.pos).rot(-self.player_pos.dir);

        let distance = adj_goal_pos.x;
        let size = (dist_to_plane / 2.0) / distance;

        let start_x =
            (SCREEN_W as f32 / 2.0) - size / 2.0 + adj_goal_pos.y * (dist_to_plane / distance);
        let start_y = dist_to_plane - size / 2.0;

        let goal_image = include_bytes!("../../../assets/goal_7x7.bin");

        // draw goal, 7x7
        for x in start_x as usize..(start_x + size) as usize {
            let texture_x = (((x as f32 - start_x) * 7.0) / size) as usize;

            if x < SCREEN_W && distance < self.zbuffer[x] {
                for y in start_y as usize..(start_y + size) as usize {
                    let texture_y = (((y as f32 - start_y) * 7.0) / size) as usize;

                    if goal_image[texture_x * 7 + texture_y] != 255 && x < SCREEN_W && y < SCREEN_H
                    {
                        image.draw_pixel_xy(DEFAULT_PALETTE[0], x, y);
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
