#![feature(test)]

extern crate test;

mod direction;
pub mod lattice;
mod lfsr;
mod maze;
mod snail;
mod solvers;
mod utils;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(test)]
mod tests {
    use crate::{lattice::SnailLattice, maze::SNAIL_MOVEMENT_TIME};
    use test::Bencher;

    #[bench]
    fn cloning_snail_tick(b: &mut Bencher) {
        let mut lattice = SnailLattice::new("clone", 10, 100, 100, 0xFEAD);

        b.iter(|| {
            lattice.tick(SNAIL_MOVEMENT_TIME);
        });
    }

    #[bench]
    fn cloning_snail_render(b: &mut Bencher) {
        let mut lattice = SnailLattice::new("clone", 10, 100, 100, 0xFEAD);
        lattice.tick(100000);

        // 4 * (10)
        let dimensions = lattice.get_dimensions();
        let mut buffer = vec![0; 4 * dimensions[0] * dimensions[1]];

        b.iter(|| {
            lattice.render(&mut buffer);
        });
    }
}
