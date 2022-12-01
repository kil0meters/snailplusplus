use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec2 {
    pub x: usize,
    pub y: usize,
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
pub fn discrete_lerp(v1: i32, v2: i32, fact1: i32, fact2: i32) -> i32 {
    let difference = v2 - v1;
    let add = (fact1 * difference) / fact2;
    v1 + add
}

pub fn draw_pixel(buffer: &mut [u8], index: usize, pixel: [u8; 3]) {
    buffer[index] = pixel[0];
    buffer[index + 1] = pixel[1];
    buffer[index + 2] = pixel[2];
    buffer[index + 3] = 0xFF;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (crate::utils::log(&format_args!($($t)*).to_string()))
}

pub(crate) use console_log;
