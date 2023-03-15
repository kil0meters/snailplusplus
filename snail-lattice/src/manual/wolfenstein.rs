use std::f32::consts::PI;

use crate::{
    image::Image,
    snail::DEFAULT_PALETTE,
    utils::{Vec2f, Vec2i},
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

pub struct WolfensteinGame {
    player_pos: Ray,
    size: usize,
    grid: Vec<u8>,

    zbuffer: Vec<f32>,
    bg_buffer: Vec<u8>,
}

impl WolfensteinGame {
    pub fn new() -> WolfensteinGame {
        WolfensteinGame {
            player_pos: Ray {
                pos: Vec2f::new(2.0, 2.0),
                dir: 0.0,
            },
            size: 5,
            bg_buffer: generate_bg_buffer(SCREEN_W, SCREEN_H),
            zbuffer: vec![0.0; SCREEN_W],
            grid: vec![
                1, 1, 1, 1, 1, //
                1, 0, 0, 0, 1, //
                1, 0, 1, 0, 1, //
                1, 0, 0, 0, 1, //
                1, 1, 1, 1, 1,
            ],
        }
    }

    fn cast_ray(&self, ray: Ray) -> f32 {
        let ray_dir = Vec2f::new(ray.dir.cos(), ray.dir.sin());

        let ray_step = Vec2f::new(
            (1.0 / ray_dir.x).abs(),
            (1.0 / ray_dir.y).abs(), // (1.0 + (ray_dir.y / ray_dir.x).powi(2)).sqrt(),
                                     // (1.0 + (ray_dir.x / ray_dir.y).powi(2)).sqrt(),
        );

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

            if self.grid[(map_pos.y * self.size as i32 + map_pos.x) as usize] != 0 {
                tile_found = true;
            }
        }

        if side == 0 {
            ray_length.x - ray_step.x
        } else {
            ray_length.y - ray_step.y
        }
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![240, 240]
    }

    pub fn tick(&mut self, keys: Vec<u32>, dt: f32) {
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
            let angle = (x as f32 - 240.0 / 2.0).atan2(dist_to_plane);
            ray.dir = angle + self.player_pos.dir;

            let dist = self.cast_ray(ray);
            self.zbuffer[x] = dist;

            let height = ((LINE_HEIGHT as f32 / 2.0) / dist / (self.player_pos.dir - ray.dir).cos())
                as usize;

            let color = map_color(height as f32 / LINE_HEIGHT as f32);

            for y in 0..height.min(SCREEN_H / 2) {
                image.draw_pixel_xy(color, x, 120 + y);
                image.draw_pixel_xy(color, x, 120 - y);
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

fn map_color(value: f32) -> [u8; 3] {
    hsl_to_rgb(0.6716667, 0.92, (0.488 * value.sqrt()).min(0.488))
}
