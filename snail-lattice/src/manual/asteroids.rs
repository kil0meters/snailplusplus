use std::f32::consts::PI;

use crate::{
    image::Image,
    lfsr::LFSR,
    snail::DEFAULT_PALETTE,
    utils::{console_log, Vec2, Vec2f, Vec2i},
};

struct Asteroid {
    pos: Vec2f,
    vel: Vec2f,
    rot: f32,
    size: f32,
}

impl Asteroid {
    fn new(pos: Vec2f, vel: Vec2f, size: f32) -> Asteroid {
        Asteroid {
            pos,
            vel,
            rot: 0.0,
            size,
        }
    }

    fn movement(&mut self, dt: f32) {
        self.rot += 0.001 * dt;
        self.pos += self.vel * dt;

        self.pos.wrap(240.0);
    }

    // returns None if size < 4.0
    fn split(self, split_dir: f32) -> Option<(Asteroid, Asteroid)> {
        if self.size < 8.0 {
            return None;
        }

        let asteroid1 = Asteroid::new(
            self.pos,
            self.vel.rot(split_dir + PI / 2.0) * 1.5,
            self.size / 2.0,
        );

        let asteroid2 = Asteroid::new(
            self.pos,
            self.vel.rot(split_dir - PI / 2.0) * 1.5,
            self.size / 2.0,
        );

        Some((asteroid1, asteroid2))
    }

    fn collides(&self, pos: Vec2f, size: f32) -> bool {
        (self.pos.x - pos.x).powi(2) + (self.pos.y - pos.y).powi(2) <= (size + self.size).powi(2)
    }

    fn draw(&self, buffer: &mut [u8]) {
        let point1 = (self.pos + Vec2f::new(0.0, self.size)).rot_around(self.pos, self.rot);
        let point2 = point1.rot_around(self.pos, (72.0 * PI) / 180.0).to_vec2i();
        let point3 = point1
            .rot_around(self.pos, 2.0 * (72.0 * PI) / 180.0)
            .to_vec2i();
        let point4 = point1
            .rot_around(self.pos, 3.0 * (72.0 * PI) / 180.0)
            .to_vec2i();
        let point5 = point1
            .rot_around(self.pos, 4.0 * (72.0 * PI) / 180.0)
            .to_vec2i();
        let point1 = point1.to_vec2i();

        let mut image = Image {
            buffer,
            buffer_width: 240,
        };

        image.draw_line(DEFAULT_PALETTE[4], point1, point2);
        image.draw_line(DEFAULT_PALETTE[4], point2, point3);
        image.draw_line(DEFAULT_PALETTE[4], point3, point4);
        image.draw_line(DEFAULT_PALETTE[4], point4, point5);
        image.draw_line(DEFAULT_PALETTE[4], point5, point1);
    }
}

struct Bullet {
    pos: Vec2f,
    rot: f32,
    age: f32,
}

const MAX_BULLET_AGE: f32 = 2000.0;

impl Bullet {
    fn new(pos: Vec2f, rot: f32) -> Bullet {
        Bullet { pos, rot, age: 0.0 }
    }

    fn movement(&mut self, dt: f32) {
        self.age += dt;
        self.pos += Vec2f::new(0.0, 0.2).rot(self.rot) * dt;

        self.pos.wrap(240.0);
    }

    fn draw(&self, buffer: &mut [u8]) {
        let mut image = Image {
            buffer,
            buffer_width: 240,
        };

        let point1 = self.pos.to_vec2i();
        let point2 = (self.pos + Vec2f::new(0.0, 5.0))
            .rot_around(self.pos, self.rot)
            .to_vec2i();

        image.draw_line(DEFAULT_PALETTE[4], point1, point2);
    }
}

struct Player {
    pos: Vec2f,
    vel: Vec2f,
    rot: f32,
}

impl Player {
    fn new() -> Player {
        Player {
            pos: Vec2f { x: 120., y: 120. },
            vel: Vec2f { x: 0.0, y: 0.0 },
            rot: 0.0,
        }
    }

    fn movement(&mut self, keys: u32, dt: f32) {
        // right
        if (keys & 1) != 0 {
            self.rot = (self.rot + 0.002 * dt) % (2. * PI);
        }

        // left
        if (keys & 2) != 0 {
            self.rot = (self.rot - 0.002 * dt) % (2. * PI);
        }

        // up
        if (keys & 8) != 0 {
            self.vel += Vec2f::new(0.0, 0.0001 * dt).rot(self.rot);
        }

        let movement = self.vel * dt;
        self.pos += movement;
        self.vel -= movement * 0.0005;

        self.pos.wrap(240.0);
    }

