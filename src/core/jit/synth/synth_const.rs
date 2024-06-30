use super::super::Result;
use super::Synthesizer;
use crate::core::jit::Store;

pub struct SynthConst;
impl SynthConst {
    pub fn new() -> Self {
        Self
    }
}
impl Synthesizer for SynthConst {
    type Value = Result<async_graphql::Value>;

    fn synthesize(&self, _store: &Store<Self::Value>) -> Self::Value {
        todo!()
    }
}
