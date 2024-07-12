mod synth_borrow;
mod synth_const;

pub use synth_borrow::SynthBorrow;
pub use synth_const::{Synth, SynthConst};

use super::{Store, Variables};
use crate::core::json::JsonLike;
pub trait Synthesizer {
    type Value;
    type Variable: JsonLike;
    fn synthesize(
        self,
        store: Store<Self::Value>,
        variables: Variables<Self::Variable>,
    ) -> Self::Value;
}
