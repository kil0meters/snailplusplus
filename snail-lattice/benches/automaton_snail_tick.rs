#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use test::Bencher;

    use snail_lattice::{lattice::WasmLattice, maze::SNAIL_MOVEMENT_TIME};

    #[bench]
    fn automaton_snail_tick(b: &mut Bencher) {
        let mut lattice = WasmLattice::new("automaton", 0xFEAD);
        lattice.alter(1);

        b.iter(|| {
            lattice.tick(SNAIL_MOVEMENT_TIME * 4.0);
        });
    }
}
