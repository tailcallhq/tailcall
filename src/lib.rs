pub mod core;
#[cfg(feature = "cli")]
use mimalloc::MiMalloc;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg_attr(feature = "cli", global_allocator)]
static GLOBAL: MiMalloc = MiMalloc;
