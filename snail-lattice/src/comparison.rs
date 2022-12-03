use snail_lattice::lattice::SnailLattice;

const TICK_AMOUNT: usize = 1_000_000_000;

fn main() {
    let mut random_walk = SnailLattice::new("random-walk", 5, 5, 100, 0xDEAD);
    let mut random_teleport = SnailLattice::new("random-teleport", 5, 7, 100, 0xDEAD);
    let mut hold_left = SnailLattice::new("hold-left", 5, 9, 100, 0xDEAD);
    let mut tremaux = SnailLattice::new("tremaux", 5, 11, 100, 0xDEAD);
    // let mut clone = SnailLattice::new("clone", 5, 20, 100, 0xDEAD);

    println!(
        "random_walk:       {} maze fragments",
        random_walk.tick(TICK_AMOUNT)
    );
    println!(
        "random_teleport:   {} maze fragments",
        random_teleport.tick(TICK_AMOUNT)
    );
    println!(
        "hold_left:         {} maze fragments",
        hold_left.tick(TICK_AMOUNT)
    );
    println!(
        "tremaux:           {} maze fragments",
        tremaux.tick(TICK_AMOUNT)
    );
    // println!("cloning: {} maze fragments", clone.tick(TICK_AMOUNT));
}
