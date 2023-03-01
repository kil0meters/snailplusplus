use std::cmp::Ordering;

use crate::{
    direction::Direction,
    image::Image,
    lfsr::LFSR,
    maze::{Maze, CELLS_PER_IDX, SNAIL_MOVEMENT_TIME},
    snail::{Snail, DEFAULT_PALETTE},
    solvers::Solver,
};

const POPULATION_COUNT: usize = 16;
const MUTATION_AMOUNT: usize = 5;

// This does not implement a real genetic algorithm because they seem to suck for mazes and end up
// being both way too slow and computationally intensive to be viable for this game, so we instead
// simulate it with something aesthetically similar.

struct LearningSnail<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fitness: usize,
    counter: usize,
    moves: Vec<Direction>,
    pub snail: Snail<S>,
}

impl<const S: usize> LearningSnail<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new_random(lfsr: &mut LFSR, length: usize) -> Self {
        Self {
            fitness: usize::MAX,
            counter: 0,
            moves: Self::random_moves(length, lfsr),
            snail: Snail::new(),
        }
    }

    fn reset(&mut self) {
        self.snail.pos.x = 0;
        self.snail.pos.y = 0;
        self.snail.active = true;
        self.fitness = usize::MAX;
        self.counter = 0;
    }

    fn random_moves(length: usize, lfsr: &mut LFSR) -> Vec<Direction> {
        let mut moves = Vec::with_capacity(S * S);

        for _ in 0..length {
            moves.push(Direction::from_number(lfsr.next().into()));
        }

        moves
    }

    fn crossover(&self, lfsr: &mut LFSR, other: &LearningSnail<S>) -> Vec<Direction> {
        let mut new_moves = self.moves.clone();
        let len = self.moves.len();

        let pos1 = lfsr.big() % len;
        let pos2 = lfsr.big() % (len - pos1) + pos1;

        if lfsr.next() % 2 == 0 {
            new_moves[0..pos1].copy_from_slice(&other.moves[0..pos1]);
        }

        if lfsr.next() % 2 == 0 {
            new_moves[pos1..pos2].copy_from_slice(&other.moves[pos1..pos2]);
        }

        if lfsr.next() % 2 == 0 {
            new_moves[pos2..len].copy_from_slice(&other.moves[pos2..len]);
        }

        new_moves
    }

    fn mutate(&mut self, solve_sequence: &Vec<Direction>, lfsr: &mut LFSR) {
        for _ in 0..MUTATION_AMOUNT {
            let i = lfsr.big() % self.moves.len();
            self.moves[i] = solve_sequence[i];
        }
    }

    fn next_move(&mut self, maze: &Maze<S>, distances: &[usize]) {
        self.snail.direction = self.moves[self.counter];
        self.snail.move_forward(maze);

        self.counter += 1;

        let dist = distances[self.snail.pos.y * S + self.snail.pos.x];
        self.fitness = self.fitness.min(dist);
    }
}

pub struct Learning<const S: usize>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    population: Vec<LearningSnail<S>>,
    generation_timer: usize,
    generation_count: usize,
    fitness: usize,
    distances: [usize; S * S],
    solve_sequence: Vec<Direction>,
    new_maze: bool,
}

impl<const S: usize> Solver<S> for Learning<S>
where
    [usize; (S * S) / CELLS_PER_IDX + 1]: Sized,
{
    fn new() -> Self {
        Learning {
            population: Vec::new(),
            generation_count: 0,
            generation_timer: 0,
            distances: [0; S * S],
            solve_sequence: Vec::new(),
            fitness: 0,
            new_maze: true,
        }
    }

    fn draw(
        &mut self,
        animation_cycle: bool,
        movement_timer: usize,
        _lfsr: &mut LFSR,
        image: &mut Image,
        bx: usize,
        by: usize,
    ) {
        let mut start = "generation:".to_string();
        start.push_str(&self.generation_count.to_string());

        image.draw_text(&start, bx + 2, by + 1 + S * 10 - 6);

        let mut start = "fitness:".to_string();
        start.push_str(&self.fitness.to_string());

        image.draw_text(&start, bx + 2, by + 1 + S * 10 - 11);

        for snail in self.population.iter() {
            snail.snail.draw(
                DEFAULT_PALETTE,
                animation_cycle,
                movement_timer,
                self.movement_time(),
                image,
                bx,
                by,
            );
        }
    }

    fn step(&mut self, maze: &Maze<S>, lfsr: &mut LFSR) -> bool {
        if self.new_maze {
            maze.get_distances(maze.end_pos.x, maze.end_pos.y, &mut self.distances);
            self.solve_sequence = maze.get_solve_sequence(0, 0, maze.end_pos);

            for snail in self.population.iter_mut() {
                snail.reset();
                snail.moves = LearningSnail::random_moves(self.solve_sequence.len(), lfsr);
            }

            self.new_maze = false;
            self.generation_count = 0;
        }

        // if empty, seed with random snails
        if self.population.len() == 0 {
            for _ in 0..POPULATION_COUNT {
                self.population
                    .push(LearningSnail::new_random(lfsr, self.solve_sequence.len()));
            }
        }

        if self.generation_timer >= self.solve_sequence.len() {
            // let distances = self.distances.clone();

            self.population
                .sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(Ordering::Equal));

            self.fitness = self.population[0].fitness;

            let mut moves_list = vec![];

            let top_selection = POPULATION_COUNT / 5;

            for snail in self.population.iter_mut() {
                snail.mutate(&self.solve_sequence, lfsr);
            }

            // cross
            for _ in 0..POPULATION_COUNT {
                let snail1 = lfsr.big() % top_selection;
                let snail2 = lfsr.big() % top_selection;

                moves_list.push(self.population[snail1].crossover(lfsr, &self.population[snail2]));
            }

            for (snail, moves) in self.population.iter_mut().zip(moves_list) {
                snail.moves = moves;
                snail.reset();
            }

            self.generation_timer = 0;
            self.generation_count += 1;
        } else {
            for snail in self
                .population
                .iter_mut()
                .filter(|snail| snail.snail.active)
            {
                snail.next_move(maze, &self.distances);

                if snail.snail.pos == maze.end_pos {
                    self.new_maze = true;
                    return true;
                }
            }

            self.generation_timer += 1;
        }

        false
    }

    fn movement_time(&self) -> usize {
        SNAIL_MOVEMENT_TIME / 2
    }
}
