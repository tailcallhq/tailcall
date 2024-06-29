mod builder;
mod eval;
mod model;
mod store;
mod synth;
pub use builder::*;
pub use model::*;
pub use store::*;
pub use synth::*;
mod error;

// NOTE: Only used in tests and benchmarks
pub mod common;
pub use error::*;

// TODO: implement it for Synth<Value<'a>>
trait Synthesizer {
    type Value;
    fn synthesize(&self, store: &Store<Self::Value>) -> Self::Value;
}
