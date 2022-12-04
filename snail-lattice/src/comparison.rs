use snail_lattice::lattice::SnailLattice;

const MICROSECONDS_PER_SECOND: usize = 1_000_000;
const SECONDS: usize = 3600;

const TICK_AMOUNT: usize = SECONDS * MICROSECONDS_PER_SECOND;

struct MazeTest {
    name: String,
    maze_size: usize,
    count: usize,
    multiplier: f64,
}

impl MazeTest {
    fn run(&self) {
        let mut lattice = SnailLattice::new(&self.name, 5, self.maze_size, self.count, 0xDEAD);

        let fragments = (lattice.tick(TICK_AMOUNT) as f64 * self.multiplier).floor() as usize;

        println!(
            "{} generated an average of {:.2} fragments/second",
            self.name,
            fragments as f64 / (SECONDS as f64 * self.count as f64)
        );
    }
}

fn main() {
    MazeTest {
        name: "random-walk".to_string(),
        maze_size: 5,
        count: 100,
        multiplier: 1.0,
    }
    .run();

    MazeTest {
        name: "random-teleport".to_string(),
        maze_size: 7,
        count: 100,
        multiplier: 1.5,
    }
    .run();

    MazeTest {
        name: "hold-left".to_string(),
        maze_size: 9,
        count: 100,
        multiplier: 1.0,
    }
    .run();

    MazeTest {
        name: "tremaux".to_string(),
        maze_size: 11,
        count: 100,
        multiplier: 5.0,
    }
    .run();

    MazeTest {
        name: "time-travel".to_string(),
        maze_size: 13,
        count: 100,
        multiplier: 9.0,
    }
    .run();

    MazeTest {
        name: "clone".to_string(),
        maze_size: 20,
        count: 100,
        multiplier: 20.0,
    }
    .run();
}
