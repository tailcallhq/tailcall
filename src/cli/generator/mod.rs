#![allow(clippy::module_inception)]

pub mod config;
mod generator;
pub mod source;

pub use generator::Generator;
