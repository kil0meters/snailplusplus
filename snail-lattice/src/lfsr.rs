use std::mem::size_of;

// linear feedback shift register
// lets us generate pseudorandom numbers very quickly
// https://en.wikipedia.org/wiki/Linear-feedback_shift_register
pub struct LFSR {
    state: u16,
}

impl LFSR {
    pub fn new(seed: u16) -> LFSR {
        LFSR { state: seed }
    }

    // returns a random value between 0 and 4
    pub fn next(&mut self) -> u16 {
        let bit1 =
            ((self.state >> 0) ^ (self.state >> 2) ^ (self.state >> 3) ^ (self.state >> 5)) & 1;
        self.state = (self.state >> 1) | (bit1 << 15);

        let bit2 =
            ((self.state >> 0) ^ (self.state >> 2) ^ (self.state >> 3) ^ (self.state >> 5)) & 1;
        self.state = (self.state >> 1) | (bit2 << 15);

        (bit1 << 1) | bit2
    }

    // returns a random usize
    pub fn big(&mut self) -> usize {
        let mut res: usize = 0;

        for _ in 0..(4 * size_of::<usize>()) {
            res <<= 2;
            res |= self.next() as usize;
        }

        res
    }

    // returns the numbers 0 through 3 in a random order
    // used for exploring the cardinal directions in maze generation
    pub fn random_order(&mut self) -> [u16; 4] {
        let mut order = [0, 1, 2, 3];

        for i in (1..4).rev() {
            let next = self.next() << 2 | self.next();
            let j = next % (i + 1);
            let tmp = order[i as usize];
            order[i as usize] = order[j as usize];
            order[j as usize] = tmp;
        }

        order
    }
}
