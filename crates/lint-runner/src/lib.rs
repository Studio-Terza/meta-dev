mod cli;

pub use cli::{Cli, Ports, Verdict};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
