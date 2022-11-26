mod utils;
mod lfsr;
mod maze;
mod lattice;
mod snail;
mod solvers;
mod direction;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
