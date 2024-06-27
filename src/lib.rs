// Required for the #[global_allocator] proc macro
#![allow(clippy::too_many_arguments)]

pub mod core;

#[cfg(feature = "cli")]
pub mod cli;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
