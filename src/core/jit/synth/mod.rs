mod synth;

pub use synth::{AlsoSynth, Synth};

use super::{Store, Variables};
use crate::core::json::JsonLike;
pub trait Synthesizer {
    type Value;
    type Variable: for<'a> JsonLike<'a>;
    fn synthesize(
        self,
        store: Store<Self::Value>,
        variables: Variables<Self::Variable>,
    ) -> Self::Value;
}
