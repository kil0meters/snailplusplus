pub struct WolfensteinGame {}

impl WolfensteinGame {
    pub fn new() -> WolfensteinGame {
        WolfensteinGame {}
    }

    pub fn resolution(&self) -> Vec<u32> {
        vec![240, 240]
    }

    pub fn render(&self, buffer: &mut [u8]) {
        for i in 0..buffer.len() {
            buffer[i] = 0xff;
        }
    }
}

