#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_arg_infer)]
#![feature(test)]

extern crate test;

mod direction;
mod image;
pub mod lattice;
mod lfsr;
pub mod maze;
mod snail;
pub mod solvers;
mod utils;

#[cfg(test)]
mod tests {
    use crate::{lattice::SnailLattice, maze::SNAIL_MOVEMENT_TIME, solvers::Clones};
    use test::Bencher;

    #[bench]
    fn cloning_snail_tick(b: &mut Bencher) {
        let mut lattice = SnailLattice::<100, Clones<100>>::new(10, 0xFEAD);
        lattice.alter(100);

        b.iter(|| {
            lattice.tick(SNAIL_MOVEMENT_TIME);
        });
    }

    #[bench]
    fn cloning_snail_render(b: &mut Bencher) {
        let mut lattice = SnailLattice::<100, Clones<100>>::new(10, 0xFEAD);
        lattice.alter(100);
        lattice.tick(100000);

        let dimensions = lattice.get_dimensions(100);

        let mut buffer = vec![0; 4 * dimensions[0] * dimensions[1]];

        b.iter(|| {
            lattice.render(&mut buffer, 0, 100);
        });
    }
}
