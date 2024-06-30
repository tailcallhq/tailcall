mod synth_borrow;
mod synth_const;

pub use synth_borrow::SynthBorrow;
pub use synth_const::SynthConst;

use super::Store;
pub trait Synthesizer {
    type Value;
    fn synthesize(self, store: Store<Self::Value>) -> Self::Value;
}
