mod builder;
mod model;
mod exec;
mod store;
mod synth;
use async_graphql::Value;
use builder::*;
use context::Context;
use model::*;
use exec::{Executor, IRExecutor};
use store::*;
mod context;
mod error;

// NOTE: Only used in tests and benchmarks
pub mod common;
pub use error::*;
use synth::Synthesizer;

use super::blueprint::Blueprint;
use super::ir::model::IR;

#[async_trait::async_trait]
trait Jit {
    type Input;
    type Output;
    type Error;

    async fn execute(self, request: Request<Self::Input>) -> Response<Self::Output, Self::Error>;
}

pub struct ConstValueJit {
    plan: ExecutionPlan,
}

impl ConstValueJit {
    pub fn new(blueprint: Blueprint, query: String) -> Result<Self> {
        let doc = async_graphql::parser::parse_query(query)?;
        let builder = Builder::new(blueprint, doc);
        let plan = builder.build().map_err(Error::BuildError)?;
        Ok(Self { plan })
    }
}

#[async_trait::async_trait]
impl Jit for ConstValueJit {
    type Input = Value;
    type Output = Value;
    type Error = Error;

    async fn execute(self, request: Request<Self::Input>) -> Response<Self::Output, Self::Error> {
        let plan = self.plan;
        let synth = ConstValueSynth::new();
        let ir_exec = ConstValueExec::new();
        let exe = Executor::new(plan, synth, ir_exec);
        let out = exe.execute(request).await;
        Response::new(out)
    }
}

struct ConstValueExec;
impl ConstValueExec {
    pub fn new() -> Self {
        Self
    }
}
#[async_trait::async_trait]
impl IRExecutor for ConstValueExec {
    type Input = Value;
    type Output = Value;
    type Error = Error;

    async fn execute<'a>(
        &'a self,
        ir: &'a IR,
        ctx: &'a Context<'a, Self::Input, Self::Output>,
    ) -> Result<Value> {
        unimplemented!()
    }
}

struct ConstValueSynth;
impl ConstValueSynth {
    pub fn new() -> Self {
        Self
    }
}
impl Synthesizer for ConstValueSynth {
    type Value = Result<Value>;

    fn synthesize(&self, store: &Store<Self::Value>) -> Self::Value {
        unimplemented!()
    }
}
