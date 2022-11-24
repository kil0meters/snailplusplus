mod utils;
mod lfsr;
mod snail_maze;
mod snail_lattice;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