    fn draw(&self, buffer: &mut [u8]) {
        let mut image = Image {
            buffer,
            buffer_width: 240,
        };

        // triangle ship
        let point1 = (self.pos + Vec2f::new(0.0, 10.0))
            .rot_around(self.pos, self.rot)
            .to_vec2i();
        let point2 = (self.pos + Vec2f::new(5.0, -10.0))
            .rot_around(self.pos, self.rot)
            .to_vec2i();
        let point3 = (self.pos + Vec2f::new(-5.0, -10.0))
            .rot_around(self.pos, self.rot)
            .to_vec2i();

        image.draw_line(DEFAULT_PALETTE[4], point1, point2);
        image.draw_line(DEFAULT_PALETTE[4], point2, point3);
        image.draw_line(DEFAULT_PALETTE[4], point3, point1);
    }
}

pub struct AsteroidsGame {
    player: Player,
    asteroids: Vec<Asteroid>,
    bullets: Vec<Bullet>,

    shoot_cooldown: f32,
}

const SHOOT_COOLDOWN: f32 = 500.0;

impl AsteroidsGame {
    pub fn new() -> Self {
        AsteroidsGame {
            player: Player::new(),
            asteroids: vec![],
            bullets: vec![],

            shoot_cooldown: 0.0,
        }
    }

    fn generate_asteroids(&mut self, lfsr: &mut LFSR, count: usize) {
        for _ in 0..count {
            // random f32
            let size = (lfsr.big() % 24).max(6) as f32;

            // velocity
            let velx = ((lfsr.big() % 20).max(1) as f32 - 10.0) / 100.0;
            let vely = ((lfsr.big() % 20).max(1) as f32 - 10.0) / 100.0;

            let mut x = 120i32;
            let mut y = 120i32;

            while x.abs_diff(120) < 20 && y.abs_diff(120) < 20 {
                x = (lfsr.big() % 240) as i32;
                y = (lfsr.big() % 240) as i32;
            }

            self.asteroids.push(Asteroid::new(
                Vec2f::new(x as f32, y as f32),
                Vec2f::new(velx, vely),
                size,
            ));
        }
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![240, 240]
    }

    pub fn tick(&mut self, lfsr: &mut LFSR, keys: Vec<u32>, dt: f32) {
        if self.asteroids.is_empty() {
            self.generate_asteroids(lfsr, 5);
        }

        let mut keys_bits = 0;
        for key in keys {
            keys_bits |= key;
        }

        self.player.movement(keys_bits, dt);

        self.shoot_cooldown = (self.shoot_cooldown - dt).max(0.0);

        // down/shoot
        if (keys_bits & 4) != 0 {
            if self.shoot_cooldown <= 0.0 {
                self.shoot_cooldown = SHOOT_COOLDOWN;

                self.bullets
                    .push(Bullet::new(self.player.pos, self.player.rot));
            }
        } else {
            self.shoot_cooldown = 0.0;
        }

        for bullet in &mut self.bullets {
            bullet.movement(dt);
        }

        for asteroid in &mut self.asteroids {
            asteroid.movement(dt);
        }

        let mut i = 0;
        'bullets: while i < self.bullets.len() {
            if self.bullets[i].age > MAX_BULLET_AGE {
                self.bullets.swap_remove(i);
                continue 'bullets;
            }

            let mut j = 0;
            while j < self.asteroids.len() {
                if self.asteroids[j].collides(self.bullets[i].pos, 4.0) {
                    let asteroid = self.asteroids.swap_remove(j);

                    if let Some((asteroid1, asteroid2)) =
                        asteroid.split(self.bullets[i].rot + PI / 2.0)
                    {
                        self.asteroids.push(asteroid1);
                        self.asteroids.push(asteroid2);
                    }

                    self.bullets.swap_remove(i);
                    continue 'bullets;
                }

                j += 1;
            }

            i += 1;
        }

        for asteroid in &self.asteroids {
            if asteroid.collides(self.player.pos, 5.0) {
                // reset
                self.player.pos.x = 120.0;
                self.player.pos.y = 120.0;
                self.player.rot = 0.0;

                self.asteroids.clear();
                self.bullets.clear();

                break;
            }
        }
    }

    pub fn render(&self, buffer: &mut [u8]) {
        // clear buffer
        for i in (0..buffer.len()).step_by(4) {
            buffer[i] = DEFAULT_PALETTE[5][0];
            buffer[i + 1] = DEFAULT_PALETTE[5][1];
            buffer[i + 2] = DEFAULT_PALETTE[5][2];
            buffer[i + 3] = 0xff;
        }

        for bullet in &self.bullets {
            bullet.draw(buffer);
        }

        for asteroid in &self.asteroids {
            asteroid.draw(buffer);
        }

        self.player.draw(buffer);
    }
}
