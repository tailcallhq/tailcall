pub mod core;
use mimalloc::MiMalloc;

#[cfg(feature = "cli")]
pub mod cli;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
