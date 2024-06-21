#![allow(clippy::module_inception)]

pub mod config;
mod generator;
mod serializable_header_map;
mod source;

pub use generator::Generator;
