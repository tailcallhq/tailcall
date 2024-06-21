#![allow(clippy::module_inception)]

pub mod config;
mod generator;
mod source;

pub use generator::Generator;
