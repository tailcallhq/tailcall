// mod synth_borrow;
// mod synth_const;
mod synth;

// pub use synth_borrow::SynthBorrow;
// pub use synth_const::{Synth, SynthConst};
pub use synth::{Synth, AlsoSynth};

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
