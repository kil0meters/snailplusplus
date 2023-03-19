use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec2 {
    pub x: usize,
    pub y: usize,
}

impl Vec2 {
    pub fn manhattan_dist(&self, rhs: Vec2) -> usize {
        self.x.abs_diff(rhs.x) + self.y.abs_diff(rhs.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Vec2i {
        Vec2i { x, y }
    }
}

impl Add for Vec2i {
    type Output = Vec2i;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2i::new(self.x + rhs.x, self.y + rhs.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Vec2f {
        Vec2f { x, y }
    }

    pub fn to_vec2i(self) -> Vec2i {
        Vec2i::new(self.x.round() as i32, self.y.round() as i32)
    }

    pub fn dist2(self, rhs: Self) -> f32 {
        (self.x - rhs.x) * (self.x - rhs.x) + (self.y - rhs.y) * (self.y - rhs.y)
    }

    pub fn wrap(&mut self, wrapping: f32) {
        if self.x > wrapping {
            self.x -= wrapping;
        }

        if self.x < 0.0 {
            self.x += wrapping;
        }

        if self.y > wrapping {
            self.y -= wrapping;
        }

        if self.y < 0.0 {
            self.y += wrapping;
        }
    }

    pub fn rot(mut self, angle: f32) -> Vec2f {
        let s = angle.sin();
        let c = angle.cos();

        let new_x = self.x * c - self.y * s;
        let new_y = self.x * s + self.y * c;

        self.x = new_x;
        self.y = new_y;

        self
    }

    pub fn rot_around(mut self, pivot: Vec2f, angle: f32) -> Vec2f {
        self.x -= pivot.x;
        self.y -= pivot.y;

        self = self.rot(angle);

        self.x += pivot.x;
        self.y += pivot.y;

        self
    }
}

impl Add for Vec2f {
    type Output = Vec2f;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;

        self
    }
}

impl Sub for Vec2f {
    type Output = Vec2f;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;

        self
    }
}

impl Mul<f32> for Vec2f {
    type Output = Vec2f;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;

        self
    }
}

impl AddAssign for Vec2f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2f {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// discrete linear interpolation
// returns a linear intepolation between v1 and v2 baded on fact1/fact2
pub fn lerpi(v1: i32, v2: i32, fact: f32) -> i32 {
    v1 + (fact * (v2 - v1) as f32).floor() as i32
}

// pub fn lerpf(v1: f32, v2: f32, fact: f32) -> f32 {
//     v1 + (fact * (v2 - v1) as f32)
// }

/* pub fn draw_rectangle(
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: [u8; 3],
    buffer: &mut [u8],
    buffer_width: usize,
    bx: usize,
    by: usize,
) {
    let px = 4 * ((y + by) * buffer_width + x + bx);

    for row in 0..h {
        for col in 0..w {
            draw_pixel(buffer, px + 4 * (row * buffer_width + col), color);
        }
    }
} */

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (crate::utils::log(&format_args!($($t)*).to_string()))
}

#[allow(unused_imports)]
pub(crate) use console_log;
