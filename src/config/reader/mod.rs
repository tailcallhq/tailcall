pub mod reader;
#[cfg(feature = "default")]
mod reader_cli;
#[cfg(not(feature = "default"))]
mod reader_wasm;

pub use reader::*;
